use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    schemars, tool, tool_handler, tool_router, ServerHandler,
    transport::stdio, ServiceExt,
};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct BasemarkMcp {
    tool_router: ToolRouter<BasemarkMcp>,
    base_url: String,
    token: String,
}

// ---------------------------------------------------------------------------
// Tool parameter types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    #[schemars(description = "Search query string")]
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadDocParams {
    #[schemars(description = "Document ID")]
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateDocParams {
    #[schemars(description = "Document title")]
    pub title: String,
    #[schemars(description = "Document content in markdown")]
    pub content: Option<String>,
    #[schemars(description = "Collection ID to place the document in")]
    pub collection_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateDocParams {
    #[schemars(description = "Document ID")]
    pub id: String,
    #[schemars(description = "New title for the document")]
    pub title: Option<String>,
    #[schemars(description = "New content for the document in markdown")]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteDocParams {
    #[schemars(description = "Document ID to delete")]
    pub id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListDocsParams {
    #[schemars(description = "Optional collection ID to filter by")]
    pub collection_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateCollectionParams {
    #[schemars(description = "Name of the new collection")]
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ShareDocParams {
    #[schemars(description = "Document ID to share")]
    pub id: String,
    #[schemars(description = "Set the document as public (true) or private (false)")]
    pub is_public: Option<bool>,
    #[schemars(description = "Email address to invite a user to the document")]
    pub invite_email: Option<String>,
}

// ---------------------------------------------------------------------------
// Tool implementations
// ---------------------------------------------------------------------------

#[tool_router]
impl BasemarkMcp {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            tool_router: Self::tool_router(),
            base_url,
            token,
        }
    }

    fn client(&self) -> crate::client::BasemarkClient {
        crate::client::BasemarkClient::new(&self.base_url, &self.token)
    }

    #[tool(description = "Search documents by keyword")]
    async fn search_docs(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client().search(&params.query).await {
            Ok(results) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&results).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[tool(description = "Read a document's content as markdown")]
    async fn read_doc(
        &self,
        Parameters(params): Parameters<ReadDocParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client().read_document_markdown(&params.id).await {
            Ok(md) => Ok(CallToolResult::success(vec![Content::text(md)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[tool(description = "Create a new document")]
    async fn create_doc(
        &self,
        Parameters(params): Parameters<CreateDocParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let client = self.client();
        match client
            .create_document(
                &params.title,
                params.content.as_deref(),
                params.collection_id.as_deref(),
            )
            .await
        {
            Ok(doc) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&doc).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[tool(description = "Update an existing document's title and/or content")]
    async fn update_doc(
        &self,
        Parameters(params): Parameters<UpdateDocParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let client = self.client();
        match client
            .update_document(
                &params.id,
                params.title.as_deref(),
                params.content.as_deref(),
            )
            .await
        {
            Ok(doc) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&doc).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[tool(description = "Delete a document by ID")]
    async fn delete_doc(
        &self,
        Parameters(params): Parameters<DeleteDocParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client().delete_document(&params.id).await {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Deleted document {}",
                params.id
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[tool(description = "List documents, optionally filtered by collection ID")]
    async fn list_docs(
        &self,
        Parameters(params): Parameters<ListDocsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self
            .client()
            .list_documents(params.collection_id.as_deref())
            .await
        {
            Ok(docs) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&docs).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[tool(description = "List all collections")]
    async fn list_collections(&self) -> Result<CallToolResult, ErrorData> {
        match self.client().list_collections().await {
            Ok(collections) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&collections).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[tool(description = "Create a new collection")]
    async fn create_collection(
        &self,
        Parameters(params): Parameters<CreateCollectionParams>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.client().create_collection(&params.name).await {
            Ok(collection) => Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&collection).unwrap_or_default(),
            )])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[tool(description = "Share a document by setting visibility or inviting a user by email")]
    async fn share_doc(
        &self,
        Parameters(params): Parameters<ShareDocParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let client = self.client();
        let mut results: Vec<String> = Vec::new();

        if let Some(is_public) = params.is_public {
            match client.update_document_public(&params.id, is_public).await {
                Ok(_doc) => {
                    let visibility = if is_public { "public" } else { "private" };
                    results.push(format!("Document is now {visibility}"));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Error setting visibility: {e}"
                    ))]));
                }
            }
        }

        if let Some(ref email) = params.invite_email {
            match client.invite_to_document(&params.id, email).await {
                Ok(()) => {
                    results.push(format!("Invited {email}"));
                }
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Error inviting user: {e}"
                    ))]));
                }
            }
        }

        if results.is_empty() {
            Ok(CallToolResult::error(vec![Content::text(
                "No share action specified. Provide is_public and/or invite_email.",
            )]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(
                results.join("\n"),
            )]))
        }
    }
}

// ---------------------------------------------------------------------------
// ServerHandler impl
// ---------------------------------------------------------------------------

#[tool_handler]
impl ServerHandler for BasemarkMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "basemark",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Basemark wiki MCP server. Create, read, update, delete, and search documents."
                    .to_string(),
            )
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(config: &crate::config::Config) -> anyhow::Result<()> {
    let server = BasemarkMcp::new(config.url()?.to_string(), config.token()?.to_string());
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
