use std::path::PathBuf;

use crate::cache::Cache;

pub async fn download_and_extract(
    client: &reqwest::Client,
    paper_id: &str,
) -> anyhow::Result<(PathBuf, Vec<String>)> {
    let dest = Cache::paper_dir(paper_id);

    if dest.exists() {
        let files = list_files(&dest)?;
        return Ok((dest, files));
    }

    std::fs::create_dir_all(&dest)?;

    let url = format!("https://arxiv.org/e-print/{}", paper_id);
    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dest);
            return Err(anyhow::anyhow!("Download failed: {}", e));
        }
    };

    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dest);
            return Err(anyhow::anyhow!("Failed to read response body: {}", e));
        }
    };

    if content_type.contains("application/pdf") {
        let pdf_path = dest.join("paper.pdf");
        if let Err(e) = std::fs::write(&pdf_path, &bytes) {
            let _ = std::fs::remove_dir_all(&dest);
            return Err(anyhow::anyhow!("Failed to write PDF: {}", e));
        }
    } else {
        // Try tar.gz extraction
        let cursor = std::io::Cursor::new(&bytes[..]);
        let gz = flate2::read::GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(gz);
        if archive.unpack(&dest).is_err() {
            // Fall back to raw bytes
            let bin_path = dest.join("source.bin");
            if let Err(e) = std::fs::write(&bin_path, &bytes) {
                let _ = std::fs::remove_dir_all(&dest);
                return Err(anyhow::anyhow!("Failed to write source: {}", e));
            }
        }
    }

    let files = list_files(&dest)?;
    Ok((dest, files))
}

fn list_files(dir: &PathBuf) -> anyhow::Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        files.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(files)
}
