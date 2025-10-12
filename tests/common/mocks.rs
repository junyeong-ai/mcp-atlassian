use serde_json::Value;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub struct MockAtlassianServer {
    pub server: MockServer,
}

impl MockAtlassianServer {
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    pub fn uri(&self) -> String {
        self.server.uri()
    }

    pub async fn mock_jira_search(&self, jql: &str, response_body: Value, status: u16) {
        Mock::given(method("GET"))
            .and(path("/rest/api/3/search"))
            .and(query_param("jql", jql))
            .respond_with(ResponseTemplate::new(status).set_body_json(response_body))
            .expect(1)
            .mount(&self.server)
            .await;
    }

    pub async fn mock_jira_get_issue(&self, issue_key: &str, response_body: Value, status: u16) {
        Mock::given(method("GET"))
            .and(path(format!("/rest/api/3/issue/{}", issue_key)))
            .respond_with(ResponseTemplate::new(status).set_body_json(response_body))
            .expect(1)
            .mount(&self.server)
            .await;
    }

    pub async fn mock_confluence_search(&self, cql: &str, response_body: Value, status: u16) {
        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/search"))
            .and(query_param("cql", cql))
            .respond_with(ResponseTemplate::new(status).set_body_json(response_body))
            .expect(1)
            .mount(&self.server)
            .await;
    }

    pub async fn mock_confluence_get_page(&self, page_id: &str, response_body: Value, status: u16) {
        Mock::given(method("GET"))
            .and(path(format!("/wiki/api/v2/pages/{}", page_id)))
            .respond_with(ResponseTemplate::new(status).set_body_json(response_body))
            .expect(1)
            .mount(&self.server)
            .await;
    }

    pub async fn mock_auth_error(&self) {
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "errorMessages": ["Unauthorized"],
                "errors": {}
            })))
            .mount(&self.server)
            .await;
    }

    pub async fn mock_rate_limit_error(&self) {
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "60")
                    .set_body_json(serde_json::json!({
                        "errorMessages": ["Rate limit exceeded"],
                        "errors": {}
                    })),
            )
            .mount(&self.server)
            .await;
    }
}

pub async fn setup_mock_server() -> MockServer {
    MockServer::start().await
}
