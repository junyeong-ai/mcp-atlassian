# MCP Atlassian Server

> Rust-based MCP server for Jira and Confluence integration

A Model Context Protocol (MCP) server that connects AI assistants to Atlassian Cloud, providing 13 tools for Jira and Confluence operations.

[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05%20%7C%202025--06--18-blue?style=flat-square)](https://modelcontextprotocol.io)
[![Tools](https://img.shields.io/badge/tools-13-blue?style=flat-square)](#features)

---

## What is This?

**mcp-atlassian** implements the Model Context Protocol to enable AI assistants like Claude to interact with Atlassian Cloud services (Jira and Confluence) through natural language.

**Technical Stack:**
- Written in Rust 1.90+ (Edition 2024)
- Tokio async runtime
- MCP protocol support (versions 2024-11-05 and 2025-06-18)
- 13 implemented tools (7 Jira + 6 Confluence)
- Single binary deployment (4.4MB)

---

## Features

### Jira Tools (7)
1. **jira_get_issue** - Get detailed issue information
2. **jira_search** - Search issues using JQL
3. **jira_create_issue** - Create new issues
4. **jira_update_issue** - Update existing issues
5. **jira_add_comment** - Add comments to issues
6. **jira_transition_issue** - Change issue status
7. **jira_get_transitions** - Get available status transitions

### Confluence Tools (6)
1. **confluence_search** - Search pages using CQL (v1 API)
2. **confluence_get_page** - Get page content (v2 API)
3. **confluence_get_page_children** - List child pages (v2 API)
4. **confluence_get_comments** - Get page comments (v2 API)
5. **confluence_create_page** - Create new pages (v2 API)
6. **confluence_update_page** - Update existing pages (v2 API)

### Optimizations
- **Field Filtering**: Requests only 13 essential fields by default instead of all available fields
- **Project/Space Filtering**: Optional scoping to specific projects or spaces
- **Custom Field Support**: Configurable additional fields via environment variables

---

## Quick Start

### Prerequisites
- Rust 1.90 or later ([Install Rust](https://rustup.rs/))
- Atlassian Cloud account
- API Token ([Generate here](https://id.atlassian.com/manage-profile/security/api-tokens))

### Installation

```bash
# Clone repository
git clone https://github.com/junyeong-ai/mcp-atlassian.git
cd mcp-atlassian

# Build release binary
cargo build --release

# Binary location: target/release/mcp-atlassian
```

### Configuration

Create a `.env` file:

```env
# Required
ATLASSIAN_DOMAIN=yourcompany.atlassian.net
ATLASSIAN_EMAIL=you@example.com
ATLASSIAN_API_TOKEN=your_api_token_here

# Optional - Performance
MAX_CONNECTIONS=100
REQUEST_TIMEOUT_MS=30000
LOG_LEVEL=warn

# Optional - Field Filtering
JIRA_CUSTOM_FIELDS=customfield_10001,customfield_10002
CONFLUENCE_CUSTOM_INCLUDES=ancestors,history

# Optional - Scoped Access
JIRA_PROJECTS_FILTER=PROJ1,PROJ2
CONFLUENCE_SPACES_FILTER=SPACE1,SPACE2
```

### Claude Desktop Setup

Add to `claude_desktop_config.json`:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "atlassian": {
      "command": "/path/to/mcp-atlassian/target/release/mcp-atlassian",
      "env": {
        "ATLASSIAN_DOMAIN": "yourcompany.atlassian.net",
        "ATLASSIAN_EMAIL": "you@example.com",
        "ATLASSIAN_API_TOKEN": "your_api_token_here"
      }
    }
  }
}
```

Restart Claude Desktop to load the server.

---

## Project Structure

```
src/
├── main.rs                     - Application entry point
├── config/
│   └── mod.rs                  - Environment configuration
├── mcp/
│   ├── server.rs               - MCP protocol server
│   ├── handlers.rs             - Tool registration
│   └── types.rs                - Protocol types
├── tools/
│   ├── handler.rs              - Tool trait definition
│   ├── jira/
│   │   ├── mod.rs              - 7 Jira tools
│   │   └── field_filtering.rs  - Field optimization
│   └── confluence/
│       ├── mod.rs              - 6 Confluence tools
│       └── field_filtering.rs  - API optimization
└── utils/
    ├── http_utils.rs           - HTTP client
    └── logging.rs              - Structured logging
```

---

## Configuration Reference

### Required Variables
- `ATLASSIAN_DOMAIN` - Your Atlassian domain (e.g., `company.atlassian.net`)
- `ATLASSIAN_EMAIL` - Your account email
- `ATLASSIAN_API_TOKEN` - API token from Atlassian

### Optional Variables
- `MAX_CONNECTIONS` - HTTP pool size (default: 100, range: 1-1000)
- `REQUEST_TIMEOUT_MS` - Request timeout (default: 30000, range: 100-60000)
- `LOG_LEVEL` - Logging level (error/warn/info/debug/trace, default: warn)
- `JIRA_CUSTOM_FIELDS` - Additional Jira fields (comma-separated)
- `CONFLUENCE_CUSTOM_INCLUDES` - Additional Confluence includes
- `JIRA_PROJECTS_FILTER` - Limit to specific Jira projects
- `CONFLUENCE_SPACES_FILTER` - Limit to specific Confluence spaces

---

## Development

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run
```

### Testing

```bash
# Run tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for issues
cargo check
```

---

## Technical Details

### MCP Protocol
- Implements JSON-RPC 2.0 over stdio
- Supports protocol versions: 2024-11-05, 2025-06-18
- Methods: initialize, tools/list, tools/call, prompts/list, resources/list

### API Versions
- Jira: REST API v3
- Confluence: REST API v2 (v1 for search only)

### Dependencies
- tokio 1.47 - Async runtime
- reqwest 0.12 - HTTP client
- serde 1.0 - Serialization
- tracing 0.1 - Structured logging

### Build Configuration
- Optimization level: 3
- Link-time optimization: enabled
- Codegen units: 1
- Strip symbols: enabled
- Binary size: 4.4MB

---

## Security

- Authentication: HTTP Basic Auth with API token
- Transport: HTTPS only
- Credentials: Environment variables only
- Project/Space filtering for access control

---

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

## Resources

- [MCP Protocol](https://modelcontextprotocol.io)
- [Jira REST API v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Confluence REST API v2](https://developer.atlassian.com/cloud/confluence/rest/v2/)
- [Atlassian API Tokens](https://id.atlassian.com/manage-profile/security/api-tokens)

---

**Built with Rust 🦀 | MCP Protocol Implementation**
