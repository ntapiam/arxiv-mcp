pub mod downloader;

use std::path::PathBuf;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CachedPaper {
    pub paper_id: String,
    pub path: String,
}

pub struct Cache;

impl Cache {
    pub fn root() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("arxiv-mcp")
    }

    pub fn paper_dir(paper_id: &str) -> PathBuf {
        let sanitized = paper_id.replace('/', "_");
        Self::root().join(sanitized)
    }

    pub fn list() -> anyhow::Result<Vec<CachedPaper>> {
        let root = Self::root();
        if !root.exists() {
            return Ok(vec![]);
        }
        let mut papers = Vec::new();
        for entry in std::fs::read_dir(&root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let paper_id = entry.file_name().to_string_lossy().to_string();
                let path = entry.path().to_string_lossy().to_string();
                papers.push(CachedPaper { paper_id, path });
            }
        }
        Ok(papers)
    }
}
