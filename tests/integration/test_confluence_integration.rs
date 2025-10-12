// T025: Confluence Tool Integration Tests
// Integration tests for Confluence tools using MockAtlassianServer

use mcp_atlassian::config::Config;
use mcp_atlassian::tools::confluence::{GetPageHandler, SearchHandler};
use mcp_atlassian::tools::ToolHandler;
use serde_json::json;
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
async fn test_confluence_search_integration() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock the search endpoint (v1 API)
    Mock::given(method("GET"))
        .and(path("/wiki/rest/api/search"))
        .and(query_param("cql", "type=page"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [
                {
                    "content": {
                        "id": "12345",
                        "type": "page",
                        "title": "Test Page"
                    }
                }
            ],
            "totalSize": 1
        })))
        .mount(&mock_server)
        .await;

    let handler = SearchHandler;
    let args = json!({
        "query": "type=page",
        "limit": 10
    });

    let result = handler.execute(args, &config).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response["success"], true);
    assert_eq!(response["total"], 1);
}

#[tokio::test]
async fn test_confluence_get_page_integration() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock the get page endpoint (v2 API)
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "12345",
            "title": "Test Page",
            "body": {
                "storage": {
                    "value": "<p>Test content</p>",
                    "representation": "storage"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let handler = GetPageHandler;
    let args = json!({"page_id": "12345"});

    let result = handler.execute(args, &config).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response["success"], true);
    assert_eq!(response["page"]["id"], "12345");
}

#[tokio::test]
async fn test_confluence_search_with_space_filter() {
    let mock_server = MockServer::start().await;
    let mut config = create_test_config(&mock_server).await;
    config.confluence_spaces_filter = vec!["SPACE1".to_string(), "SPACE2".to_string()];

    // Mock should receive CQL with space filter injected
    Mock::given(method("GET"))
        .and(path("/wiki/rest/api/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "results": [],
            "totalSize": 0
        })))
        .mount(&mock_server)
        .await;

    let handler = SearchHandler;
    let args = json!({
        "query": "type=page",
        "limit": 10
    });

    let result = handler.execute(args, &config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_confluence_auth_error_handling() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock 401 Unauthorized response
    Mock::given(method("GET"))
        .and(path("/wiki/rest/api/search"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "message": "Unauthorized",
            "statusCode": 401
        })))
        .mount(&mock_server)
        .await;

    let handler = SearchHandler;
    let args = json!({
        "query": "type=page",
        "limit": 10
    });

    let result = handler.execute(args, &config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_confluence_page_not_found() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock 404 Not Found response
    Mock::given(method("GET"))
        .and(path("/wiki/api/v2/pages/99999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "message": "Page not found",
            "statusCode": 404
        })))
        .mount(&mock_server)
        .await;

    let handler = GetPageHandler;
    let args = json!({"page_id": "99999"});

    let result = handler.execute(args, &config).await;
    assert!(result.is_err());
}
