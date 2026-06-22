use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use ws_core::error::WorkspaceError;
use ws_core::models::{
    AuthStatus, Comment, CreateEpicInput, CreateIssueInput, Issue, LinkIssuesInput, UpdateIssueInput,
};
use ws_core::providers::IssueProvider;

pub mod confluence;
pub use confluence::ConfluenceProvider;


pub struct JiraProvider {
    pub base_url: Option<String>,
    pub email: Option<String>,
    pub token: Option<String>,
    pub default_project: String,
}

impl JiraProvider {
    pub fn new(base_url: Option<String>, default_project: Option<String>) -> Self {
        let email = std::env::var("JIRA_EMAIL").ok();
        let token = std::env::var("JIRA_API_TOKEN").ok();
        Self {
            base_url,
            email,
            token,
            default_project: default_project.unwrap_or_else(|| "PLATFORM".to_string()),
        }
    }

    fn is_mock(&self) -> bool {
        self.base_url.is_none() || self.email.is_none() || self.token.is_none()
    }

    fn get_headers(&self) -> Result<HeaderMap, WorkspaceError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let (Some(email), Some(token)) = (&self.email, &self.token) {
            let auth_str = format!("{}:{}", email, token);
            let encoded = base64::encode(auth_str);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {}", encoded)).map_err(|e| {
                    WorkspaceError::provider("jira", format!("Invalid auth header: {}", e))
                })?,
            );
        }
        Ok(headers)
    }

    fn client(&self) -> Result<reqwest::Client, WorkspaceError> {
        reqwest::Client::builder().build().map_err(|e| {
            WorkspaceError::provider("jira", format!("Failed to build client: {}", e))
        })
    }
}

// Jira REST API DTOs
#[derive(Deserialize, Serialize)]
struct JiraUser {
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "emailAddress")]
    email_address: Option<String>,
}

#[derive(Deserialize)]
struct JiraIssueFields {
    summary: String,
    description: Option<serde_json::Value>, // In v3 description is a rich text document
    status: JiraStatus,
    issuetype: JiraIssueType,
    assignee: Option<JiraUser>,
    project: JiraProject,
}

#[derive(Deserialize)]
struct JiraProject {
    key: String,
}

#[derive(Deserialize)]
struct JiraStatus {
    name: String,
}

#[derive(Deserialize)]
struct JiraIssueType {
    name: String,
}

#[derive(Deserialize)]
struct JiraIssue {
    key: String,
    fields: JiraIssueFields,
}

#[async_trait]
impl IssueProvider for JiraProvider {
    fn kind(&self) -> &'static str {
        "jira"
    }

    async fn check_auth(&self) -> Result<AuthStatus, WorkspaceError> {
        if self.is_mock() {
            return Ok(AuthStatus {
                authenticated: true,
                username: Some("jira-mock-user".to_string()),
                details: Some("Jira provider running in mock mode (credentials missing)".to_string()),
            });
        }

        let base_url = self.base_url.as_ref().unwrap();
        let url = format!("{}/rest/api/3/myself", base_url.trim_end_matches('/'));
        let client = self.client()?;
        let response = client
            .get(&url)
            .headers(self.get_headers()?)
            .send()
            .await
            .map_err(|e| WorkspaceError::provider("jira", format!("Auth request failed: {}", e)))?;

        if response.status().is_success() {
            let user: JiraUser = response.json().await.map_err(|e| {
                WorkspaceError::provider("jira", format!("Failed to parse user details: {}", e))
            })?;
            Ok(AuthStatus {
                authenticated: true,
                username: Some(user.display_name),
                details: Some("Successfully authenticated with Jira Cloud".to_string()),
            })
        } else {
            Ok(AuthStatus {
                authenticated: false,
                username: None,
                details: Some(format!(
                    "Jira returned status code: {}. Check credentials.",
                    response.status()
                )),
            })
        }
    }

    async fn get_issue(&self, key: &str) -> Result<Issue, WorkspaceError> {
        if self.is_mock() {
            info!("Jira get_issue '{}' (Mock)", key);
            return Ok(Issue {
                key: key.to_string(),
                summary: format!("Mock Issue: Implement Slack notifications"),
                description: Some("Please build Slack notification templates and retries.".to_string()),
                status: "To Do".to_string(),
                issue_type: "Task".to_string(),
                assignee: Some("mock-assignee".to_string()),
                project_key: self.default_project.clone(),
            });
        }

        let base_url = self.base_url.as_ref().unwrap();
        let url = format!("{}/rest/api/3/issue/{}", base_url.trim_end_matches('/'), key);
        let client = self.client()?;
        let response = client
            .get(&url)
            .headers(self.get_headers()?)
            .send()
            .await
            .map_err(|e| WorkspaceError::provider("jira", format!("Failed to fetch issue: {}", e)))?;

        if !response.status().is_success() {
            return Err(WorkspaceError::provider(
                "jira",
                format!("Failed to get issue {}: status {}", key, response.status()),
            ));
        }

        let jira_issue: JiraIssue = response.json().await.map_err(|e| {
            WorkspaceError::provider("jira", format!("Failed to parse issue: {}", e))
        })?;

        // Try to parse description to string (very simplified, as v3 is Document format)
        let desc = jira_issue.fields.description.map(|v| v.to_string());

        Ok(Issue {
            key: jira_issue.key,
            summary: jira_issue.fields.summary,
            description: desc,
            status: jira_issue.fields.status.name,
            issue_type: jira_issue.fields.issuetype.name,
            assignee: jira_issue.fields.assignee.map(|u| u.display_name),
            project_key: jira_issue.fields.project.key,
        })
    }

    async fn create_epic(&self, input: CreateEpicInput) -> Result<Issue, WorkspaceError> {
        if self.is_mock() {
            info!("Jira create_epic '{}' (Mock)", input.name);
            return Ok(Issue {
                key: format!("{}-100", input.project),
                summary: input.summary,
                description: input.description,
                status: "To Do".to_string(),
                issue_type: "Epic".to_string(),
                assignee: None,
                project_key: input.project,
            });
        }

        let base_url = self.base_url.as_ref().unwrap();
        let url = format!("{}/rest/api/3/issue", base_url.trim_end_matches('/'));

        // Simplistic POST body
        let body = serde_json::json!({
            "fields": {
                "project": {
                    "key": input.project
                },
                "summary": input.summary,
                "issuetype": {
                    "name": "Epic"
                },
                "customfield_10011": input.name // standard epic name field
            }
        });

        let client = self.client()?;
        let response = client
            .post(&url)
            .headers(self.get_headers()?)
            .json(&body)
            .send()
            .await
            .map_err(|e| WorkspaceError::provider("jira", format!("Failed to create epic: {}", e)))?;

        if !response.status().is_success() {
            return Err(WorkspaceError::provider(
                "jira",
                format!("Failed to create epic: status {}", response.status()),
            ));
        }

        #[derive(Deserialize)]
        struct CreateResp {
            key: String,
        }
        let resp: CreateResp = response.json().await.map_err(|e| {
            WorkspaceError::provider("jira", format!("Failed to parse response: {}", e))
        })?;

        self.get_issue(&resp.key).await
    }

    async fn create_issue(&self, input: CreateIssueInput) -> Result<Issue, WorkspaceError> {
        if self.is_mock() {
            info!("Jira create_issue '{}' (Mock)", input.summary);
            return Ok(Issue {
                key: format!("{}-101", input.project),
                summary: input.summary,
                description: input.description,
                status: "To Do".to_string(),
                issue_type: input.issue_type,
                assignee: None,
                project_key: input.project,
            });
        }

        let base_url = self.base_url.as_ref().unwrap();
        let url = format!("{}/rest/api/3/issue", base_url.trim_end_matches('/'));

        let mut fields = serde_json::json!({
            "project": {
                "key": input.project
            },
            "summary": input.summary,
            "issuetype": {
                "name": input.issue_type
            }
        });

        if let Some(epic_key) = input.epic_key {
            // epic link field in Jira is usually parent
            fields["parent"] = serde_json::json!({ "key": epic_key });
        }

        let body = serde_json::json!({ "fields": fields });

        let client = self.client()?;
        let response = client
            .post(&url)
            .headers(self.get_headers()?)
            .json(&body)
            .send()
            .await
            .map_err(|e| WorkspaceError::provider("jira", format!("Failed to create issue: {}", e)))?;

        if !response.status().is_success() {
            return Err(WorkspaceError::provider(
                "jira",
                format!("Failed to create issue: status {}", response.status()),
            ));
        }

        #[derive(Deserialize)]
        struct CreateResp {
            key: String,
        }
        let resp: CreateResp = response.json().await.map_err(|e| {
            WorkspaceError::provider("jira", format!("Failed to parse response: {}", e))
        })?;

        self.get_issue(&resp.key).await
    }

    async fn update_issue(&self, key: &str, input: UpdateIssueInput) -> Result<Issue, WorkspaceError> {
        if self.is_mock() {
            info!("Jira update_issue '{}' (Mock)", key);
            return self.get_issue(key).await;
        }

        let base_url = self.base_url.as_ref().unwrap();
        let url = format!("{}/rest/api/3/issue/{}", base_url.trim_end_matches('/'), key);

        let mut update_fields = serde_json::json!({});
        if let Some(summary) = input.summary {
            update_fields["summary"] = serde_json::json!(summary);
        }

        let body = serde_json::json!({ "fields": update_fields });

        let client = self.client()?;
        let response = client
            .put(&url)
            .headers(self.get_headers()?)
            .json(&body)
            .send()
            .await
            .map_err(|e| WorkspaceError::provider("jira", format!("Failed to update issue: {}", e)))?;

        if !response.status().is_success() {
            return Err(WorkspaceError::provider(
                "jira",
                format!("Failed to update issue: status {}", response.status()),
            ));
        }

        self.get_issue(key).await
    }

    async fn link_issues(&self, input: LinkIssuesInput) -> Result<(), WorkspaceError> {
        if self.is_mock() {
            info!(
                "Jira link_issues '{}' -> '{}' ({}) (Mock)",
                input.inward_key, input.outward_key, input.link_type
            );
            return Ok(());
        }

        let base_url = self.base_url.as_ref().unwrap();
        let url = format!("{}/rest/api/3/issueLink", base_url.trim_end_matches('/'));

        let body = serde_json::json!({
            "type": {
                "name": input.link_type
            },
            "inwardIssue": {
                "key": input.inward_key
            },
            "outwardIssue": {
                "key": input.outward_key
            }
        });

        let client = self.client()?;
        let response = client
            .post(&url)
            .headers(self.get_headers()?)
            .json(&body)
            .send()
            .await
            .map_err(|e| WorkspaceError::provider("jira", format!("Failed to link issues: {}", e)))?;

        if !response.status().is_success() {
            return Err(WorkspaceError::provider(
                "jira",
                format!("Failed to link issues: status {}", response.status()),
            ));
        }

        Ok(())
    }

    async fn add_comment(&self, key: &str, body: &str) -> Result<(), WorkspaceError> {
        if self.is_mock() {
            info!("Jira add_comment on '{}': '{}' (Mock)", key, body);
            return Ok(());
        }

        let base_url = self.base_url.as_ref().unwrap();
        let url = format!("{}/rest/api/3/issue/{}/comment", base_url.trim_end_matches('/'), key);

        let json_body = serde_json::json!({
            "body": {
                "version": 1,
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [
                            {
                                "type": "text",
                                "text": body
                            }
                        ]
                    }
                ]
            }
        });

        let client = self.client()?;
        let response = client
            .post(&url)
            .headers(self.get_headers()?)
            .json(&json_body)
            .send()
            .await
            .map_err(|e| WorkspaceError::provider("jira", format!("Failed to add comment: {}", e)))?;

        if !response.status().is_success() {
            return Err(WorkspaceError::provider(
                "jira",
                format!("Failed to add comment: status {}", response.status()),
            ));
        }

        Ok(())
    }
}
