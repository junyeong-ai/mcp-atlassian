# 🔧 MCP Atlassian

> AI Agent를 위한 초경량 Atlassian MCP 서버

Claude, ChatGPT 등 AI Agent가 Jira와 Confluence를 직접 제어할 수 있게 해주는 Model Context Protocol 서버.
Rust 기반 **4.4MB 바이너리**로 **응답 최적화**와 **빠른 실행 속도** 제공.

[![CI](https://github.com/junyeong-ai/mcp-atlassian/workflows/CI/badge.svg)](https://github.com/junyeong-ai/mcp-atlassian/actions)
[![codecov](https://codecov.io/gh/junyeong-ai/mcp-atlassian/branch/main/graph/badge.svg)](https://codecov.io/gh/junyeong-ai/mcp-atlassian)
[![Tools](https://img.shields.io/badge/MCP%20tools-14-blue?style=flat-square)](#🔧-14개-mcp-도구)
[![Rust](https://img.shields.io/badge/rust-1.90%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-2024--11--05%20%7C%202025--06--18-blue?style=flat-square)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)

**[한국어](README.md)** | [English](README.en.md)

---

## 📖 목차

- [왜 mcp-atlassian인가?](#🤖-왜-mcp-atlassian인가)
- [AI Agent 활용 예시](#💬-ai-agent-활용-예시)
- [3단계 시작하기](#🚀-3단계-시작하기)
- [환경변수 상세 가이드](#🎛️-환경변수-상세-가이드)
- [Jira Search 필드 최적화](#🔍-jira-search-필드-최적화)
- [기술 스택](#📊-기술-스택)
- [프로젝트 구조](#🏗️-프로젝트-구조)
- [개발](#🛠️-개발)
- [보안](#🔐-보안)
- [Troubleshooting](#❓-troubleshooting)
- [참고 자료](#📚-참고-자료)
- [라이센스](#📝-라이센스)
- [기여](#🤝-기여)

---

## 🤖 왜 mcp-atlassian인가?

AI Agent가 Atlassian을 사용할 때 **최적화된 경험**을 제공합니다:

### ✨ ADF로 리치 텍스트 포맷팅
- **Atlassian Document Format 지원**: 포맷이 적용된 설명과 댓글 작성
- **자동 변환**: 일반 텍스트가 자동으로 ADF로 변환
- **지원 포맷**: 제목, 코드 블록, 목록, 굵게, 기울임, 인라인 코드
- **ADF 지원 4개 도구**: `jira_create_issue`, `jira_update_issue`, `jira_add_comment`, `jira_update_comment`

### ⚡ AI Agent를 위한 응답 최적화
- **Jira 검색 필드 최적화**: 17개 핵심 필드만 반환 (description 제외)
  ```
  기본 필드: key, summary, status, priority, issuetype, assignee,
            reporter, creator, created, updated, duedate, resolutiondate,
            project, labels, components, parent, subtasks
  ```
- **커스터마이징 가능**: 환경변수로 필요한 필드만 요청
- **확장 필드 제외**: `-renderedFields`로 불필요한 데이터 제거

### 🚀 초경량 Self-Hosted
- **4.4MB 단일 바이너리**: 별도 런타임 불필요
- **즉시 실행**: 네이티브 바이너리로 빠른 시작
- **낮은 리소스**: Rust의 메모리 효율성

### 🔧 14개 MCP 도구
**Jira (8개)** - 4개 도구에 ADF 지원:
- `jira_search` - JQL 검색 (최적화된 필드)
- `jira_get_issue` - 이슈 상세 조회
- `jira_create_issue` ✨ - 이슈 생성 (ADF 지원)
- `jira_update_issue` ✨ - 이슈 수정 (ADF 지원)
- `jira_add_comment` ✨ - 댓글 추가 (ADF 지원)
- `jira_update_comment` ✨ - 댓글 수정 (ADF 지원)
- `jira_transition_issue` - 상태 전환
- `jira_get_transitions` - 가능한 전환 조회

**Confluence (6개)**:
- `confluence_search` - CQL 검색
- `confluence_get_page` - 페이지 조회
- `confluence_get_page_children` - 하위 페이지 목록
- `confluence_get_comments` - 댓글 조회
- `confluence_create_page` - 페이지 생성
- `confluence_update_page` - 페이지 수정

### 🔒 안전한 접근 제어
- **프로젝트/스페이스 필터링**: 특정 프로젝트/스페이스만 접근
- **환경변수 기반 인증**: API Token 안전 관리
- **HTTPS 전용**: 모든 통신 암호화

---

## 💬 AI Agent 활용 예시

### Claude Desktop에서

```
사용자: "이번 주 생성된 버그 목록 보여줘"
→ AI Agent가 jira_search 도구 자동 호출
→ 최적화된 17개 필드 응답

사용자: "PROJ-123에 코드 리뷰 완료 댓글 달아줘"
→ AI Agent가 jira_add_comment 도구 호출
→ 일반 텍스트가 자동으로 ADF로 변환

사용자: "댓글 10042를 '승인됨'으로 수정해줘"
→ AI Agent가 jira_update_comment 도구 호출
→ 포맷팅 지원과 함께 댓글 수정

사용자: "프로젝트 README 페이지 만들어줘"
→ AI Agent가 confluence_create_page 도구 호출
→ 스페이스 자동 확인 및 페이지 생성
```

---

## 🚀 3단계 시작하기

**전제 조건:** Rust 1.90+ 설치 필요 ([설치 가이드](https://www.rust-lang.org/tools/install))
**총 소요 시간:** ~10분 (Rust 이미 설치된 경우) ⚡

### 1️⃣ 빌드 (⏱️ ~5분)

```bash
# Rust 설치 (1.90+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 저장소 클론
git clone https://github.com/junyeong-ai/mcp-atlassian.git
cd mcp-atlassian

# Release 빌드
cargo build --release

# 바이너리 위치: target/release/mcp-atlassian (4.4MB)
```

### 2️⃣ 환경변수 설정 (⏱️ ~3분)

`.env` 파일 생성:

```env
# 필수 (3개)
ATLASSIAN_DOMAIN=yourcompany.atlassian.net
ATLASSIAN_EMAIL=you@example.com
ATLASSIAN_API_TOKEN=your_api_token_here

# 선택 - 필드 최적화 (기본: 17개 필드)
JIRA_SEARCH_DEFAULT_FIELDS=key,summary,status,assignee
JIRA_SEARCH_CUSTOM_FIELDS=customfield_10015,customfield_10016

# 선택 - 접근 제어
JIRA_PROJECTS_FILTER=PROJ1,PROJ2
CONFLUENCE_SPACES_FILTER=SPACE1,SPACE2

# 선택 - 성능
REQUEST_TIMEOUT_MS=30000
LOG_LEVEL=warn
```

**API Token 생성**: [Atlassian API Tokens](https://id.atlassian.com/manage-profile/security/api-tokens)

### 3️⃣ Claude Desktop 연결 (⏱️ ~2분)

`claude_desktop_config.json` 편집:

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

Claude Desktop 재시작 → 🎉 사용 준비 완료!

---

## 🎛️ 환경변수 상세 가이드

### 필드 최적화

#### `JIRA_SEARCH_DEFAULT_FIELDS`
전체 기본 필드를 완전히 대체합니다.

```env
# 최소 필드만 요청 (최대 최적화)
JIRA_SEARCH_DEFAULT_FIELDS=key,summary,status,assignee
```

#### `JIRA_SEARCH_CUSTOM_FIELDS`
기본 17개 필드에 추가 필드를 확장합니다.

```env
# 기본 17개 + 커스텀 필드 2개 = 총 19개
JIRA_SEARCH_CUSTOM_FIELDS=customfield_10015,customfield_10016
```

**필드 결정 우선순위**:

```
┌─────────────────────────────────┐
│ 1. API fields 파라미터           │  ← 최우선 (명시적 요청)
└─────────────────────────────────┘
           ↓ (없으면)
┌─────────────────────────────────┐
│ 2. JIRA_SEARCH_DEFAULT_FIELDS   │  ← 기본값 완전 대체
└─────────────────────────────────┘
           ↓ (없으면)
┌─────────────────────────────────┐
│ 3. 기본 17개 필드                │  ← 내장 기본값
│    + JIRA_SEARCH_CUSTOM_FIELDS  │     (선택적 확장)
└─────────────────────────────────┘
```

### 접근 제어

#### `JIRA_PROJECTS_FILTER`
특정 Jira 프로젝트만 접근 허용:

```env
JIRA_PROJECTS_FILTER=TEAM1,TEAM2,PROJ3
```

AI Agent가 JQL에 프로젝트를 명시하지 않으면 자동으로 필터 추가:
```
사용자 JQL: status = Open
실제 실행: project IN ("TEAM1","TEAM2","PROJ3") AND (status = Open)
```

#### `CONFLUENCE_SPACES_FILTER`
특정 Confluence 스페이스만 접근 허용:

```env
CONFLUENCE_SPACES_FILTER=TEAM,DOCS,KB
```

### 성능 튜닝

#### `REQUEST_TIMEOUT_MS`
API 요청 타임아웃 (기본: 30000ms):

```env
REQUEST_TIMEOUT_MS=10000  # 빠른 실패 (네트워크 빠른 환경)
REQUEST_TIMEOUT_MS=60000  # 느린 네트워크 대응
```

#### `LOG_LEVEL`
로그 상세도 (기본: warn):

```env
LOG_LEVEL=error  # 에러만
LOG_LEVEL=info   # 상세 로그
LOG_LEVEL=debug  # 디버깅용
```

---

## 🔍 Jira Search 필드 최적화

### 기본 17개 필드 (카테고리별)

| 카테고리 | 필드 | 설명 |
|---------|------|------|
| 🔑 **식별** | `key` | 이슈 고유 키 (예: PROJ-123) |
| 📝 **핵심 메타데이터** | `summary` | 이슈 제목 |
| | `status` | 현재 상태 (Open, In Progress 등) |
| | `priority` | 우선순위 (High, Medium, Low) |
| | `issuetype` | 이슈 유형 (Bug, Task, Story 등) |
| 👥 **담당자** | `assignee` | 할당된 담당자 |
| | `reporter` | 이슈 보고자 |
| | `creator` | 이슈 생성자 |
| 📅 **날짜** | `created` | 생성일 |
| | `updated` | 최종 수정일 |
| | `duedate` | 마감일 |
| | `resolutiondate` | 해결일 |
| 🏷️ **분류** | `project` | 프로젝트 정보 |
| | `labels` | 라벨 목록 |
| | `components` | 컴포넌트 목록 |
| 🌳 **계층** | `parent` | 상위 이슈 |
| | `subtasks` | 하위 이슈 목록 |

### 제외된 필드

- **`description`**: 대용량 텍스트 필드 (상세 조회 시에만 포함)
- **`id`**: `key`와 중복
- **`renderedFields`**: 렌더링된 HTML (expand=-renderedFields)

### 실전 활용

```bash
# 방법 1: API 호출 시 명시 (최우선)
{
  "jql": "project = KEY",
  "fields": ["key", "summary", "status"]
}

# 방법 2: 환경변수로 기본값 변경
JIRA_SEARCH_DEFAULT_FIELDS=key,summary,status,assignee

# 방법 3: 기본값에 추가
JIRA_SEARCH_CUSTOM_FIELDS=customfield_10015
```

---

## 📊 기술 스택

| 구성요소 | 기술 | 특징 |
|---------|------|------|
| **언어** | Rust 1.90+ (Edition 2024) | 메모리 안전, 고성능 |
| **런타임** | Tokio 1.47 | 비동기 I/O |
| **HTTP** | Reqwest 0.12 (rustls) | TLS 1.2+ 지원 |
| **직렬화** | Serde 1.0 | JSON 처리 |
| **로깅** | Tracing 0.1 | 구조화된 로깅 |
| **빌드 최적화** | LTO + Strip | 4.4MB 바이너리 |

### API 버전
- **Jira**: REST API v3
- **Confluence**: REST API v2 (검색만 v1)

### MCP 프로토콜
- JSON-RPC 2.0 over stdio
- 지원 버전: `2024-11-05`, `2025-06-18`

---

## 🏗️ 프로젝트 구조

```
src/
├── main.rs                   # 진입점
├── config/
│   └── mod.rs                # 환경변수 관리
├── mcp/
│   ├── server.rs             # MCP 프로토콜 서버
│   ├── handlers.rs           # 도구 등록
│   └── types.rs              # MCP 타입 정의
├── tools/
│   ├── handler.rs            # ToolHandler trait
│   ├── jira/
│   │   ├── mod.rs            # 8개 Jira 도구
│   │   ├── adf_utils.rs      # ADF 검증 & 변환
│   │   └── field_filtering.rs # 필드 최적화
│   └── confluence/
│       ├── mod.rs            # 6개 Confluence 도구
│       └── field_filtering.rs # API 최적화
└── utils/
    ├── http_utils.rs         # HTTP 클라이언트
    └── logging.rs            # 구조화된 로깅
```

---

## 🛠️ 개발

### 빌드

```bash
# 개발 빌드
cargo build

# Release 빌드 (최적화)
cargo build --release

# 직접 실행
cargo run

# 타입 체크만
cargo check
```

### 테스트

```bash
# 전체 테스트
cargo test

# 출력 포함
cargo test -- --nocapture

# 특정 테스트
cargo test test_config_validation
```

### 코드 품질

```bash
# 포맷팅
cargo fmt

# Lint
cargo clippy

# 전체 검사
cargo check && cargo clippy && cargo test
```

### Release 빌드 설정

```toml
[profile.release]
opt-level = 3       # 최대 최적화
lto = true          # Link-time optimization
codegen-units = 1   # 단일 코드 생성
strip = true        # 심볼 제거
```

**결과**: 4.4MB 최적화된 바이너리

---

## 🔐 보안

### 인증
- **방식**: HTTP Basic Auth
- **포맷**: `Authorization: Basic base64(email:api_token)`
- **전송**: HTTPS 전용

### 입력 검증
- 필수 파라미터 검증
- JQL/CQL은 Atlassian API로 전달
- JSON 스키마 검증

### 접근 제어
- 프로젝트/스페이스 필터링 (서버 측)
- 사용자 지정 필터 우선
- 우회 불가능

---

## ❓ Troubleshooting

### Claude Desktop에서 도구가 보이지 않아요

**해결 방법:**

1. **설정 파일 확인**
   ```bash
   # macOS
   cat ~/Library/Application\ Support/Claude/claude_desktop_config.json

   # Windows
   type %APPDATA%\Claude\claude_desktop_config.json
   ```

2. **Claude Desktop 완전 재시작**
   - 메뉴에서 "Quit" (단순 창 닫기 아님)
   - 다시 실행

3. **바이너리 경로 확인**
   ```bash
   # 바이너리가 존재하는지 확인
   ls -la target/release/mcp-atlassian

   # 실행 권한 확인
   chmod +x target/release/mcp-atlassian
   ```

### Atlassian API 연결 실패

**원인 1: API Token 오류**
- [Atlassian API Tokens](https://id.atlassian.com/manage-profile/security/api-tokens)에서 새로 생성
- `.env` 파일 또는 `claude_desktop_config.json`에 올바르게 설정했는지 확인

**원인 2: Domain 설정 오류**
```env
# 올바른 형식 (https:// 포함하지 않음)
ATLASSIAN_DOMAIN=yourcompany.atlassian.net

# 잘못된 형식
ATLASSIAN_DOMAIN=https://yourcompany.atlassian.net  ❌
```

**원인 3: 네트워크 타임아웃**
```env
# 타임아웃 증가 (기본: 30000ms)
REQUEST_TIMEOUT_MS=60000
```

### 특정 프로젝트에만 접근하고 싶어요

`JIRA_PROJECTS_FILTER` 사용:
```env
JIRA_PROJECTS_FILTER=PROJ1,PROJ2,PROJ3
```

자세한 내용은 [접근 제어](#접근-제어) 섹션 참조.

### 커스텀 필드를 추가하고 싶어요

`JIRA_SEARCH_CUSTOM_FIELDS` 사용:
```env
JIRA_SEARCH_CUSTOM_FIELDS=customfield_10015,customfield_10016
```

자세한 내용은 [필드 최적화](#필드-최적화) 섹션 참조.

### 로그 확인 방법

```env
# .env 파일에서 로그 레벨 변경
LOG_LEVEL=debug  # error, warn, info, debug, trace 중 선택
```

```bash
# macOS에서 서버 로그 확인 (Claude Desktop 로그)
tail -f ~/Library/Logs/Claude/mcp*.log

# 또는 직접 실행하여 로그 확인
./target/release/mcp-atlassian
```

---

## 📚 참고 자료

### Atlassian API
- [Jira REST API v3](https://developer.atlassian.com/cloud/jira/platform/rest/v3/)
- [Confluence REST API v2](https://developer.atlassian.com/cloud/confluence/rest/v2/)
- [Atlassian Document Format](https://developer.atlassian.com/cloud/jira/platform/apis/document/structure/)

### MCP
- [MCP 명세](https://modelcontextprotocol.io)
- [JSON-RPC 2.0](https://www.jsonrpc.org/specification)

### Rust
- [Tokio](https://docs.rs/tokio)
- [Reqwest](https://docs.rs/reqwest)
- [Serde JSON](https://docs.rs/serde_json)

---

## 📝 라이센스

MIT License - [LICENSE](LICENSE) 파일 참조

---

## 🤝 기여

Issue 및 Pull Request 환영합니다!

1. Fork
2. Feature 브랜치 생성 (`git checkout -b feature/amazing`)
3. 변경사항 커밋 (`git commit -m 'Add amazing feature'`)
4. 브랜치 푸시 (`git push origin feature/amazing`)
5. Pull Request 생성

---

**Rust로 만든 AI Agent를 위한 초경량 MCP 서버** 🦀
