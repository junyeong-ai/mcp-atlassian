// T024: Jira Tool Integration Tests
// Integration tests for Jira tools using MockAtlassianServer

use mcp_atlassian::config::Config;
use mcp_atlassian::tools::jira::{
    CreateIssueHandler, GetIssueHandler, SearchHandler, UpdateIssueHandler,
};
use mcp_atlassian::tools::ToolHandler;
use serde_json::json;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn create_test_config(mock_server: &MockServer) -> Config {
    Config {
        atlassian_domain: mock_server.uri(),
        atlassian_email: "test@example.com".to_string(),
        atlassian_api_token: "test-token".to_string(),
        max_connections: 10,
        request_timeout_ms: 30000,
        jira_projects_filter: vec![],
        confluence_spaces_filter: vec![],
        jira_search_default_fields: None,
        jira_search_custom_fields: vec![],
    }
}

#[tokio::test]
async fn test_jira_search_integration() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock the search endpoint
    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .and(query_param("jql", "status = Open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "issues": [
                {
                    "key": "TEST-1",
                    "fields": {
                        "summary": "Test Issue"
                    }
                }
            ],
            "total": 1
        })))
        .mount(&mock_server)
        .await;

    let handler = SearchHandler;
    let args = json!({
        "jql": "status = Open",
        "limit": 20
    });

    let result = handler.execute(args, &config).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response["success"], true);
    assert_eq!(response["total"], 1);
}

#[tokio::test]
async fn test_jira_get_issue_integration() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock the get issue endpoint
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/TEST-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "TEST-123",
            "fields": {
                "summary": "Test Issue",
                "status": {
                    "name": "Open"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let handler = GetIssueHandler;
    let args = json!({"issue_key": "TEST-123"});

    let result = handler.execute(args, &config).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response["success"], true);
    assert_eq!(response["issue"]["key"], "TEST-123");
}

#[tokio::test]
async fn test_jira_search_with_project_filter() {
    let mock_server = MockServer::start().await;
    let mut config = create_test_config(&mock_server).await;
    config.jira_projects_filter = vec!["PROJ1".to_string(), "PROJ2".to_string()];

    // Mock should receive JQL with project filter injected
    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "issues": [],
            "total": 0
        })))
        .mount(&mock_server)
        .await;

    let handler = SearchHandler;
    let args = json!({
        "jql": "status = Open",
        "limit": 20
    });

    let result = handler.execute(args, &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jira_search_with_custom_fields() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "issues": [],
            "total": 0
        })))
        .mount(&mock_server)
        .await;

    let handler = SearchHandler;
    let args = json!({
        "jql": "status = Open",
        "fields": ["key", "summary", "status"]
    });

    let result = handler.execute(args, &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jira_auth_error_handling() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock 401 Unauthorized response
    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "errorMessages": ["Unauthorized"],
            "errors": {}
        })))
        .mount(&mock_server)
        .await;

    let handler = SearchHandler;
    let args = json!({
        "jql": "status = Open",
        "limit": 20
    });

    let result = handler.execute(args, &config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jira_rate_limit_handling() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock 429 Rate Limit response
    Mock::given(method("GET"))
        .and(path("/rest/api/3/search/jql"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "60")
                .set_body_json(json!({
                    "errorMessages": ["Rate limit exceeded"],
                    "errors": {}
                })),
        )
        .mount(&mock_server)
        .await;

    let handler = SearchHandler;
    let args = json!({
        "jql": "status = Open",
        "limit": 20
    });

    let result = handler.execute(args, &config).await;
    assert!(result.is_err());
}
