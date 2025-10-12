# mcp-atlassian Development Guide

**Rust-based MCP server for Atlassian Cloud | Developer Documentation**

Last updated: 2025-10-12

---

## Project Overview

Production-ready Model Context Protocol server implementing 13 tools for Jira and Confluence integration.

| Metric | Value |
|--------|-------|
| **Language** | Rust 1.90 (Edition 2024) |
| **Binary Size** | 4.4MB (release build) |
| **Tools** | 13 (7 Jira + 6 Confluence) |
| **MCP Protocol** | 2024-11-05, 2025-06-18 |

---

## Technology Stack

### Core Dependencies

```toml
[dependencies]
tokio = { version = "1.47", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
anyhow = "1.0"
async-trait = "0.1"
base64 = "0.22"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
dotenvy = "0.15"
```

### Build Configuration

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

**Result**: 4.4MB binary with full optimization

---

## Source Code Structure

```
src/
├── main.rs
│   ├─ Initialize logging
│   ├─ Load configuration
│   ├─ Create MCP server
│   └─ Handle Ctrl+C shutdown
│
├── config/
│   └── mod.rs
│       ├─ Environment variable parsing
│       ├─ Configuration validation
│       ├─ Type-safe config access
│       └─ Unit tests
│
├── mcp/
│   ├── mod.rs                - Module exports
│   ├── server.rs             - Stdio JSON-RPC server
│   ├── handlers.rs           - Tool registration
│   └── types.rs              - Protocol type definitions
│
├── tools/
│   ├── mod.rs                - Module exports
│   ├── handler.rs            - ToolHandler trait
│   ├── jira/
│   │   ├── mod.rs            - 7 tool implementations
│   │   └── field_filtering.rs - Field optimization
│   └── confluence/
│       ├── mod.rs            - 6 tool implementations
│       └── field_filtering.rs - API optimization
│
└── utils/
    ├── mod.rs                - Module exports
    ├── http_utils.rs         - HTTP client setup
    └── logging.rs            - Structured logging
```

---

## Module Descriptions

### `main.rs`

**Purpose**: Application entry point

**Responsibilities**:
- Initialize tracing to stderr
- Load `.env` file
- Validate configuration
- Create and run MCP server
- Handle Ctrl+C gracefully

**Key Code**:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    let server = McpServer::new(config).await?;
    server.run().await
}
```

### `config/mod.rs`

**Purpose**: Environment configuration management

**Struct**:
```rust
pub struct Config {
    pub atlassian_domain: String,
    pub atlassian_email: String,
    pub atlassian_api_token: String,
    pub max_connections: usize,
    pub request_timeout_ms: u64,
    pub jira_projects_filter: Vec<String>,
    pub confluence_spaces_filter: Vec<String>,

    // Jira Search Field Configuration
    pub jira_search_default_fields: Option<Vec<String>>,
    pub jira_search_custom_fields: Vec<String>,
}
```

**Validation**:
- Domain must contain `.atlassian.net`
- Email must contain `@`
- API token cannot be empty
- Max connections: 1-1000
- Request timeout: 100ms-60000ms

**URL Normalization**:
- Ensures `https://` prefix
- Converts `http://` to `https://`

### `mcp/types.rs`

**Purpose**: MCP protocol type definitions

**Protocol Versions**:
```rust
pub const PROTOCOL_VERSION: &str = "2024-11-05";
pub const PROTOCOL_VERSION_2025: &str = "2025-06-18";
```

**Core Types**:
- `JsonRpcRequest` / `JsonRpcResponse` - JSON-RPC 2.0
- `InitializeRequest` / `InitializeResult` - Protocol negotiation
- `Tool` / `ToolInputSchema` - Tool definitions
- `CallToolRequest` / `CallToolResult` - Tool execution
- `JsonRpcError` - Error codes (-32700 to -32603)

### `mcp/server.rs`

**Purpose**: MCP protocol server implementation

**Architecture**:
```rust
pub struct McpServer {
    config: Arc<Config>,
    handler: Arc<RequestHandler>,
    initialized: Arc<RwLock<bool>>,
}
```

**Request Flow**:
1. Read line from stdin
2. Parse JSON-RPC request
3. Route to handler
4. Execute method
5. Write response to stdout

**Supported Methods**:
- `initialize` - Protocol negotiation
- `initialized` - Initialization notification
- `tools/list` - List available tools
- `tools/call` - Execute a tool
- `prompts/list` - Empty list
- `resources/list` - Empty list

**Protocol Version Negotiation**:
- Accepts both 2024-11-05 and 2025-06-18
- Defaults to 2025-06-18 if not specified
- Client version takes precedence

### `mcp/handlers.rs`

**Purpose**: Tool registration and routing

**Structure**:
```rust
pub struct RequestHandler {
    tools: HashMap<String, Arc<dyn ToolHandler>>,
    config: Arc<Config>,
}
```

**Tool Registration**:
- 7 Jira tools registered
- 6 Confluence tools registered
- HashMap for O(1) lookup

**Methods**:
- `list_tools()` - Generate MCP tool schemas
- `call_tool()` - Route to appropriate handler

### `tools/handler.rs`

**Purpose**: Tool trait definition

```rust
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn execute(&self, args: Value, config: &Config) -> Result<Value>;
}
```

All 13 tools implement this trait.

### `tools/jira/mod.rs`

**Purpose**: 7 Jira REST API v3 tool implementations

**Tools Implemented**:

1. **GetIssueHandler**
   - Endpoint: `GET /rest/api/3/issue/{key}`
   - Field filtering applied
   - Custom fields support

2. **SearchHandler**
   - Endpoint: `GET /rest/api/3/search/jql`
   - Auto-inject project filter
   - Field filtering applied
   - JQL validation

3. **CreateIssueHandler**
   - Endpoint: `POST /rest/api/3/issue`
   - ADF description conversion
   - Field filtering on response

4. **UpdateIssueHandler**
   - Endpoint: `PUT /rest/api/3/issue/{key}`
   - Direct field updates

5. **AddCommentHandler**
   - Endpoint: `POST /rest/api/3/issue/{key}/comment`
   - ADF comment conversion

6. **TransitionIssueHandler**
   - Endpoint: `POST /rest/api/3/issue/{key}/transitions`
   - Workflow state changes

7. **GetTransitionsHandler**
   - Endpoint: `GET /rest/api/3/issue/{key}/transitions`
   - Available transitions

### `tools/jira/field_filtering.rs`

**Purpose**: Jira response payload optimization

**Search Default Fields**:
```rust
pub const DEFAULT_SEARCH_FIELDS: &[&str] = &[
    "key", "summary", "status", "priority", "issuetype",
    "assignee", "reporter", "creator",
    "created", "updated", "duedate", "resolutiondate",
    "project", "labels", "components",
    "parent", "subtasks",
];
```

**Field Resolution Function**:
```rust
pub fn resolve_search_fields(
    api_fields: Option<Vec<String>>,
    config: &Config,
) -> Vec<String>
```

**Priority Hierarchy**:
1. API-provided `fields` parameter
2. `JIRA_SEARCH_DEFAULT_FIELDS` environment variable
3. `DEFAULT_SEARCH_FIELDS` + `JIRA_SEARCH_CUSTOM_FIELDS`
4. `DEFAULT_SEARCH_FIELDS` only

**Query Parameter Generation**:
```rust
?fields=key,summary,status,...&expand=-renderedFields
```

**Optimization**:
- Requests only 17 fields by default
- Excludes `description` field (large text content)
- Excludes `renderedFields` via expand parameter
- Excludes `id` field (redundant with `key`)

### `tools/confluence/mod.rs`

**Purpose**: 6 Confluence REST API tool implementations

**Tools Implemented**:

1. **SearchHandler**
   - Endpoint: `GET /wiki/rest/api/search` (v1 API)
   - Auto-inject space filter
   - CQL query support
   - Only v1 tool (no v2 search endpoint exists)

2. **GetPageHandler**
   - Endpoint: `GET /wiki/api/v2/pages/{id}` (v2 API)
   - V2 field optimization

3. **GetPageChildrenHandler**
   - Endpoint: `GET /wiki/api/v2/pages/{id}/children` (v2 API)
   - Recursive child listing

4. **GetCommentsHandler**
   - Endpoint: `GET /wiki/api/v2/pages/{id}/footer-comments` (v2 API)
   - Comment threading support

5. **CreatePageHandler**
   - Endpoint: `POST /wiki/api/v2/pages` (v2 API)
   - Space key → space ID conversion
   - HTML storage format

6. **UpdatePageHandler**
   - Endpoint: `PUT /wiki/api/v2/pages/{id}` (v2 API)
   - Auto-fetch and increment version
   - Version conflict handling

### `tools/confluence/field_filtering.rs`

**Purpose**: Confluence v2 API parameter optimization

**Default Parameters**:
```rust
vec![
    ("body-format", "storage"),
    ("include-version", "true"),
]
```

**Custom Includes**:
- Environment: `CONFLUENCE_CUSTOM_INCLUDES`
- Valid values: `ancestors`, `children`, `history`, `operations`, `labels`, `properties`
- Maps to v2 `include-*` boolean parameters

**V1 vs V2**:
- V1 (search): Single `expand` parameter
- V2 (all others): Individual `include-*` parameters

### `utils/http_utils.rs`

**Purpose**: HTTP client configuration

**Functions**:
- `create_atlassian_client()` - Reqwest client with pooling and timeout
- `create_auth_header()` - Base64 Basic auth header

**Auth Format**:
```
Authorization: Basic base64(email:api_token)
```

### `utils/logging.rs`

**Purpose**: Structured logging configuration

**Setup**:
- Tracing subscriber to stderr only
- Stdout reserved for MCP protocol
- `LOG_LEVEL` environment variable support
- Default: `warn` level

**Log Macros**:
```rust
log_startup!(config);
log_request!(request_id, method);
log_response!(request_id, latency_ms, token_count);
log_shutdown!();
```

---

## Configuration System

### Environment Variables

#### Required
```env
ATLASSIAN_DOMAIN=company.atlassian.net
ATLASSIAN_EMAIL=user@example.com
ATLASSIAN_API_TOKEN=abc123
```

#### Optional - Performance
```env
MAX_CONNECTIONS=100          # HTTP pool size (1-1000)
REQUEST_TIMEOUT_MS=30000     # Timeout ms (100-60000)
LOG_LEVEL=warn              # error/warn/info/debug/trace
```

#### Optional - Jira Search Field Configuration
```env
# Override default fields completely (17 built-in fields)
JIRA_SEARCH_DEFAULT_FIELDS="key,summary,status,assignee,priority"

# Add custom fields to built-in defaults
JIRA_SEARCH_CUSTOM_FIELDS="customfield_10015,customfield_10016"
```

#### Optional - Scoped Access
```env
JIRA_PROJECTS_FILTER=PROJ1,PROJ2,PROJ3
CONFLUENCE_SPACES_FILTER=SPACE1,SPACE2
```

### Validation Rules

From `config/mod.rs`:

1. **Domain**: Must contain `.atlassian.net`
2. **Email**: Must contain `@`
3. **API Token**: Cannot be empty
4. **Max Connections**: 1-1000
5. **Request Timeout**: 100ms-60000ms

---

## API Endpoints

### Jira REST API v3

| Tool | Method | Endpoint |
|------|--------|----------|
| get_issue | GET | `/rest/api/3/issue/{key}` |
| search | GET | `/rest/api/3/search/jql` |
| create_issue | POST | `/rest/api/3/issue` |
| update_issue | PUT | `/rest/api/3/issue/{key}` |
| add_comment | POST | `/rest/api/3/issue/{key}/comment` |
| transition_issue | POST | `/rest/api/3/issue/{key}/transitions` |
| get_transitions | GET | `/rest/api/3/issue/{key}/transitions` |

### Confluence REST API

| Tool | Method | Endpoint | API Version |
|------|--------|----------|-------------|
| search | GET | `/wiki/rest/api/search` | v1 |
| get_page | GET | `/wiki/api/v2/pages/{id}` | v2 |
| get_page_children | GET | `/wiki/api/v2/pages/{id}/children` | v2 |
| get_comments | GET | `/wiki/api/v2/pages/{id}/footer-comments` | v2 |
| create_page | POST | `/wiki/api/v2/pages` | v2 |
| update_page | PUT | `/wiki/api/v2/pages/{id}` | v2 |

**Note**: Search uses v1 API because v2 does not provide a search endpoint.

---

## Field Filtering Implementation

### Jira Search Optimization

**Search Default Fields** (17 optimized fields):
- Identification: `key`
- Core Metadata: `summary`, `status`, `priority`, `issuetype`
- People: `assignee`, `reporter`, `creator`
- Dates: `created`, `updated`, `duedate`, `resolutiondate`
- Classification: `project`, `labels`, `components`
- Hierarchy: `parent`, `subtasks`

**Removed Fields** (for token efficiency):
- `id` - Redundant with `key`
- `description` - Heavy field (10-14K tokens), use `jira_get_issue` for details

**Field Resolution Priority**:
1. API `fields` parameter (highest)
2. `JIRA_SEARCH_DEFAULT_FIELDS` env var (override defaults)
3. Built-in defaults + `JIRA_SEARCH_CUSTOM_FIELDS` (extend defaults)
4. Built-in defaults only (fallback)

**Configuration**:
```env
# Override default fields completely
JIRA_SEARCH_DEFAULT_FIELDS="key,summary,status,assignee"

# Add custom fields to defaults
JIRA_SEARCH_CUSTOM_FIELDS="customfield_10015,customfield_10016"
```

**Runtime Override**:
```json
{
  "jql": "project = KEY",
  "limit": 20,
  "fields": ["key", "summary", "status", "assignee"]
}
```

**Optimization**:
```rust
?fields=key,summary,status,...&expand=-renderedFields
```

**Configuration**:
- Default: 17 fields (excludes description)
- Default limit: 20 issues per query
- Configurable via environment variables or API parameters

**Other Tools** (GetIssue, CreateIssue, etc.):
- Use simplified 11-field filtering
- Include `description` for detail views

### Confluence Field Filtering

**V2 Default Parameters**:
```rust
?body-format=storage&include-version=true
```

**Custom Includes**:
- `include-ancestors=true`
- `include-history=true`
- `include-labels=true`
- etc.

**V1 Expand** (search only):
```rust
?expand=ancestors,history
```

---

## Project/Space Filtering

### Purpose
1. **Security**: Prevent unauthorized access
2. **Performance**: Reduce search scope
3. **Compliance**: Enforce data policies

### Jira Implementation

From `tools/jira/mod.rs`:

```rust
let final_jql = if !config.jira_projects_filter.is_empty() {
    if jql_lower.contains("project ") || ... {
        jql.to_string()  // User specified project
    } else {
        format!("project IN ({}) AND ({})", projects, jql)
    }
} else {
    jql.to_string()
};
```

**Behavior**:
- Auto-inject filter if not present
- User-specified filters take precedence
- Empty list = no filtering

### Confluence Implementation

From `tools/confluence/mod.rs`:

```rust
let final_cql = if !config.confluence_spaces_filter.is_empty() {
    if cql_lower.contains("space ") || ... {
        cql.to_string()  // User specified space
    } else {
        format!("space IN ({}) AND ({})", spaces, cql)
    }
} else {
    cql.to_string()
};
```

**Behavior**: Same as Jira

---

## Development Workflow

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run with .env
cargo run

# Check without building
cargo check
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_config_validation

# With output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint (zero warnings policy)
cargo clippy

# Check unused dependencies
cargo udeps
```

---

## Build Configuration

From `Cargo.toml`:

```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen unit
strip = true            # Strip symbols
```

**Result**:
- Binary size: 4.4MB
- Full optimization enabled
- Debug symbols removed

---

## Testing

### Unit Tests (11 total)

**Config Tests** (`config/mod.rs`):
- `test_config_validation` - Valid configuration
- `test_invalid_domain` - Domain validation

**Field Filtering Tests** (`tools/jira/field_filtering.rs`):
- `test_default_search_fields_count` - Verify 17 default fields
- `test_default_fields_no_description` - Verify description exclusion
- `test_default_fields_no_id` - Verify id exclusion
- `test_resolve_priority_1_api_fields` - API parameter priority
- `test_resolve_priority_2_env_override` - Environment variable override
- `test_resolve_priority_3_defaults_with_custom` - Defaults + custom fields
- `test_resolve_priority_4_defaults_only` - Built-in defaults only
- `test_resolve_empty_api_fields_fallback` - Empty array handling
- `test_new_fields_included` - Verify new fields (duedate, labels, etc.)

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module
cargo test config::tests
cargo test field_filtering::tests
```

### Manual Protocol Testing

```bash
# Test initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | cargo run

# Test tools/list
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | cargo run
```

---

## Security

### Authentication

**Method**: HTTP Basic Auth
**Format**: `Authorization: Basic base64(email:api_token)`
**Transport**: HTTPS only

### Input Validation

- Required parameters checked in handlers
- JQL/CQL passed to Atlassian API
- No SQL injection risk (REST API only)
- JSON schema validation

### Scoped Access

- Project/space filtering applied server-side
- Cannot be bypassed by client
- User filters take precedence (no override)

---

## Error Handling

### JSON-RPC Error Codes

From `mcp/types.rs`:

```rust
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;
```

### Error Flow

1. Parse error → `-32700`
2. Invalid JSON-RPC → `-32600`
3. Unknown method → `-32601`
4. Missing params → `-32602`
5. Tool execution error → `-32603`

---

## Resources

### Atlassian API Documentation
- [Jira REST API v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Confluence REST API v2](https://developer.atlassian.com/cloud/confluence/rest/v2/)
- [Atlassian Document Format](https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/)

### MCP Protocol
- [MCP Specification](https://modelcontextprotocol.io)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)

### Rust Resources
- [Tokio Documentation](https://docs.rs/tokio)
- [Reqwest Documentation](https://docs.rs/reqwest)
- [Serde JSON](https://docs.rs/serde_json)

---

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

**All information verified against actual implementation on 2025-10-12**
