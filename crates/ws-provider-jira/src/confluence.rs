use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use ws_core::error::WorkspaceError;
use ws_core::models::AuthStatus;
use ws_core::providers::DocProvider;

pub struct ConfluenceProvider {
    pub base_url: Option<String>,
    pub email: Option<String>,
    pub token: Option<String>,
    pub default_space: String,
}

impl ConfluenceProvider {
    pub fn new(base_url: Option<String>, default_space: Option<String>) -> Self {
        let email = std::env::var("CONFLUENCE_EMAIL").ok();
        let token = std::env::var("CONFLUENCE_API_TOKEN").ok();
        Self {
            base_url,
            email,
            token,
            default_space: default_space.unwrap_or_default(),
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
                    WorkspaceError::provider("confluence", format!("Invalid auth header: {}", e))
                })?,
            );
        }
        Ok(headers)
    }

    fn client(&self) -> Result<reqwest::Client, WorkspaceError> {
        reqwest::Client::builder().build().map_err(|e| {
            WorkspaceError::provider("confluence", format!("Failed to build client: {}", e))
        })
    }
}

#[derive(Deserialize)]
struct ConfluenceSpace {
    key: String,
}

#[derive(Deserialize)]
struct ConfluenceContent {
    id: String,
    title: String,
    body: Option<ConfluenceBody>,
}

#[derive(Deserialize)]
struct ConfluenceBody {
    storage: Option<ConfluenceStorageValue>,
}

#[derive(Deserialize)]
struct ConfluenceStorageValue {
    value: String,
}

#[derive(Deserialize)]
struct ConfluenceSearchResults {
    results: Vec<ConfluenceContent>,
}

#[async_trait]
impl DocProvider for ConfluenceProvider {
    fn kind(&self) -> &'static str {
        "confluence"
    }

    async fn check_auth(&self) -> Result<AuthStatus, WorkspaceError> {
        if self.is_mock() {
            return Ok(AuthStatus {
                authenticated: true,
                username: Some("confluence-mock-user".to_string()),
                details: Some(
                    "Confluence provider running in mock mode (credentials missing)".to_string(),
                ),
            });
        }

        let base_url = self.base_url.as_ref().unwrap();
        // Simple auth check via current user info
        let url = format!(
            "{}/wiki/rest/api/user/current",
            base_url.trim_end_matches('/')
        );
        let client = self.client()?;
        let response = client
            .get(&url)
            .headers(self.get_headers()?)
            .send()
            .await
            .map_err(|e| {
                WorkspaceError::provider("confluence", format!("Auth check failed: {}", e))
            })?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct UserInfo {
                #[serde(rename = "displayName")]
                display_name: String,
            }
            let info: UserInfo = response.json().await.map_err(|e| {
                WorkspaceError::provider(
                    "confluence",
                    format!("Failed to parse user details: {}", e),
                )
            })?;
            Ok(AuthStatus {
                authenticated: true,
                username: Some(info.display_name),
                details: Some("Successfully authenticated with Confluence Cloud".to_string()),
            })
        } else {
            Ok(AuthStatus {
                authenticated: false,
                username: None,
                details: Some(format!(
                    "Confluence returned status code: {}. Check credentials.",
                    response.status()
                )),
            })
        }
    }

    async fn get_page(&self, space: &str, title: &str) -> Result<String, WorkspaceError> {
        if self.is_mock() {
            info!(
                "Confluence get_page in space '{}', title '{}' (Mock)",
                space, title
            );
            return Ok(format!(
                "# Mock Confluence Page\n\nSpace: {}\nTitle: {}\n\nThis is placeholder confluence page content.",
                space, title
            ));
        }

        let base_url = self.base_url.as_ref().unwrap();
        let url = format!(
            "{}/wiki/rest/api/content?spaceKey={}&title={}&expand=body.storage",
            base_url.trim_end_matches('/'),
            space,
            urlencoding::encode(title)
        );

        let client = self.client()?;
        let response = client
            .get(&url)
            .headers(self.get_headers()?)
            .send()
            .await
            .map_err(|e| {
                WorkspaceError::provider("confluence", format!("Failed to get page: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(WorkspaceError::provider(
                "confluence",
                format!(
                    "Failed to retrieve page '{}': status {}",
                    title,
                    response.status()
                ),
            ));
        }

        let results: ConfluenceSearchResults = response.json().await.map_err(|e| {
            WorkspaceError::provider("confluence", format!("Failed to parse page results: {}", e))
        })?;

        let first = results.results.first().ok_or_else(|| {
            WorkspaceError::NotFound(format!(
                "Page '{}' not found in Confluence space '{}'",
                title, space
            ))
        })?;

        let body = first
            .body
            .as_ref()
            .and_then(|b| b.storage.as_ref())
            .map(|s| s.value.clone())
            .unwrap_or_default();

        Ok(body)
    }

    async fn create_page(
        &self,
        space: &str,
        title: &str,
        body: &str,
    ) -> Result<String, WorkspaceError> {
        if self.is_mock() {
            info!(
                "Confluence create_page in space '{}', title '{}' (Mock)",
                space, title
            );
            return Ok(format!(
                "mock-page-id-{}",
                title.replace(' ', "-").to_lowercase()
            ));
        }

        let base_url = self.base_url.as_ref().unwrap();
        let url = format!("{}/wiki/rest/api/content", base_url.trim_end_matches('/'));

        let post_body = serde_json::json!({
            "type": "page",
            "title": title,
            "space": {
                "key": space
            },
            "body": {
                "storage": {
                    "value": body,
                    "representation": "storage"
                }
            }
        });

        let client = self.client()?;
        let response = client
            .post(&url)
            .headers(self.get_headers()?)
            .json(&post_body)
            .send()
            .await
            .map_err(|e| {
                WorkspaceError::provider("confluence", format!("Failed to create page: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(WorkspaceError::provider(
                "confluence",
                format!("Failed to create page: status {}", response.status()),
            ));
        }

        #[derive(Deserialize)]
        struct CreatePageResp {
            id: String,
        }
        let resp: CreatePageResp = response.json().await.map_err(|e| {
            WorkspaceError::provider("confluence", format!("Failed to parse response: {}", e))
        })?;

        Ok(resp.id)
    }

    async fn update_page(
        &self,
        page_id: &str,
        title: &str,
        body: &str,
    ) -> Result<(), WorkspaceError> {
        if self.is_mock() {
            info!(
                "Confluence update_page ID '{}', title '{}' (Mock)",
                page_id, title
            );
            return Ok(());
        }

        let base_url = self.base_url.as_ref().unwrap();

        // Confluence updates require a version increment. We must GET the current version first.
        let get_url = format!(
            "{}/wiki/rest/api/content/{}?expand=version,space",
            base_url.trim_end_matches('/'),
            page_id
        );

        let client = self.client()?;
        let get_response = client
            .get(&get_url)
            .headers(self.get_headers()?)
            .send()
            .await
            .map_err(|e| {
                WorkspaceError::provider(
                    "confluence",
                    format!("Failed to fetch version before update: {}", e),
                )
            })?;

        if !get_response.status().is_success() {
            return Err(WorkspaceError::provider(
                "confluence",
                format!(
                    "Failed to get version for page {}: status {}",
                    page_id,
                    get_response.status()
                ),
            ));
        }

        #[derive(Deserialize)]
        struct PageVersion {
            number: usize,
        }
        #[derive(Deserialize)]
        struct PageMeta {
            version: PageVersion,
            space: ConfluenceSpace,
        }

        let meta: PageMeta = get_response.json().await.map_err(|e| {
            WorkspaceError::provider("confluence", format!("Failed to parse page version: {}", e))
        })?;

        let put_url = format!(
            "{}/wiki/rest/api/content/{}",
            base_url.trim_end_matches('/'),
            page_id
        );
        let put_body = serde_json::json!({
            "id": page_id,
            "type": "page",
            "title": title,
            "space": {
                "key": meta.space.key
            },
            "body": {
                "storage": {
                    "value": body,
                    "representation": "storage"
                }
            },
            "version": {
                "number": meta.version.number + 1
            }
        });

        let put_response = client
            .put(&put_url)
            .headers(self.get_headers()?)
            .json(&put_body)
            .send()
            .await
            .map_err(|e| {
                WorkspaceError::provider("confluence", format!("Failed to update page: {}", e))
            })?;

        if !put_response.status().is_success() {
            return Err(WorkspaceError::provider(
                "confluence",
                format!(
                    "Failed to save updated page: status {}",
                    put_response.status()
                ),
            ));
        }

        Ok(())
    }
}
