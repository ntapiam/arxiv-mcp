pub mod downloader;

use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use serde::Serialize;

const EXPIRY_DAYS: u64 = 30;
const EXPIRY_DURATION: Duration = Duration::from_secs(EXPIRY_DAYS * 24 * 60 * 60);

#[derive(Debug, Serialize)]
pub struct CachedPaper {
    pub paper_id: String,
    pub path: String,
    pub cached_at: String,
    pub outdated: bool,
}

#[derive(Debug, Serialize)]
pub struct PurgeResult {
    pub purged: Vec<String>,
    pub count: usize,
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
        let now = SystemTime::now();
        let mut papers = Vec::new();
        for entry in std::fs::read_dir(&root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let paper_id = entry.file_name().to_string_lossy().to_string();
                let path = entry.path().to_string_lossy().to_string();
                let (cached_at, outdated) = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(|mtime| {
                        let age = now.duration_since(mtime).unwrap_or(Duration::ZERO);
                        let secs = mtime
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or(Duration::ZERO)
                            .as_secs();
                        let cached_at = format_unix_timestamp(secs);
                        (cached_at, age > EXPIRY_DURATION)
                    })
                    .unwrap_or_else(|| ("unknown".to_string(), false));
                papers.push(CachedPaper { paper_id, path, cached_at, outdated });
            }
        }
        Ok(papers)
    }

    pub fn purge_outdated() -> anyhow::Result<PurgeResult> {
        let papers = Self::list()?;
        let mut purged = Vec::new();
        for paper in papers.into_iter().filter(|p| p.outdated) {
            std::fs::remove_dir_all(&paper.path)?;
            purged.push(paper.paper_id);
        }
        let count = purged.len();
        Ok(PurgeResult { purged, count })
    }
}

fn format_unix_timestamp(secs: u64) -> String {
    // Manual ISO 8601 UTC formatting without external deps
    let mut remaining = secs;
    let s = remaining % 60; remaining /= 60;
    let m = remaining % 60; remaining /= 60;
    let h = remaining % 24; remaining /= 24;

    // Days since epoch to year/month/day
    let mut days = remaining as i64;
    let mut year = 1970i64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year { break; }
        days -= days_in_year;
        year += 1;
    }
    let months = [31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for &d in &months {
        if days < d { break; }
        days -= d;
        month += 1;
    }
    let day = days + 1;
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
