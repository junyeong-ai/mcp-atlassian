# 🔧 MCP Atlassian

> Ultra-lightweight High-Performance Atlassian MCP Server for AI Agents

Model Context Protocol server that enables AI Agents like Claude and ChatGPT to directly control Jira and Confluence.
Built with Rust, delivering **4.4MB binary** with **Zero-Copy optimizations** and **fast execution**.

[![CI](https://github.com/junyeong-ai/mcp-atlassian/workflows/CI/badge.svg)](https://github.com/junyeong-ai/mcp-atlassian/actions)
[![codecov](https://codecov.io/gh/junyeong-ai/mcp-atlassian/branch/main/graph/badge.svg)](https://codecov.io/gh/junyeong-ai/mcp-atlassian)
[![Tools](https://img.shields.io/badge/MCP%20tools-14-blue?style=flat-square)](#🔧-14-mcp-tools)
[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05%20%7C%202025--06--18-blue?style=flat-square)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)

[한국어](README.md) | **[English](README.en.md)**

---

## 📖 Table of Contents

- [Why mcp-atlassian?](#🤖-why-mcp-atlassian)
- [AI Agent Use Cases](#💬-ai-agent-use-cases)
- [Quick Start (3 Steps)](#🚀-quick-start-3-steps)
- [Environment Variables Guide](#🎛️-environment-variables-guide)
- [Jira Search Field Optimization](#🔍-jira-search-field-optimization)
- [Tech Stack](#📊-tech-stack)
- [Project Structure](#🏗️-project-structure)
- [Development](#🛠️-development)
- [Security](#🔐-security)
- [Troubleshooting](#❓-troubleshooting)
- [Resources](#📚-resources)
- [License](#📝-license)
- [Contributing](#🤝-contributing)

---

## 🤖 Why mcp-atlassian?

Provides **optimized experience** for AI Agents using Atlassian:

### 🚀 Rust-Based High-Performance Self-Hosted

- **4.4MB Single Binary**: No runtime dependencies required
- **Instant Execution**: Native binary with fast startup
- **Low Resource**: Rust's memory efficiency

### ✨ Perfect ADF Support for Rich Text Formatting

**Native Atlassian Document Format Support**

- **4 Tools with Perfect ADF**: `jira_create_issue`, `jira_update_issue`, `jira_add_comment`, `jira_update_comment`
- **Auto-conversion**: Plain text → ADF automatic conversion (100% backward compatible)
- **Optimized Validation**: <1ms document validation (top-level only)
- **Zero-Copy Processing**: Efficient large document handling with move semantics

**Supported Formatting**:
- **Block**: Headings (H1-H6), code blocks (syntax highlighting), lists (ordered/unordered)
- **Inline**: Bold, italic, inline code
- **Nested**: Full support for complex document structures

**Example**:
```json
{
  "type": "doc",
  "version": 1,
  "content": [
    {
      "type": "heading",
      "attrs": {"level": 2},
      "content": [{"type": "text", "text": "Bug Fix"}]
    },
    {
      "type": "codeBlock",
      "attrs": {"language": "rust"},
      "content": [{"type": "text", "text": "fn main() { ... }"}]
    }
  ]
}
```

### 🎯 Response Optimization for AI Agents

**Smart Filtering for Maximum Token Efficiency**

#### Jira Search Optimization
- **17 Essential Fields**: Excludes description, removes unnecessary fields
- **Auto-filtering**: Automatically removes avatarUrls, iconUrl, self, etc.
- **Environment Control**: Project-specific field customization
- **Priority Hierarchy**: API → environment → defaults + custom → defaults

**Default 17 Fields**:
```
key, summary, status, priority, issuetype, assignee,
reporter, creator, created, updated, duedate, resolutiondate,
project, labels, components, parent, subtasks
```

**Response Size Comparison**:
```
Default Response: ~50+ fields, includes large description
Optimized Response: 17 fields, essential info only (60-70% reduction)
```

#### Conditional Compilation Optimization
- **Production Builds**: Complete stats tracking removal, Arc<Mutex<>> overhead eliminated
- **Test Builds**: Full debugging capabilities preserved
- **Result**: Zero overhead production execution

### 🔧 14 MCP Tools

**Jira (8 tools)** - 4 with ADF support:
- `jira_search` - JQL search (optimized 17 fields)
- `jira_get_issue` - Get issue details
- `jira_create_issue` ✨ - Create issue (ADF support)
- `jira_update_issue` ✨ - Update issue (ADF support)
- `jira_add_comment` ✨ - Add comment (ADF support)
- `jira_update_comment` ✨ - Update comment (ADF support)
- `jira_transition_issue` - Transition status
- `jira_get_transitions` - Get available transitions

**Confluence (6 tools)**:
- `confluence_search` - CQL search
- `confluence_get_page` - Get page
- `confluence_get_page_children` - List child pages
- `confluence_get_comments` - Get comments
- `confluence_create_page` - Create page
- `confluence_update_page` - Update page

### 🔒 Secure Access Control

- **Project/Space Filtering**: Access only specific projects/spaces
- **Environment-based Auth**: Secure API Token management
- **HTTPS Only**: All communications encrypted

---

## 💬 AI Agent Use Cases

### With Claude Desktop
```
User: "Show me bugs created this week"
→ AI Agent automatically calls jira_search tool
→ Returns optimized 17-field response (token savings)
→ Zero-copy for fast response

User: "Add a code review completed comment to PROJ-123"
→ AI Agent calls jira_add_comment tool
→ Plain text auto-converts to ADF
→ Move semantics for efficient processing

User: "Create a formatted release notes issue"
→ AI Agent calls jira_create_issue
→ Auto-generates headings, code blocks, lists in ADF
→ Zero-copy for large documents

User: "Update comment 10042 to say 'Approved'"
→ AI Agent calls jira_update_comment tool
→ Comment updated with ADF formatting support
→ std::mem::replace for copy-free update

User: "Create a project README page"
→ AI Agent calls confluence_create_page tool
→ Auto-verifies space and creates page
```

---

## 🚀 Quick Start (3 Steps)

**Prerequisites:** Rust 1.90+ required ([Installation Guide](https://www.rust-lang.org/tools/install))
**Total Time:** ~10 minutes (if Rust already installed) ⚡

### 1️⃣ Build (⏱️ ~5 min)

```bash
# Install Rust (1.90+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/junyeong-ai/mcp-atlassian.git
cd mcp-atlassian

# Release build (LTO + optimizations)
cargo build --release

# Binary location: target/release/mcp-atlassian (4.4MB)
```

### 2️⃣ Environment Configuration (⏱️ ~3 min)

Create `.env` file:

```env
# Required (3 variables)
ATLASSIAN_DOMAIN=yourcompany.atlassian.net
ATLASSIAN_EMAIL=you@example.com
ATLASSIAN_API_TOKEN=your_api_token_here

# Optional - Field Optimization (default: 17 fields)
JIRA_SEARCH_DEFAULT_FIELDS=key,summary,status,assignee
JIRA_SEARCH_CUSTOM_FIELDS=customfield_10015,customfield_10016

# Optional - Response Optimization (default 25 fields auto-removed, specify additional only)
# RESPONSE_EXCLUDE_FIELDS=customField1,customField2

# Optional - Access Control
JIRA_PROJECTS_FILTER=PROJ1,PROJ2
CONFLUENCE_SPACES_FILTER=SPACE1,SPACE2

# Optional - Performance
REQUEST_TIMEOUT_MS=30000
LOG_LEVEL=warn
```

**Generate API Token**: [Atlassian API Tokens](https://id.atlassian.com/manage-profile/security/api-tokens)

### 3️⃣ Connect to Claude Desktop (⏱️ ~2 min)

Edit `claude_desktop_config.json`:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "atlassian": {
      "command": "/Users/yourname/mcp-atlassian/target/release/mcp-atlassian",
      "env": {
        "ATLASSIAN_DOMAIN": "yourcompany.atlassian.net",
        "ATLASSIAN_EMAIL": "you@example.com",
        "ATLASSIAN_API_TOKEN": "your_api_token_here"
      }
    }
  }
}
```

Restart Claude Desktop → 🎉 Ready to use!

---

## 🎛️ Environment Variables Guide

### Field Optimization

#### `JIRA_SEARCH_DEFAULT_FIELDS`
Completely replaces default fields.

```env
# Request minimal fields only (maximum optimization)
JIRA_SEARCH_DEFAULT_FIELDS=key,summary,status,assignee
```

#### `JIRA_SEARCH_CUSTOM_FIELDS`
Extends the default 17 fields with additional fields.

```env
# Default 17 + 2 custom fields = 19 total
JIRA_SEARCH_CUSTOM_FIELDS=customfield_10015,customfield_10016
```

#### `RESPONSE_EXCLUDE_FIELDS`
Removes specific fields from all responses (token optimization).

```env
# Default (25 fields, 20-30% token reduction):
# - UI metadata: avatarUrls, iconUrl, profilePicture, icon, avatarId,
#                colorName, iconCssClass
# - API metadata: expand, _expandable, self
# - Fixed values: accountType, projectTypeKey, simplified, entityType
# - Empty objects/arrays: childTypes, macroRenderedOutput, restrictions, breadcrumbs
# - Workflow metadata: hasScreen, isAvailable, isConditional, isGlobal,
#                      isInitial, isLooped
# - Duplicates: friendlyLastModified

# Specify only additional fields (default 25 are auto-applied)
RESPONSE_EXCLUDE_FIELDS=customField1,customField2
```

**Field Resolution Priority**:

```
┌─────────────────────────────────┐
│ 1. API fields parameter         │  ← Highest (explicit request)
└─────────────────────────────────┘
           ↓ (if not provided)
┌─────────────────────────────────┐
│ 2. JIRA_SEARCH_DEFAULT_FIELDS   │  ← Completely replaces defaults
└─────────────────────────────────┘
           ↓ (if not provided)
┌─────────────────────────────────┐
│ 3. Default 17 fields             │  ← Built-in defaults
│    + JIRA_SEARCH_CUSTOM_FIELDS  │     (optional extension)
└─────────────────────────────────┘
           ↓ (applied to all responses)
┌─────────────────────────────────┐
│ 4. RESPONSE_EXCLUDE_FIELDS      │  ← Remove unnecessary metadata
└─────────────────────────────────┘
```

### Access Control

#### `JIRA_PROJECTS_FILTER`
Allow access to specific Jira projects only:

```env
JIRA_PROJECTS_FILTER=TEAM1,TEAM2,PROJ3
```

Auto-adds filter if AI Agent doesn't specify project in JQL:
```
User JQL: status = Open
Actual execution: project IN ("TEAM1","TEAM2","PROJ3") AND (status = Open)
```

#### `CONFLUENCE_SPACES_FILTER`
Allow access to specific Confluence spaces only:

```env
CONFLUENCE_SPACES_FILTER=TEAM,DOCS,KB
```

### Performance Tuning

#### `REQUEST_TIMEOUT_MS`
API request timeout (default: 30000ms):

```env
REQUEST_TIMEOUT_MS=10000  # Fast fail (fast network)
REQUEST_TIMEOUT_MS=60000  # Slow network tolerance
```

#### `LOG_LEVEL`
Log verbosity (default: warn):

```env
LOG_LEVEL=error  # Errors only
LOG_LEVEL=info   # Detailed logs
LOG_LEVEL=debug  # Debugging
```

---

## 🔍 Jira Search Field Optimization

### Default 17 Fields (By Category)

| Category | Field | Description |
|---------|------|-------------|
| 🔑 **Identification** | `key` | Unique issue key (e.g., PROJ-123) |
| 📝 **Core Metadata** | `summary` | Issue title |
| | `status` | Current status (Open, In Progress, etc.) |
| | `priority` | Priority level (High, Medium, Low) |
| | `issuetype` | Issue type (Bug, Task, Story, etc.) |
| 👥 **People** | `assignee` | Assigned user |
| | `reporter` | Issue reporter |
| | `creator` | Issue creator |
| 📅 **Dates** | `created` | Creation date |
| | `updated` | Last update date |
| | `duedate` | Due date |
| | `resolutiondate` | Resolution date |
| 🏷️ **Classification** | `project` | Project information |
| | `labels` | Label list |
| | `components` | Component list |
| 🌳 **Hierarchy** | `parent` | Parent issue |
| | `subtasks` | Subtask list |

### Excluded Fields

- **`description`**: Large text field (included only in detail view)
- **`id`**: Redundant with `key`
- **`renderedFields`**: Rendered HTML (expand=-renderedFields)

### Practical Usage

```bash
# Method 1: Specify in API call (highest priority)
{
  "jql": "project = KEY",
  "fields": ["key", "summary", "status"]
}

# Method 2: Override defaults with environment variable
JIRA_SEARCH_DEFAULT_FIELDS=key,summary,status,assignee

# Method 3: Extend defaults
JIRA_SEARCH_CUSTOM_FIELDS=customfield_10015

# Method 4: Remove unnecessary fields from response (default 25 auto-applied, specify additional only)
# RESPONSE_EXCLUDE_FIELDS=customField1,customField2
```

---

## 📊 Tech Stack

| Component | Technology | Features |
|-----------|-----------|----------|
| **Language** | Rust 1.90+ (Edition 2024) | Memory safety, high performance |
| **Runtime** | Tokio 1.47 | Async I/O |
| **HTTP** | Reqwest 0.12 (rustls) | TLS 1.2+ support |
| **Serialization** | Serde 1.0 | JSON processing |
| **Logging** | Tracing 0.1 | Structured logging |
| **Build Optimization** | LTO + Strip | 4.4MB binary |

### API Versions
- **Jira**: REST API v3
- **Confluence**: REST API v2 (v1 for search only)

### MCP Protocol
- JSON-RPC 2.0 over stdio
- Supported versions: `2024-11-05`, `2025-06-18`

---

## 🏗️ Project Structure

```
src/
├── main.rs                   # Entry point
├── config/
│   └── mod.rs                # Environment config
├── mcp/
│   ├── server.rs             # MCP protocol server
│   ├── handlers.rs           # Tool registration
│   └── types.rs              # MCP type definitions
├── tools/
│   ├── handler.rs            # ToolHandler trait
│   ├── response_optimizer.rs # Response optimization
│   ├── jira/
│   │   ├── mod.rs            # 8 Jira tools
│   │   ├── adf_utils.rs      # ADF validation & conversion
│   │   └── field_filtering.rs # Field optimization
│   └── confluence/
│       ├── mod.rs            # 6 Confluence tools
│       └── field_filtering.rs # API optimization
└── utils/
    ├── http_utils.rs         # HTTP client
    └── logging.rs            # Structured logging
```

---

## 🛠️ Development

### Build

```bash
# Development build
cargo build

# Release build (optimized, 28s)
cargo build --release

# Run directly
cargo run

# Type check only
cargo check
```

### Testing

```bash
# All tests (180 tests, 0.05s)
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_config_validation

# ADF tests only
cargo test adf_utils::tests
```

### Code Quality

```bash
# Formatting
cargo fmt

# Lint (zero warnings policy)
cargo clippy

# Full check
cargo check && cargo clippy && cargo test
```

### Release Build Configuration

```toml
[profile.release]
opt-level = 3       # Maximum optimization
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
strip = true        # Strip symbols
```

**Result**: 4.4MB optimized binary

---

## 🔐 Security

### Authentication
- **Method**: HTTP Basic Auth
- **Format**: `Authorization: Basic base64(email:api_token)`
- **Transport**: HTTPS only

### Input Validation
- Required parameter validation
- JQL/CQL passed to Atlassian API
- JSON schema validation
- ADF structure validation

### Access Control
- Project/space filtering (server-side)
- User-specified filters take precedence
- Cannot be bypassed

---

## ❓ Troubleshooting

### Tools not showing in Claude Desktop

**Solutions:**

1. **Check configuration file**
   ```bash
   # macOS
   cat ~/Library/Application\ Support/Claude/claude_desktop_config.json

   # Windows
   type %APPDATA%\Claude\claude_desktop_config.json
   ```

2. **Completely restart Claude Desktop**
   - Use "Quit" from menu (not just close window)
   - Restart application

3. **Verify binary path**
   ```bash
   # Check if binary exists
   ls -la target/release/mcp-atlassian

   # Ensure execute permission
   chmod +x target/release/mcp-atlassian
   ```

### Atlassian API connection failure

**Cause 1: API Token error**
- Generate new token at [Atlassian API Tokens](https://id.atlassian.com/manage-profile/security/api-tokens)
- Verify correct token in `.env` or `claude_desktop_config.json`

**Cause 2: Domain configuration error**
```env
# Correct format (without https://)
ATLASSIAN_DOMAIN=yourcompany.atlassian.net

# Incorrect format
ATLASSIAN_DOMAIN=https://yourcompany.atlassian.net  ❌
```

**Cause 3: Network timeout**
```env
# Increase timeout (default: 30000ms)
REQUEST_TIMEOUT_MS=60000
```

### Want to access specific projects only

Use `JIRA_PROJECTS_FILTER`:
```env
JIRA_PROJECTS_FILTER=PROJ1,PROJ2,PROJ3
```

See [Access Control](#access-control) section for details.

### Want to add custom fields

Use `JIRA_SEARCH_CUSTOM_FIELDS`:
```env
JIRA_SEARCH_CUSTOM_FIELDS=customfield_10015,customfield_10016
```

See [Field Optimization](#field-optimization) section for details.

### How to check logs

```env
# Change log level in .env file
LOG_LEVEL=debug  # Choose: error, warn, info, debug, trace
```

```bash
# Check server logs on macOS (Claude Desktop logs)
tail -f ~/Library/Logs/Claude/mcp*.log

# Or run directly to see logs
./target/release/mcp-atlassian
```

---

## 📚 Resources

### Atlassian API
- [Jira REST API v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Confluence REST API v2](https://developer.atlassian.com/cloud/confluence/rest/v2/)
- [Atlassian Document Format](https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/)

### MCP
- [MCP Specification](https://modelcontextprotocol.io)
- [JSON-RPC 2.0](https://www.jsonrpc.org/specification)

### Rust
- [Tokio](https://docs.rs/tokio)
- [Reqwest](https://docs.rs/reqwest)
- [Serde JSON](https://docs.rs/serde_json)

---

## 📝 License

MIT License - see [LICENSE](LICENSE) file

---

## 🤝 Contributing

Issues and Pull Requests welcome!

1. Fork
2. Create feature branch (`git checkout -b feature/amazing`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing`)
5. Create Pull Request

---

**Ultra-lightweight High-Performance MCP Server for AI Agents, built with Rust** 🦀
