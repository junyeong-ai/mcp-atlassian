use mcp_atlassian::config::Config;

pub struct ConfigBuilder {
    domain: String,
    email: String,
    token: String,
    max_connections: usize,
    request_timeout_ms: u64,
    jira_projects_filter: Vec<String>,
    confluence_spaces_filter: Vec<String>,
    jira_search_default_fields: Option<Vec<String>>,
    jira_search_custom_fields: Vec<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            domain: "test.atlassian.net".to_string(),
            email: "test@example.com".to_string(),
            token: "test-token".to_string(),
            max_connections: 10,
            request_timeout_ms: 30000,
            jira_projects_filter: vec![],
            confluence_spaces_filter: vec![],
            jira_search_default_fields: None,
            jira_search_custom_fields: vec![],
        }
    }

    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = domain.into();
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = token.into();
        self
    }

    pub fn with_jira_filter(mut self, projects: Vec<String>) -> Self {
        self.jira_projects_filter = projects;
        self
    }

    pub fn with_confluence_filter(mut self, spaces: Vec<String>) -> Self {
        self.confluence_spaces_filter = spaces;
        self
    }

    pub fn with_invalid_token(mut self) -> Self {
        self.token = "invalid-token".to_string();
        self
    }

    pub fn build(self) -> Config {
        Config {
            atlassian_domain: if self.domain.starts_with("http") {
                self.domain
            } else {
                format!("https://{}", self.domain)
            },
            atlassian_email: self.email,
            atlassian_api_token: self.token,
            max_connections: self.max_connections,
            request_timeout_ms: self.request_timeout_ms,
            jira_projects_filter: self.jira_projects_filter,
            confluence_spaces_filter: self.confluence_spaces_filter,
            jira_search_default_fields: self.jira_search_default_fields,
            jira_search_custom_fields: self.jira_search_custom_fields,
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Placeholder structs for JSON fixture loading
pub struct JiraFixtures;
pub struct ConfluenceFixtures;

impl JiraFixtures {
    pub fn search_response() -> serde_json::Value {
        serde_json::json!({
            "total": 2,
            "startAt": 0,
            "maxResults": 20,
            "issues": [
                {
                    "key": "TEST-123",
                    "fields": {
                        "summary": "Test issue 1",
                        "status": {"name": "Open"},
                        "priority": {"name": "High"}
                    }
                }
            ]
        })
    }

    pub fn issue_response() -> serde_json::Value {
        serde_json::json!({
            "key": "TEST-123",
            "fields": {
                "summary": "Test issue",
                "description": "Test description",
                "status": {"name": "Open"},
                "priority": {"name": "High"}
            }
        })
    }

    pub fn auth_error() -> serde_json::Value {
        serde_json::json!({
            "errorMessages": ["Unauthorized"],
            "errors": {}
        })
    }
}

impl ConfluenceFixtures {
    pub fn page_response() -> serde_json::Value {
        serde_json::json!({
            "id": "12345",
            "status": "current",
            "title": "Test Page",
            "spaceId": "67890",
            "body": {
                "storage": {
                    "value": "<p>Test content</p>",
                    "representation": "storage"
                }
            },
            "version": {
                "number": 1
            }
        })
    }

    pub fn search_response() -> serde_json::Value {
        serde_json::json!({
            "results": [
                {
                    "content": {
                        "id": "12345",
                        "type": "page",
                        "title": "Test Page"
                    }
                }
            ],
            "size": 1,
            "totalSize": 1
        })
    }
}
