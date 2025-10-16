//! Response optimization module for reducing token consumption
//!
//! This module provides functionality to remove unnecessary fields and empty strings
//! from API responses to optimize token usage for LLM interactions.

use anyhow::Result;
use serde_json::Value;
#[cfg(test)]
use std::sync::{Arc, Mutex};

/// Default fields to exclude from API responses for token optimization
///
/// These fields provide no value to LLMs and consume significant tokens:
/// - `avatarUrls`: User avatar URLs in multiple sizes (Jira)
/// - `iconUrl`: Issue type/status/priority icon URLs (Jira)
/// - `profilePicture`: User profile pictures (Confluence)
/// - `icon`: Space/content icons (Confluence)
/// - `self`: Self-referencing API URLs (both Jira and Confluence)
/// - `expand`: API expansion metadata (Jira)
/// - `avatarId`: UI avatar ID (Jira)
/// - `accountType`: Always "atlassian" (Jira/Confluence)
/// - `projectTypeKey`: Always "software" (Jira)
/// - `simplified`: Workflow setting (Jira)
/// - `_expandable`: API expansion metadata (Confluence)
/// - `childTypes`, `macroRenderedOutput`, `restrictions`: Always empty (Confluence)
/// - `breadcrumbs`: Always empty array (Confluence)
/// - `entityType`: Always "content" (Confluence)
/// - `iconCssClass`: UI CSS class (Confluence)
/// - `colorName`: UI color code (Jira)
/// - `hasScreen`, `isAvailable`, `isConditional`, `isGlobal`, `isInitial`, `isLooped`: Workflow metadata (Jira)
/// - `friendlyLastModified`: Duplicate of lastModified (Confluence)
/// - `editui`, `edituiv2`: Edit page URLs not needed for read-only operations (Confluence)
pub const DEFAULT_EXCLUDE_FIELDS: &[&str] = &[
    // Original fields (5)
    "avatarUrls",     // Jira: User avatars (16x16, 24x24, 32x32, 48x48)
    "iconUrl",        // Jira: Entity icons
    "profilePicture", // Confluence: User profile images
    "icon",           // Confluence: Space/content icons
    "self",           // Common: Self-referencing API URLs
    // Zero Risk additions (22)
    "expand",               // API expansion metadata
    "avatarId",             // UI avatar ID
    "accountType",          // 99% "atlassian" fixed value
    "projectTypeKey",       // 99% "software" fixed value
    "simplified",           // Internal workflow setting
    "_expandable",          // Confluence API metadata
    "childTypes",           // Always empty object
    "macroRenderedOutput",  // Always empty object
    "restrictions",         // Always empty object
    "breadcrumbs",          // Always empty array
    "entityType",           // Always "content" fixed value
    "iconCssClass",         // UI CSS class
    "colorName",            // UI color, name is sufficient
    "hasScreen",            // UI behavior metadata
    "isAvailable",          // Always true (filtered)
    "isConditional",        // Workflow internal setting
    "isGlobal",             // Workflow internal setting
    "isInitial",            // Workflow internal setting
    "isLooped",             // Workflow internal setting
    "friendlyLastModified", // Duplicate of lastModified
    "editui",               // Confluence edit draft URL (read-only unnecessary)
    "edituiv2",             // Confluence edit v2 URL (read-only unnecessary)
];

/// Statistics for a single optimization operation (test-only)
#[cfg(test)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OptimizationStats {
    /// Number of excluded fields removed
    pub fields_removed: usize,
    /// Number of empty string fields removed
    pub empty_strings_removed: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: f64,
}

/// Response optimizer for removing unnecessary fields and empty strings
///
/// Thread-safe and designed to be shared via `Arc` across async handlers.
pub struct ResponseOptimizer {
    exclude_fields: Vec<String>,
    remove_empty_strings: bool,
    #[cfg(test)]
    stats: Arc<Mutex<OptimizationStats>>,
}

impl ResponseOptimizer {
    /// Create optimizer from application configuration
    ///
    /// Uses `RESPONSE_EXCLUDE_FIELDS` env var if set, otherwise uses `DEFAULT_EXCLUDE_FIELDS`.
    pub fn from_config(config: &crate::config::Config) -> Self {
        let exclude_fields = if let Some(ref fields) = config.response_exclude_fields {
            tracing::info!(
                "Using {} custom response exclude fields from config",
                fields.len()
            );
            fields.clone()
        } else {
            tracing::debug!(
                "Using {} default response exclude fields",
                DEFAULT_EXCLUDE_FIELDS.len()
            );
            DEFAULT_EXCLUDE_FIELDS
                .iter()
                .map(|s| s.to_string())
                .collect()
        };

        Self {
            exclude_fields,
            remove_empty_strings: true,
            #[cfg(test)]
            stats: Arc::new(Mutex::new(OptimizationStats::default())),
        }
    }

    /// Create optimizer with custom exclusion rules (test-only)
    #[cfg(test)]
    pub fn new_with_rules(exclude_fields: Vec<String>) -> Self {
        Self {
            exclude_fields,
            remove_empty_strings: true,
            stats: Arc::new(Mutex::new(OptimizationStats::default())),
        }
    }

    /// Apply optimization to JSON response in-place
    ///
    /// Removes excluded fields and empty strings recursively using mutable reference.
    /// This approach avoids cloning for maximum performance.
    ///
    /// # Arguments
    /// * `value` - Mutable reference to JSON value (optimized in-place)
    ///
    /// # Returns
    /// * `Ok(())` - Optimization succeeded
    /// * `Err` - Currently never fails, but returns Result for future extensibility
    pub fn optimize(&self, value: &mut Value) -> Result<()> {
        #[cfg(test)]
        let start = std::time::Instant::now();
        #[cfg(test)]
        let mut stats = OptimizationStats::default();

        // Apply recursive optimization
        #[cfg(test)]
        self.optimize_recursive(value, &mut stats);
        #[cfg(not(test))]
        self.optimize_recursive(value);

        #[cfg(test)]
        {
            // Record processing time
            stats.processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;

            // Update shared stats
            if let Ok(mut shared_stats) = self.stats.lock() {
                *shared_stats = stats;
            }
        }

        Ok(())
    }

    /// Recursively optimize a JSON value (production version - no stats)
    ///
    /// Removes excluded fields and empty strings at all nesting levels.
    #[cfg(not(test))]
    fn optimize_recursive(&self, value: &mut Value) {
        match value {
            Value::Object(map) => {
                // Step 1: Remove excluded fields
                for field in &self.exclude_fields {
                    map.remove(field);
                }

                // Step 2: Remove empty strings (preserve nulls)
                if self.remove_empty_strings {
                    map.retain(|_, v| !matches!(v, Value::String(s) if s.is_empty()));
                }

                // Step 3: Recursively process nested values
                for nested_value in map.values_mut() {
                    self.optimize_recursive(nested_value);
                }
            }
            Value::Array(arr) => {
                // Recursively process array elements
                for item in arr.iter_mut() {
                    self.optimize_recursive(item);
                }
            }
            _ => {
                // Primitive types: no optimization needed
            }
        }
    }

    /// Recursively optimize a JSON value (test version - with stats)
    ///
    /// Removes excluded fields and empty strings at all nesting levels.
    #[cfg(test)]
    fn optimize_recursive(&self, value: &mut Value, stats: &mut OptimizationStats) {
        match value {
            Value::Object(map) => {
                // Step 1: Remove excluded fields
                for field in &self.exclude_fields {
                    if map.remove(field).is_some() {
                        stats.fields_removed += 1;
                    }
                }

                // Step 2: Remove empty strings (preserve nulls)
                if self.remove_empty_strings {
                    map.retain(|_, v| {
                        if let Value::String(s) = v
                            && s.is_empty()
                        {
                            stats.empty_strings_removed += 1;
                            return false; // Remove empty string
                        }
                        true // Keep everything else (including null)
                    });
                }

                // Step 3: Recursively process nested values
                for nested_value in map.values_mut() {
                    self.optimize_recursive(nested_value, stats);
                }
            }
            Value::Array(arr) => {
                // Recursively process array elements
                for item in arr.iter_mut() {
                    self.optimize_recursive(item, stats);
                }
            }
            _ => {
                // Primitive types: no optimization needed
            }
        }
    }

    /// Get statistics from last optimization operation (test-only)
    #[cfg(test)]
    pub fn get_last_optimization_stats(&self) -> OptimizationStats {
        self.stats.lock().map(|stats| *stats).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_exclude_fields_count() {
        assert_eq!(DEFAULT_EXCLUDE_FIELDS.len(), 27);
        // Original 5 fields
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"avatarUrls"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"iconUrl"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"profilePicture"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"icon"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"self"));
        // Zero Risk additions (22 fields)
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"expand"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"avatarId"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"accountType"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"projectTypeKey"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"simplified"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"friendlyLastModified"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"editui"));
        assert!(DEFAULT_EXCLUDE_FIELDS.contains(&"edituiv2"));
    }

    #[test]
    fn test_remove_avatarurls() {
        let optimizer = ResponseOptimizer::new_with_rules(vec!["avatarUrls".to_string()]);
        let mut input = json!({
            "name": "John",
            "avatarUrls": {
                "16x16": "https://example.com/16.png",
                "48x48": "https://example.com/48.png"
            }
        });

        optimizer.optimize(&mut input).unwrap();
        assert!(!input.as_object().unwrap().contains_key("avatarUrls"));
        assert_eq!(input["name"], "John");

        let stats = optimizer.get_last_optimization_stats();
        assert_eq!(stats.fields_removed, 1);
    }

    #[test]
    fn test_remove_empty_strings() {
        let optimizer = ResponseOptimizer::new_with_rules(vec![]);
        let mut input = json!({
            "name": "",
            "status": null,
            "count": 0,
            "valid": "data"
        });

        optimizer.optimize(&mut input).unwrap();
        assert!(!input.as_object().unwrap().contains_key("name"));
        assert!(input.as_object().unwrap().contains_key("status"));
        assert_eq!(input["status"], json!(null));
        assert_eq!(input["count"], 0);
        assert_eq!(input["valid"], "data");

        let stats = optimizer.get_last_optimization_stats();
        assert_eq!(stats.empty_strings_removed, 1);
    }

    #[test]
    fn test_recursive_optimization() {
        let optimizer = ResponseOptimizer::new_with_rules(vec!["iconUrl".to_string()]);
        let mut input = json!({
            "status": {
                "name": "Open",
                "iconUrl": "https://example.com/icon.png"
            },
            "assignee": {
                "displayName": "John",
                "email": ""
            }
        });

        optimizer.optimize(&mut input).unwrap();
        assert!(!input["status"].as_object().unwrap().contains_key("iconUrl"));
        assert_eq!(input["status"]["name"], "Open");
        assert!(!input["assignee"].as_object().unwrap().contains_key("email"));
        assert_eq!(input["assignee"]["displayName"], "John");

        let stats = optimizer.get_last_optimization_stats();
        assert_eq!(stats.fields_removed, 1);
        assert_eq!(stats.empty_strings_removed, 1);
    }

    #[test]
    fn test_array_optimization() {
        let optimizer = ResponseOptimizer::new_with_rules(vec!["self".to_string()]);
        let mut input = json!({
            "issues": [
                {"key": "PROJ-1", "self": "https://api/issue/1"},
                {"key": "PROJ-2", "self": "https://api/issue/2"}
            ]
        });

        optimizer.optimize(&mut input).unwrap();
        let issues = input["issues"].as_array().unwrap();
        assert_eq!(issues.len(), 2);
        assert!(!issues[0].as_object().unwrap().contains_key("self"));
        assert!(!issues[1].as_object().unwrap().contains_key("self"));
        assert_eq!(issues[0]["key"], "PROJ-1");
        assert_eq!(issues[1]["key"], "PROJ-2");

        let stats = optimizer.get_last_optimization_stats();
        assert_eq!(stats.fields_removed, 2);
    }

    #[test]
    fn test_all_exclude_fields() {
        let optimizer = ResponseOptimizer::new_with_rules(
            DEFAULT_EXCLUDE_FIELDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
        let mut input = json!({
            "key": "PROJ-123",
            "self": "https://api/issue/123",
            "assignee": {
                "displayName": "John",
                "avatarUrls": {"48x48": "url"},
                "profilePicture": {"path": "url"}
            },
            "status": {
                "name": "Open",
                "iconUrl": "url",
                "icon": {"path": "url"}
            }
        });

        optimizer.optimize(&mut input).unwrap();
        assert!(!input.as_object().unwrap().contains_key("self"));
        assert!(
            !input["assignee"]
                .as_object()
                .unwrap()
                .contains_key("avatarUrls")
        );
        assert!(
            !input["assignee"]
                .as_object()
                .unwrap()
                .contains_key("profilePicture")
        );
        assert!(!input["status"].as_object().unwrap().contains_key("iconUrl"));
        assert!(!input["status"].as_object().unwrap().contains_key("icon"));

        let stats = optimizer.get_last_optimization_stats();
        assert_eq!(stats.fields_removed, 5);
    }
}
