use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::Serialize;

use crate::arxiv::api::ArxivClient;
use crate::cache::{downloader, Cache};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct SearchParams {
    #[schemars(description = "Search query string")]
    query: String,
    #[schemars(description = "Maximum number of results (default: 10)")]
    max_results: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct PaperIdParams {
    #[schemars(description = "arXiv paper ID (e.g. 2301.07041)")]
    paper_id: String,
}

#[derive(Debug, Serialize)]
struct DownloadResult {
    path: String,
    files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArxivServer {
    tool_router: ToolRouter<Self>,
    client: reqwest::Client,
}

impl ArxivServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            client: reqwest::Client::new(),
        }
    }
}

#[tool_router]
impl ArxivServer {
    #[tool(description = "Search arXiv papers by query string")]
    async fn search_papers(
        &self,
        Parameters(params): Parameters<SearchParams>,
    ) -> Result<String, String> {
        let arxiv = ArxivClient::new(self.client.clone());
        let max = params.max_results.unwrap_or(10);
        let papers = arxiv
            .search(&params.query, max)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&papers).map_err(|e| e.to_string())
    }

    #[tool(description = "Get detailed information about a specific arXiv paper")]
    async fn get_paper_info(
        &self,
        Parameters(params): Parameters<PaperIdParams>,
    ) -> Result<String, String> {
        let arxiv = ArxivClient::new(self.client.clone());
        let paper = arxiv
            .get_paper_info(&params.paper_id)
            .await
            .map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&paper).map_err(|e| e.to_string())
    }

    #[tool(description = "Download source files for an arXiv paper")]
    async fn download_sources(
        &self,
        Parameters(params): Parameters<PaperIdParams>,
    ) -> Result<String, String> {
        let (path, files) = downloader::download_and_extract(&self.client, &params.paper_id)
            .await
            .map_err(|e| e.to_string())?;
        let result = DownloadResult {
            path: path.to_string_lossy().to_string(),
            files,
        };
        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[tool(description = "List all locally cached arXiv papers")]
    async fn list_cached_papers(&self) -> Result<String, String> {
        let papers = Cache::list().map_err(|e| e.to_string())?;
        serde_json::to_string_pretty(&papers).map_err(|e| e.to_string())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ArxivServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("arXiv MCP server — search papers, fetch info, download sources")
    }
}
