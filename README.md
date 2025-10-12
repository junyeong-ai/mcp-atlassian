# MCP Atlassian Server

> Rust-based MCP server for Jira and Confluence integration

Model Context Protocol (MCP) server that connects AI assistants to Atlassian Cloud, providing 13 tools for Jira and Confluence operations.

[![CI](https://github.com/junyeong-ai/mcp-atlassian/workflows/CI/badge.svg)](https://github.com/junyeong-ai/mcp-atlassian/actions)
[![Lint](https://github.com/junyeong-ai/mcp-atlassian/workflows/Lint/badge.svg)](https://github.com/junyeong-ai/mcp-atlassian/actions)
[![codecov](https://codecov.io/gh/junyeong-ai/mcp-atlassian/branch/main/graph/badge.svg)](https://codecov.io/gh/junyeong-ai/mcp-atlassian)
[![Rust](https://img.shields.io/badge/rust-1.90%2B%20(2024%20edition)-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05%20%7C%202025--06--18-blue?style=flat-square)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-blue?style=flat-square)](https://github.com/junyeong-ai/mcp-atlassian/releases)
[![Tools](https://img.shields.io/badge/MCP%20tools-13-blue?style=flat-square)](#features)
[![Tests](https://img.shields.io/badge/tests-135%20passing-success?style=flat-square)](#testing)

---

## Features

### Jira Tools (7)
1. **jira_get_issue** - Get issue by key
2. **jira_search** - Search issues using JQL (optimized with 17 default fields)
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

**Jira Search Field Filtering**:
- Default 17 fields: key, summary, status, priority, issuetype, assignee, reporter, creator, created, updated, duedate, resolutiondate, project, labels, components, parent, subtasks
- Configurable via environment variables or API parameters
- Excludes heavy fields (description) for token efficiency

**Project/Space Filtering**:
- Optional scoping to specific projects or spaces
- Auto-injected into queries when configured

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

# Binary location: target/release/mcp-atlassian (4.4MB)
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

# Optional - Jira Search Field Configuration
JIRA_SEARCH_DEFAULT_FIELDS=key,summary,status,assignee
JIRA_SEARCH_CUSTOM_FIELDS=customfield_10015,customfield_10016

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

## Configuration Reference

### Required Variables
- `ATLASSIAN_DOMAIN` - Atlassian domain (e.g., `company.atlassian.net`)
- `ATLASSIAN_EMAIL` - Account email
- `ATLASSIAN_API_TOKEN` - API token from Atlassian

### Optional - Performance
- `MAX_CONNECTIONS` - HTTP pool size (default: 100, range: 1-1000)
- `REQUEST_TIMEOUT_MS` - Request timeout in ms (default: 30000, range: 100-60000)
- `LOG_LEVEL` - Logging level: error, warn, info, debug, trace (default: warn)

### Optional - Jira Search Field Configuration
- `JIRA_SEARCH_DEFAULT_FIELDS` - Override default fields completely (comma-separated)
- `JIRA_SEARCH_CUSTOM_FIELDS` - Add custom fields to defaults (comma-separated, e.g., `customfield_10015`)

**Field Resolution Priority**:
1. API call `fields` parameter (highest)
2. `JIRA_SEARCH_DEFAULT_FIELDS` environment variable
3. Built-in defaults (17 fields) + `JIRA_SEARCH_CUSTOM_FIELDS`
4. Built-in defaults only (fallback)

**Built-in Default Fields (17)**:
- Identification: key
- Core: summary, status, priority, issuetype
- People: assignee, reporter, creator
- Dates: created, updated, duedate, resolutiondate
- Classification: project, labels, components
- Hierarchy: parent, subtasks

### Optional - Scoped Access
- `JIRA_PROJECTS_FILTER` - Limit to specific Jira projects (comma-separated)
- `CONFLUENCE_SPACES_FILTER` - Limit to specific Confluence spaces (comma-separated)

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

## Development

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run

# Check without building
cargo check
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_config_validation
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

### Stack
- **Language**: Rust 1.90+ (Edition 2024)
- **Runtime**: Tokio 1.47 (async)
- **HTTP**: Reqwest 0.12 (rustls-tls)
- **Serialization**: Serde 1.0
- **Logging**: Tracing 0.1

### MCP Protocol
- JSON-RPC 2.0 over stdio
- Supported versions: 2024-11-05, 2025-06-18
- Methods: initialize, initialized, tools/list, tools/call, prompts/list, resources/list

### API Versions
- **Jira**: REST API v3
- **Confluence**: REST API v2 (v1 for search only, as v2 has no search endpoint)

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

## Security

- **Authentication**: HTTP Basic Auth with API token
- **Transport**: HTTPS only
- **Credentials**: Environment variables or .env file
- **Access Control**: Optional project/space filtering

---

## Resources

- [MCP Protocol](https://modelcontextprotocol.io)
- [Jira REST API v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Confluence REST API v2](https://developer.atlassian.com/cloud/confluence/rest/v2/)
- [Atlassian API Tokens](https://id.atlassian.com/manage-profile/security/api-tokens)

---

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

**Built with Rust 🦀**
