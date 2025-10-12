// T026: Stateful Integration Tests
// Tests for multi-step workflows and stateful operations

use mcp_atlassian::config::Config;
use mcp_atlassian::tools::jira::{
    AddCommentHandler, CreateIssueHandler, GetIssueHandler, TransitionIssueHandler,
    UpdateIssueHandler,
};
use mcp_atlassian::tools::ToolHandler;
use serde_json::json;
use wiremock::matchers::{method, path};
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
async fn test_create_and_get_issue_workflow() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock create issue
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "10001",
            "key": "TEST-100",
            "self": "https://test.atlassian.net/rest/api/3/issue/10001"
        })))
        .mount(&mock_server)
        .await;

    // Mock get issue
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/TEST-100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "TEST-100",
            "fields": {
                "summary": "New Test Issue",
                "status": {"name": "Open"}
            }
        })))
        .mount(&mock_server)
        .await;

    // Step 1: Create issue
    let create_handler = CreateIssueHandler;
    let create_args = json!({
        "project_key": "TEST",
        "summary": "New Test Issue",
        "issue_type": "Task",
        "description": "Test description"
    });

    let create_result = create_handler.execute(create_args, &config).await;
    assert!(create_result.is_ok());
    let create_response = create_result.unwrap();
    assert_eq!(create_response["success"], true);

    // Step 2: Get the created issue
    let get_handler = GetIssueHandler;
    let get_args = json!({"issue_key": "TEST-100"});

    let get_result = get_handler.execute(get_args, &config).await;
    assert!(get_result.is_ok());
    let get_response = get_result.unwrap();
    assert_eq!(get_response["issue"]["key"], "TEST-100");
}

#[tokio::test]
async fn test_update_and_comment_workflow() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock update issue
    Mock::given(method("PUT"))
        .and(path("/rest/api/3/issue/TEST-100"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    // Mock add comment
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/TEST-100/comment"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "10050",
            "body": {
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": "Updated the issue"
                    }]
                }]
            }
        })))
        .mount(&mock_server)
        .await;

    // Step 1: Update issue
    let update_handler = UpdateIssueHandler;
    let update_args = json!({
        "issue_key": "TEST-100",
        "fields": {
            "summary": "Updated Summary"
        }
    });

    let update_result = update_handler.execute(update_args, &config).await;
    assert!(update_result.is_ok());

    // Step 2: Add comment
    let comment_handler = AddCommentHandler;
    let comment_args = json!({
        "issue_key": "TEST-100",
        "comment": "Updated the issue"
    });

    let comment_result = comment_handler.execute(comment_args, &config).await;
    assert!(comment_result.is_ok());
}

#[tokio::test]
async fn test_issue_transition_workflow() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock get transitions
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/TEST-100/transitions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "transitions": [
                {
                    "id": "11",
                    "name": "In Progress",
                    "to": {
                        "id": "3",
                        "name": "In Progress"
                    }
                },
                {
                    "id": "21",
                    "name": "Done",
                    "to": {
                        "id": "10001",
                        "name": "Done"
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    // Mock transition issue
    Mock::given(method("POST"))
        .and(path("/rest/api/3/issue/TEST-100/transitions"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    // Step 1: Get available transitions
    let get_transitions_handler =
        mcp_atlassian::tools::jira::GetTransitionsHandler;
    let get_transitions_args = json!({"issue_key": "TEST-100"});

    let get_transitions_result = get_transitions_handler
        .execute(get_transitions_args, &config)
        .await;
    assert!(get_transitions_result.is_ok());
    let transitions_response = get_transitions_result.unwrap();
    assert!(transitions_response["transitions"].is_array());

    // Step 2: Transition issue to "In Progress"
    let transition_handler = TransitionIssueHandler;
    let transition_args = json!({
        "issue_key": "TEST-100",
        "transition_id": "11"
    });

    let transition_result = transition_handler.execute(transition_args, &config).await;
    assert!(transition_result.is_ok());
}

#[tokio::test]
async fn test_multiple_operations_same_issue() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock get issue (will be called multiple times)
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/TEST-100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "TEST-100",
            "fields": {
                "summary": "Test Issue",
                "status": {"name": "Open"}
            }
        })))
        .expect(3)
        .mount(&mock_server)
        .await;

    let handler = GetIssueHandler;
    let args = json!({"issue_key": "TEST-100"});

    // Call the same endpoint multiple times
    for _ in 0..3 {
        let result = handler.execute(args.clone(), &config).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_requests() {
    let mock_server = MockServer::start().await;
    let config = create_test_config(&mock_server).await;

    // Mock different issue requests
    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/TEST-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "TEST-1",
            "fields": {"summary": "Issue 1"}
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/TEST-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "TEST-2",
            "fields": {"summary": "Issue 2"}
        })))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/rest/api/3/issue/TEST-3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "TEST-3",
            "fields": {"summary": "Issue 3"}
        })))
        .mount(&mock_server)
        .await;

    // Execute requests concurrently using join
    let (result1, result2, result3) = tokio::join!(
        GetIssueHandler.execute(json!({"issue_key": "TEST-1"}), &config),
        GetIssueHandler.execute(json!({"issue_key": "TEST-2"}), &config),
        GetIssueHandler.execute(json!({"issue_key": "TEST-3"}), &config)
    );

    // Verify all requests succeeded
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}
