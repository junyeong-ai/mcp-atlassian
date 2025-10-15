# mcp-atlassian Development Guide

**Rust MCP Server for Atlassian Cloud**

Last updated: 2025-10-15

---

## Project Overview

Production-ready Model Context Protocol server implementing 14 tools for Jira and Confluence with zero-copy optimizations.

| Metric | Value |
|--------|-------|
| **Language** | Rust 2024 Edition |
| **Binary** | 4.4MB (release, stripped) |
| **Tools** | 14 (8 Jira + 6 Confluence) |
| **Tests** | 183 passing (100% critical paths) |
| **Build** | 28s release, LTO enabled |
| **Warnings** | Zero (strict policy) |

### Technology Stack

```toml
tokio = "1.47"              # Async runtime
serde_json = "1.0"          # JSON processing
reqwest = "0.12"            # HTTP client (rustls)
anyhow = "1.0"              # Error handling
tracing = "0.1"             # Structured logging
```

### Build Profile

```toml
[profile.release]
opt-level = 3               # Maximum optimization
lto = true                  # Link-time optimization
codegen-units = 1           # Single codegen unit
strip = true                # Strip debug symbols
```

---

## Architecture

### Module Structure

```
src/
├── main.rs                 # Entry point, server initialization
├── config/mod.rs           # Environment config with cached base_url
├── mcp/
│   ├── server.rs           # JSON-RPC stdio server
│   ├── handlers.rs         # Tool registration (14 handlers)
│   └── types.rs            # MCP protocol types
├── tools/
│   ├── handler.rs          # ToolHandler trait
│   ├── response_optimizer.rs  # Token reduction (conditional compilation)
│   ├── jira/
│   │   ├── mod.rs          # 8 Jira handlers (zero-copy optimized)
│   │   ├── adf_utils.rs    # ADF processing (move semantics)
│   │   └── field_filtering.rs # Field optimization
│   └── confluence/
│       ├── mod.rs          # 6 Confluence handlers
│       └── field_filtering.rs # Builder pattern (consuming self)
└── utils/
    ├── http_utils.rs       # HTTP client factory
    └── logging.rs          # Stderr logging (stdout = protocol)
```

### Key Design Patterns

1. **Zero-Copy Everywhere**
   - `Config.get_atlassian_base_url()` returns `&str` (cached at init)
   - ADF processing uses move semantics (no clone)
   - `std::mem::replace()` for extracting JSON values

2. **Conditional Compilation**
   - `ResponseOptimizer` stats tracking only in tests
   - `#[cfg(test)]` for test-only code paths
   - Production builds have zero overhead

3. **Builder Pattern**
   - `FieldConfiguration.with_additional_includes(mut self)` consumes self
   - No unnecessary cloning

4. **Field Filtering**
   - Jira search: 17 optimized fields (no description)
   - Priority: API params > env override > defaults + custom > defaults

---

## Core Modules

### `config/mod.rs`

**Purpose**: Environment configuration with zero-cost URL access

```rust
pub struct Config {
    pub atlassian_domain: String,
    pub atlassian_email: String,
    pub atlassian_api_token: String,
    pub request_timeout_ms: u64,

    // Field filtering configuration
    pub jira_search_default_fields: Option<Vec<String>>,
    pub jira_search_custom_fields: Vec<String>,
    pub response_exclude_fields: Option<Vec<String>>,

    // Cached normalized base URL (zero-cost access)
    #[serde(skip)]
    pub(crate) base_url: String,
}

impl Config {
    /// Returns normalized base URL without allocation
    #[inline]
    pub fn get_atlassian_base_url(&self) -> &str {
        &self.base_url  // Zero-copy reference
    }
}
```

**Optimization**: URL normalization done once at initialization, eliminating String allocations on every API call (14 handlers × request count).

### `mcp/server.rs`

**Purpose**: JSON-RPC 2.0 stdio server

**Flow**:
1. Read line from stdin
2. Parse JSON-RPC request
3. Route to handler (`initialize`, `tools/list`, `tools/call`)
4. Execute and write response to stdout

**Protocol Versions**: Supports both `2024-11-05` and `2025-06-18`.

### `tools/jira/mod.rs`

**Purpose**: 8 Jira REST API v3 handlers with ADF support

**Handlers**:

| Handler | Endpoint | Optimization |
|---------|----------|--------------|
| GetIssue | `GET /issue/{key}` | Field filtering |
| Search | `GET /search/jql` | 17-field optimization |
| CreateIssue | `POST /issue` | ADF move semantics |
| UpdateIssue | `PUT /issue/{key}` | std::mem::replace |
| AddComment | `POST /issue/{key}/comment` | ADF zero-copy |
| UpdateComment | `PUT /issue/{key}/comment/{id}` | ADF zero-copy |
| TransitionIssue | `POST /issue/{key}/transitions` | - |
| GetTransitions | `GET /issue/{key}/transitions` | - |

**Zero-Copy Pattern**:
```rust
async fn execute(&self, mut args: Value, config: &Config) -> Result<Value> {
    // Extract value without cloning
    let description = args.get_mut("description")
        .map(|v| std::mem::replace(v, Value::Null))
        .unwrap_or(Value::Null);

    // Process with move semantics (no clone)
    let adf = adf_utils::process_description_input(description)?;
    // ...
}
```

### `tools/jira/adf_utils.rs`

**Purpose**: ADF validation and conversion with move semantics

```rust
/// Process ADF input by consuming value (no clone)
pub fn process_adf_input(value: Value, field_name: &str) -> Result<Value> {
    match value {
        Value::String(text) => Ok(text_to_adf(&text)),
        Value::Object(_) => {
            validate_adf(&value)?;
            Ok(value)  // Move, not clone
        }
        Value::Null => Ok(text_to_adf("")),
        _ => anyhow::bail!("{} must be string or ADF object", field_name)
    }
}
```

**Optimization**: Large JSON documents (10s of KB) are moved instead of cloned.

### `tools/jira/field_filtering.rs`

**Purpose**: Response payload optimization

```rust
pub const DEFAULT_SEARCH_FIELDS: &[&str] = &[
    "key", "summary", "status", "priority", "issuetype",
    "assignee", "reporter", "creator",
    "created", "updated", "duedate", "resolutiondate",
    "project", "labels", "components", "parent", "subtasks",
];

pub fn resolve_search_fields(
    api_fields: Option<Vec<String>>,
    config: &Config,
) -> Vec<String>
```

**Priority Hierarchy**:
1. API `fields` parameter (highest)
2. `JIRA_SEARCH_DEFAULT_FIELDS` env var
3. `DEFAULT_SEARCH_FIELDS` + `JIRA_SEARCH_CUSTOM_FIELDS`
4. `DEFAULT_SEARCH_FIELDS` only

**Optimization**: 17 fields vs 50+ fields, excludes heavy `description` field.

### `tools/confluence/field_filtering.rs`

**Purpose**: Builder pattern with consuming self

```rust
pub struct FieldConfiguration {
    default_params: Vec<(&'static str, &'static str)>,
    custom_includes: Vec<String>,
}

impl FieldConfiguration {
    /// Consumes self and returns modified config (no clone)
    pub fn with_additional_includes(mut self, additional: Vec<String>) -> Self {
        for param in additional {
            if !self.custom_includes.contains(&param) {
                self.custom_includes.push(param);
            }
        }
        self  // Move ownership
    }
}
```

**Optimization**: Eliminates struct cloning in builder pattern.

### `tools/response_optimizer.rs`

**Purpose**: Token reduction with conditional compilation

**Production Build**:
- No stats tracking overhead
- No `Arc<Mutex<>>` synchronization
- Optimized recursive implementation

**Test Build**:
- Stats tracking enabled
- Performance metrics
- Debug capabilities

```rust
#[cfg(not(test))]
fn optimize_recursive(&self, value: &mut Value) {
    // Production: No stats overhead
}

#[cfg(test)]
fn optimize_recursive(&self, value: &mut Value, stats: &mut OptimizationStats) {
    // Test: Track removals for verification
}
```

**Optimization**: Conditional compilation eliminates all stats overhead in production.

---

## API Tools

### Jira Tools (8)

**ADF-Enabled** (4):
- `jira_create_issue` - Accepts string or ADF for description
- `jira_update_issue` - Accepts string or ADF for description
- `jira_add_comment` - Accepts string or ADF for comment
- `jira_update_comment` - Accepts string or ADF for body

**Standard** (4):
- `jira_get_issue` - Fetch issue with field filtering
- `jira_search` - JQL search with 17-field optimization
- `jira_transition_issue` - Change workflow state
- `jira_get_transitions` - List available transitions

### Confluence Tools (6)

- `confluence_search` - CQL search (v1 API)
- `confluence_get_page` - Fetch page (v2 API)
- `confluence_get_page_children` - List children (v2 API)
- `confluence_get_comments` - Fetch comments (v2 API)
- `confluence_create_page` - Create page (v2 API)
- `confluence_update_page` - Update page with version handling (v2 API)

### ADF Support

**Validation Rules**:
- `type`: Must be "doc"
- `version`: Must be 1
- `content`: Must be array

**Supported Nodes**:
- Block: `paragraph`, `heading`, `codeBlock`, `bulletList`, `orderedList`, `listItem`
- Inline: `text` with marks (`strong`, `em`, `code`)

**Example**:
```json
{
  "type": "doc",
  "version": 1,
  "content": [{
    "type": "heading",
    "attrs": {"level": 2},
    "content": [{"type": "text", "text": "Problem"}]
  }]
}
```

---

## Configuration

### Required Environment Variables

```env
ATLASSIAN_DOMAIN=company.atlassian.net
ATLASSIAN_EMAIL=user@example.com
ATLASSIAN_API_TOKEN=token123
```

### Optional - Performance

```env
REQUEST_TIMEOUT_MS=30000     # 100-60000ms
LOG_LEVEL=warn               # error/warn/info/debug/trace
```

### Optional - Field Filtering

```env
# Override default search fields (17 built-in)
JIRA_SEARCH_DEFAULT_FIELDS="key,summary,status,assignee"

# Extend defaults with custom fields
JIRA_SEARCH_CUSTOM_FIELDS="customfield_10015,customfield_10016"

# Exclude fields from all responses (token optimization)
RESPONSE_EXCLUDE_FIELDS="avatarUrls,iconUrl,self"
```

### Optional - Access Control

```env
JIRA_PROJECTS_FILTER=PROJ1,PROJ2
CONFLUENCE_SPACES_FILTER=SPACE1,SPACE2
```

### Configuration Validation

- Domain must contain `.atlassian.net`
- Email must contain `@`
- API token cannot be empty
- Timeout: 100ms-60000ms
- URL normalization: Ensures `https://` prefix

---

## Development

### Build & Run

```bash
# Development build
cargo build

# Release build (optimized, 28s)
cargo build --release

# Run server
cargo run

# Check without building
cargo check
```

### Testing

```bash
# Run all tests (183 passing)
cargo test

# Run specific module
cargo test jira::adf_utils
cargo test field_filtering

# With output
cargo test -- --nocapture

# Coverage (tarpaulin)
cargo tarpaulin --out Stdout
```

### Code Quality

```bash
# Format (required before commit)
cargo fmt

# Lint (zero warnings policy)
cargo clippy

# Check unused dependencies
cargo udeps
```

### Manual Protocol Testing

```bash
# Initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | cargo run

# List tools
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | cargo run

# Create issue with ADF
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"jira_create_issue","arguments":{"project_key":"TEST","summary":"Test","issue_type":"Task","description":"Plain text"}}}' | cargo run
```

---

## Performance

### Optimization Strategies

1. **Cached Base URL** (Priority 1)
   - Impact: Every API call (14 handlers)
   - Technique: Pre-compute at init, return `&str`
   - Savings: String allocation per request

2. **Zero-Copy ADF Processing** (Priority 2)
   - Impact: Large JSON documents (10s of KB)
   - Technique: Move semantics, `std::mem::replace()`
   - Savings: Clone of entire ADF document

3. **Conditional Compilation** (Priority 3)
   - Impact: Stats tracking overhead
   - Technique: `#[cfg(test)]` for test-only code
   - Savings: `Arc<Mutex<>>` + time measurements

4. **Builder Pattern** (Priority 4)
   - Impact: Config struct cloning
   - Technique: Consuming `mut self`
   - Savings: Struct clone per builder call

5. **Field Filtering**
   - Jira search: 17 fields vs 50+ fields
   - Response optimizer: Remove avatars, icons, self-links
   - Technique: Query params + recursive filtering

### Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| Binary Size | 4.4MB | LTO + strip enabled |
| Build Time | 28s | Release profile |
| Test Time | 0.05s | 183 tests |
| Warnings | 0 | Strict policy |
| ADF Validation | <1ms | Top-level only |

---

## Project/Space Filtering

**Auto-injection**: If filter configured and user query doesn't specify project/space:

```rust
// Jira
let final_jql = format!("project IN ({}) AND ({})", projects, jql);

// Confluence
let final_cql = format!("space IN ({}) AND ({})", spaces, cql);
```

**User filters take precedence**: If query contains `project` or `space` keyword, no injection.

---

## Error Handling

### JSON-RPC Error Codes

| Code | Meaning |
|------|---------|
| -32700 | Parse error |
| -32600 | Invalid request |
| -32601 | Method not found |
| -32602 | Invalid params |
| -32603 | Internal error |

### ADF Validation Errors

- Clear error messages with field names
- Validation happens before API call
- Detailed type mismatch information

---

## Security

**Authentication**: HTTP Basic Auth (Base64-encoded email:token)
**Transport**: HTTPS only (enforced)
**Input Validation**: All handler parameters validated
**Access Control**: Server-side project/space filtering

---

## Resources

### Atlassian APIs
- [Jira REST API v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Confluence REST API v2](https://developer.atlassian.com/cloud/confluence/rest/v2/)
- [ADF Specification](https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/)

### MCP Protocol
- [MCP Specification](https://modelcontextprotocol.io)
- [JSON-RPC 2.0](https://www.jsonrpc.org/specification)

### Rust
- [Tokio Async Runtime](https://docs.rs/tokio)
- [Reqwest HTTP Client](https://docs.rs/reqwest)
- [Serde JSON](https://docs.rs/serde_json)

---

## License

MIT License

---

**Document optimized for AI agent comprehension - focuses on architecture, optimizations, and development workflow**
