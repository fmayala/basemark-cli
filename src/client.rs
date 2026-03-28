use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// HTTP client for the Basemark wiki API.
pub struct BasemarkClient {
    http: Client,
    base_url: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub content: String,
    pub collection_id: Option<String>,
    pub is_public: Option<bool>,
    pub sort_order: Option<f64>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub snippet: String,
}

/// Wrapper for the JSON envelope the API returns.
#[derive(Deserialize)]
struct DataEnvelope<T> {
    data: T,
}

impl BasemarkClient {
    pub fn new(url: &str, token: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    /// Create a new document.
    pub async fn create_document(
        &self,
        title: &str,
        content: Option<&str>,
        collection_id: Option<&str>,
    ) -> Result<Document> {
        let url = format!("{}/api/documents.create", self.base_url);
        let mut body = serde_json::json!({ "title": title });
        if let Some(c) = content {
            body["text"] = serde_json::Value::String(c.to_string());
        }
        if let Some(cid) = collection_id {
            body["collectionId"] = serde_json::Value::String(cid.to_string());
        }

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .context("Failed to send create request")?
            .error_for_status()
            .context("API returned an error for create")?;

        let envelope: DataEnvelope<Document> = resp.json().await?;
        Ok(envelope.data)
    }

    /// Retrieve a document by ID (JSON).
    pub async fn read_document(&self, id: &str) -> Result<Document> {
        let url = format!("{}/api/documents.info", self.base_url);
        let body = serde_json::json!({ "id": id });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .context("Failed to send read request")?
            .error_for_status()
            .context("API returned an error for read")?;

        let envelope: DataEnvelope<Document> = resp.json().await?;
        Ok(envelope.data)
    }

    /// Retrieve a document's content as markdown text.
    pub async fn read_document_markdown(&self, id: &str) -> Result<String> {
        let url = format!(
            "{}/api/documents/{}/export",
            self.base_url, id
        );

        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "text/markdown")
            .send()
            .await
            .context("Failed to send markdown read request")?
            .error_for_status()
            .context("API returned an error for markdown read")?;

        let text = resp.text().await?;
        Ok(text)
    }

    /// Update a document's title and/or content.
    pub async fn update_document(
        &self,
        id: &str,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Document> {
        let url = format!("{}/api/documents.update", self.base_url);
        let mut body = serde_json::json!({ "id": id });
        if let Some(t) = title {
            body["title"] = serde_json::Value::String(t.to_string());
        }
        if let Some(c) = content {
            body["text"] = serde_json::Value::String(c.to_string());
        }

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .context("Failed to send update request")?
            .error_for_status()
            .context("API returned an error for update")?;

        let envelope: DataEnvelope<Document> = resp.json().await?;
        Ok(envelope.data)
    }

    /// Delete a document by ID.
    pub async fn delete_document(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/documents.delete", self.base_url);
        let body = serde_json::json!({ "id": id });

        self.http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .context("Failed to send delete request")?
            .error_for_status()
            .context("API returned an error for delete")?;

        Ok(())
    }

    /// List documents, optionally filtered by collection.
    pub async fn list_documents(
        &self,
        collection_id: Option<&str>,
    ) -> Result<Vec<Document>> {
        let url = format!("{}/api/documents.list", self.base_url);
        let mut body = serde_json::json!({});
        if let Some(cid) = collection_id {
            body["collectionId"] = serde_json::Value::String(cid.to_string());
        }

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .context("Failed to send list request")?
            .error_for_status()
            .context("API returned an error for list")?;

        let envelope: DataEnvelope<Vec<Document>> = resp.json().await?;
        Ok(envelope.data)
    }

    /// Full-text search across documents.
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let url = format!("{}/api/documents.search", self.base_url);
        let body = serde_json::json!({ "query": query });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .context("Failed to send search request")?
            .error_for_status()
            .context("API returned an error for search")?;

        let envelope: DataEnvelope<Vec<SearchResult>> = resp.json().await?;
        Ok(envelope.data)
    }
}
