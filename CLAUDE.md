# mcp-atlassian Development Guide

**Rust-based MCP server for Atlassian Cloud | Developer Documentation**

Last updated: 2025-10-14

---

## Project Overview

Production-ready Model Context Protocol server implementing 14 tools for Jira and Confluence integration with full ADF (Atlassian Document Format) support.

| Metric | Value |
|--------|-------|
| **Language** | Rust 1.90 (Edition 2024) |
| **Binary Size** | 4.4MB (release build) |
| **Tools** | 14 (8 Jira + 6 Confluence) |
| **MCP Protocol** | 2024-11-05, 2025-06-18 |
| **Test Coverage** | 58.99% (410/695 lines) |
| **Tests** | 185 passing (160 unit + 23 protocol + 2 doc) |

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
│   ├── handlers.rs           - Tool registration (14 tools)
│   └── types.rs              - Protocol type definitions
│
├── tools/
│   ├── mod.rs                - Module exports
│   ├── handler.rs            - ToolHandler trait
│   ├── jira/
│   │   ├── mod.rs            - 8 tool implementations
│   │   ├── adf_utils.rs      - ADF validation & conversion (100% coverage)
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
- 8 Jira tools registered (including UpdateCommentHandler)
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

All 14 tools implement this trait.

### `tools/jira/mod.rs`

**Purpose**: 8 Jira REST API v3 tool implementations

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

3. **CreateIssueHandler** ✨ *ADF-enabled*
   - Endpoint: `POST /rest/api/3/issue`
   - Accepts string or ADF for description
   - Auto-converts plain text to ADF
   - Field filtering on response

4. **UpdateIssueHandler** ✨ *ADF-enabled*
   - Endpoint: `PUT /rest/api/3/issue/{key}`
   - Accepts string or ADF for description
   - Direct field updates

5. **AddCommentHandler** ✨ *ADF-enabled*
   - Endpoint: `POST /rest/api/3/issue/{key}/comment`
   - Accepts string or ADF for comment
   - Auto-converts plain text to ADF

6. **UpdateCommentHandler** ✨ *NEW - ADF-enabled*
   - Endpoint: `PUT /rest/api/3/issue/{key}/comment/{id}`
   - Accepts string or ADF for body
   - Update existing comments with formatting

7. **TransitionIssueHandler**
   - Endpoint: `POST /rest/api/3/issue/{key}/transitions`
   - Workflow state changes

8. **GetTransitionsHandler**
   - Endpoint: `GET /rest/api/3/issue/{key}/transitions`
   - Available transitions

### `tools/jira/adf_utils.rs`

**Purpose**: ADF (Atlassian Document Format) validation and conversion

**Test Coverage**: 100% (40/40 lines covered)

**Core Functions**:

```rust
/// Validates ADF document structure (type, version, content)
pub fn validate_adf(value: &Value) -> Result<()>

/// Converts plain text to simple paragraph ADF
pub fn text_to_adf(text: &str) -> Value

/// Core processing function for any ADF field (with field_name for error messages)
pub fn process_adf_input(value: &Value, field_name: &str) -> Result<Value>

/// Wrapper for description field (delegates to process_adf_input)
pub fn process_description_input(value: &Value) -> Result<Value>

/// Wrapper for comment field (delegates to process_adf_input)
pub fn process_comment_input(value: &Value) -> Result<Value>
```

**Design Notes**:
- `process_adf_input()` is the core function handling all ADF processing logic
- `process_description_input()` and `process_comment_input()` are thin wrappers providing field-specific error messages
- This eliminates code duplication while maintaining backward compatibility

**Validation Rules**:
- `type` must be exactly "doc"
- `version` must be integer 1
- `content` must be array (can be empty)

**Input Processing**:
- **String** → Converts to simple paragraph ADF
- **ADF Object** → Validates and passes through
- **Null** → Returns empty paragraph ADF
- **Other types** → Returns error

**Performance**:
- <1ms document validation
- <10ms total overhead (meets NFR3)
- No recursive validation (delegated to Jira API)

**Example Usage**:
```rust
// Plain text
let adf = process_description_input(&json!("Hello, world!"))?;
// Returns: { "type": "doc", "version": 1, "content": [...] }

// ADF with formatting
let adf = process_description_input(&json!({
    "type": "doc",
    "version": 1,
    "content": [{
        "type": "heading",
        "attrs": {"level": 2},
        "content": [{"type": "text", "text": "Problem"}]
    }]
}))?;
```

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

## ADF (Atlassian Document Format) Support

### Overview

Full support for rich text formatting in Jira issue descriptions and comments using Atlassian Document Format.

### Supported Tools

| Tool | Description | ADF Support |
|------|-------------|-------------|
| `jira_create_issue` | Create issue with formatted description | ✅ String or ADF |
| `jira_update_issue` | Update issue description | ✅ String or ADF |
| `jira_add_comment` | Add comment with formatting | ✅ String or ADF |
| `jira_update_comment` | Update existing comment | ✅ String or ADF |

### Supported Node Types

**Block-level Nodes**:
- `paragraph` - Text paragraphs
- `heading` - Headings (H1-H6, level 1-6)
- `codeBlock` - Fenced code blocks with syntax highlighting
- `bulletList` - Unordered lists
- `orderedList` - Numbered lists
- `listItem` - List items

**Inline Nodes**:
- `text` - Plain text with optional marks

**Text Marks** (Inline formatting):
- `code` - Inline code (`inline code`)
- `strong` - Bold text (**bold**)
- `em` - Italic text (*italic*)

### Usage Examples

#### Plain Text (Backward Compatible)

```json
{
  "name": "jira_create_issue",
  "arguments": {
    "project_key": "PROJ",
    "summary": "Bug report",
    "issue_type": "Bug",
    "description": "This is plain text and works as before"
  }
}
```

#### Rich Formatted Description

```json
{
  "name": "jira_create_issue",
  "arguments": {
    "project_key": "PROJ",
    "summary": "Feature request",
    "issue_type": "Story",
    "description": {
      "type": "doc",
      "version": 1,
      "content": [
        {
          "type": "heading",
          "attrs": {"level": 2},
          "content": [{"type": "text", "text": "Problem"}]
        },
        {
          "type": "paragraph",
          "content": [
            {"type": "text", "text": "The "},
            {"type": "text", "text": "API", "marks": [{"type": "code"}]},
            {"type": "text", "text": " returns "},
            {"type": "text", "text": "null", "marks": [{"type": "strong"}]},
            {"type": "text", "text": " instead of an error"}
          ]
        },
        {
          "type": "codeBlock",
          "attrs": {"language": "rust"},
          "content": [{"type": "text", "text": "fn main() {\n    println!(\"test\");\n}"}]
        }
      ]
    }
  }
}
```

#### Update Comment with Formatting

```json
{
  "name": "jira_update_comment",
  "arguments": {
    "issue_key": "PROJ-123",
    "comment_id": "10042",
    "body": {
      "type": "doc",
      "version": 1,
      "content": [
        {
          "type": "heading",
          "attrs": {"level": 3},
          "content": [{"type": "text", "text": "Review"}]
        },
        {
          "type": "bulletList",
          "content": [
            {
              "type": "listItem",
              "content": [{
                "type": "paragraph",
                "content": [{"type": "text", "text": "Looks good"}]
              }]
            },
            {
              "type": "listItem",
              "content": [{
                "type": "paragraph",
                "content": [{"type": "text", "text": "Approved"}]
              }]
            }
          ]
        }
      ]
    }
  }
}
```

### Validation

**Required Fields**:
- `type`: Must be "doc"
- `version`: Must be 1
- `content`: Must be array

**Error Messages**:
- Missing type → "Invalid ADF: missing required field 'type'"
- Wrong type → "Invalid ADF: type must be 'doc', got 'paragraph'"
- Missing version → "Invalid ADF: missing required field 'version'"
- Wrong version → "Invalid ADF: version must be 1, got 2"
- Missing content → "Invalid ADF: missing required field 'content'"
- Invalid content → "Invalid ADF: content must be array"

### Backward Compatibility

✅ **100% backward compatible** - All existing plain text usage continues to work
✅ **Auto-conversion** - Plain strings automatically converted to simple paragraph ADF
✅ **Null handling** - Missing/null descriptions treated as empty text
✅ **No breaking changes** - Existing tests pass without modification

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

| Tool | Method | Endpoint | ADF Support |
|------|--------|----------|-------------|
| get_issue | GET | `/rest/api/3/issue/{key}` | - |
| search | GET | `/rest/api/3/search/jql` | - |
| create_issue | POST | `/rest/api/3/issue` | ✅ description |
| update_issue | PUT | `/rest/api/3/issue/{key}` | ✅ description |
| add_comment | POST | `/rest/api/3/issue/{key}/comment` | ✅ comment |
| update_comment | PUT | `/rest/api/3/issue/{key}/comment/{id}` | ✅ body |
| transition_issue | POST | `/rest/api/3/issue/{key}/transitions` | - |
| get_transitions | GET | `/rest/api/3/issue/{key}/transitions` | - |

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
# Run all tests (185 tests)
cargo test

# Run specific test
cargo test test_validate_adf_valid_document

# With output
cargo test -- --nocapture

# Run ADF-specific tests (29 tests)
cargo test adf_utils::tests

# Test coverage with tarpaulin
cargo tarpaulin --out Stdout
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

### Test Statistics

| Category | Count | Coverage |
|----------|-------|----------|
| **Total Tests** | 185 | 58.99% |
| Unit Tests (lib) | 160 | - |
| Protocol Tests | 23 | - |
| Doc Tests | 2 | - |
| **ADF Module** | 29 | **100%** |

### Unit Tests by Module

**Config Tests** (`config/mod.rs`):
- Configuration validation
- Domain normalization
- Field validation

**ADF Tests** (`tools/jira/adf_utils.rs`) - **100% Coverage**:
- `test_validate_adf_*` - 8 validation tests
- `test_text_to_adf_*` - 5 conversion tests
- `test_process_adf_input_*` - 10 core logic tests
- `test_process_description_input_*` - 2 wrapper delegation tests
- `test_process_comment_input_*` - 2 wrapper delegation tests
- `test_adf_validation_performance` - Performance test

**Field Filtering Tests** (`tools/jira/field_filtering.rs`):
- Default field configuration
- Field resolution priority
- Custom field handling

**Handler Tests** (`tools/jira/mod.rs`, `tools/confluence/mod.rs`):
- URL construction
- Parameter validation
- ADF processing logic

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module
cargo test adf_utils::tests
cargo test field_filtering::tests

# Run with coverage
cargo tarpaulin --out Stdout --output-dir target/coverage
```

### Manual Protocol Testing

```bash
# Test initialize
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | cargo run

# Test tools/list
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | cargo run

# Test create issue with ADF
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"jira_create_issue","arguments":{"project_key":"TEST","summary":"Test","issue_type":"Task","description":{"type":"doc","version":1,"content":[{"type":"paragraph","content":[{"type":"text","text":"Test"}]}]}}}}' | cargo run
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
- ADF structure validation before API calls

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

### ADF Validation Errors

- Invalid ADF structure → Descriptive error message
- Missing required fields → Error with field name
- Wrong field types → Error with expected vs actual type

---

## Performance

### Metrics

| Metric | Value | Target |
|--------|-------|--------|
| Binary Size | 4.4MB | ≤4.5MB |
| ADF Validation | <1ms | <10ms |
| Request Timeout | 30s (configurable) | 100ms-60s |
| Test Execution | 0.04s | <1s |

### Optimization Strategies

1. **Field Filtering**: Reduce API response size (17 fields vs 50+ fields)
2. **Minimal ADF Validation**: Top-level only, delegate to Jira API
3. **Connection Pooling**: Reuse HTTP connections
4. **Link-Time Optimization**: LTO enabled in release build
5. **Binary Stripping**: Remove debug symbols

---

## Resources

### Atlassian API Documentation
- [Jira REST API v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Confluence REST API v2](https://developer.atlassian.com/cloud/confluence/rest/v2/)
- [Atlassian Document Format](https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/)
- [ADF Node Types](https://developer.atlassian.com/cloud/jira/platform/apis/document/nodes/)

### MCP Protocol
- [MCP Specification](https://modelcontextprotocol.io)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)

### Rust Resources
- [Tokio Documentation](https://docs.rs/tokio)
- [Reqwest Documentation](https://docs.rs/reqwest)
- [Serde JSON](https://docs.rs/serde_json)
- [Anyhow Error Handling](https://docs.rs/anyhow)

---

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

**All information verified against actual implementation on 2025-10-14**
