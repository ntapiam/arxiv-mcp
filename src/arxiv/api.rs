use quick_xml::events::Event;
use quick_xml::Reader;

use super::types::Paper;

pub struct ArxivClient {
    client: reqwest::Client,
}

impl ArxivClient {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn search(&self, query: &str, max_results: u32) -> anyhow::Result<Vec<Paper>> {
        let url = format!(
            "https://export.arxiv.org/api/query?search_query={}&max_results={}&sortBy=relevance",
            query, max_results
        );
        let resp = self.client.get(&url).send().await?.text().await?;
        parse_atom_feed(&resp)
    }

    pub async fn get_paper_info(&self, paper_id: &str) -> anyhow::Result<Paper> {
        let url = format!(
            "https://export.arxiv.org/api/query?id_list={}",
            paper_id
        );
        let resp = self.client.get(&url).send().await?.text().await?;
        let papers = parse_atom_feed(&resp)?;
        papers
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Paper not found: {}", paper_id))
    }

    pub async fn get_bibtex(&self, paper_id: &str) -> anyhow::Result<String> {
        let url = format!("https://arxiv.org/bibtex/{}", paper_id);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "arXiv returned status {} for paper '{}'",
                resp.status(),
                paper_id
            ));
        }
        let text = resp.text().await?;
        if text.trim().is_empty() {
            return Err(anyhow::anyhow!("No BibTeX entry found for '{}'", paper_id));
        }
        Ok(text)
    }
}

fn extract_arxiv_id(raw: &str) -> String {
    let trimmed = raw.trim();
    // URL like http://arxiv.org/abs/2301.07041v1
    let id = if let Some(pos) = trimmed.rfind('/') {
        &trimmed[pos + 1..]
    } else {
        trimmed
    };
    // Strip version suffix vN
    if let Some(v_pos) = id.rfind('v') {
        let suffix = &id[v_pos + 1..];
        if suffix.chars().all(|c| c.is_ascii_digit()) {
            return id[..v_pos].to_string();
        }
    }
    id.to_string()
}

fn parse_atom_feed(xml: &str) -> anyhow::Result<Vec<Paper>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut papers = Vec::new();

    // State for current entry
    let mut in_entry = false;
    let mut current_tag = String::new();
    let mut id_buf = String::new();
    let mut title_buf = String::new();
    let mut summary_buf = String::new();
    let mut published_buf = String::new();
    let mut authors: Vec<String> = Vec::new();
    let mut categories: Vec<String> = Vec::new();
    let mut url_buf = String::new();
    let mut in_author = false;
    let mut author_name_buf = String::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let local = e.local_name();
                let tag = std::str::from_utf8(local.as_ref()).unwrap_or("").to_string();
                match tag.as_str() {
                    "entry" => {
                        in_entry = true;
                        id_buf.clear();
                        title_buf.clear();
                        summary_buf.clear();
                        published_buf.clear();
                        authors.clear();
                        categories.clear();
                        url_buf.clear();
                    }
                    "author" if in_entry => {
                        in_author = true;
                        author_name_buf.clear();
                    }
                    "link" if in_entry => {
                        let mut href = String::new();
                        let mut rel = String::new();
                        for attr in e.attributes().flatten() {
                            let key = std::str::from_utf8(attr.key.local_name().as_ref())
                                .unwrap_or("")
                                .to_string();
                            let val = attr.unescape_value().unwrap_or_default().to_string();
                            match key.as_str() {
                                "href" => href = val,
                                "rel" => rel = val,
                                _ => {}
                            }
                        }
                        if rel == "alternate" && !href.is_empty() {
                            url_buf = href;
                        }
                    }
                    "category" if in_entry => {
                        for attr in e.attributes().flatten() {
                            let key = std::str::from_utf8(attr.key.local_name().as_ref())
                                .unwrap_or("")
                                .to_string();
                            if key == "term" {
                                let val = attr.unescape_value().unwrap_or_default().to_string();
                                if !categories.contains(&val) {
                                    categories.push(val);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                current_tag = tag;
            }
            Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                let tag = std::str::from_utf8(local.as_ref()).unwrap_or("").to_string();
                match tag.as_str() {
                    "link" if in_entry => {
                        let mut href = String::new();
                        let mut rel = String::new();
                        for attr in e.attributes().flatten() {
                            let key = std::str::from_utf8(attr.key.local_name().as_ref())
                                .unwrap_or("")
                                .to_string();
                            let val = attr.unescape_value().unwrap_or_default().to_string();
                            match key.as_str() {
                                "href" => href = val,
                                "rel" => rel = val,
                                _ => {}
                            }
                        }
                        if rel == "alternate" && !href.is_empty() {
                            url_buf = href;
                        }
                    }
                    "category" if in_entry => {
                        for attr in e.attributes().flatten() {
                            let key = std::str::from_utf8(attr.key.local_name().as_ref())
                                .unwrap_or("")
                                .to_string();
                            if key == "term" {
                                let val = attr.unescape_value().unwrap_or_default().to_string();
                                if !categories.contains(&val) {
                                    categories.push(val);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                if in_entry {
                    let decoded = t.decode().unwrap_or_default();
                    let text = quick_xml::escape::unescape(&decoded)
                        .map(|s| s.into_owned())
                        .unwrap_or_else(|_| decoded.into_owned());
                    if in_author && current_tag == "name" {
                        author_name_buf = text;
                    } else {
                        match current_tag.as_str() {
                            "id" => id_buf = text,
                            "title" => title_buf = text,
                            "summary" => summary_buf = text,
                            "published" => published_buf = text,
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = e.local_name();
                let tag = std::str::from_utf8(local.as_ref()).unwrap_or("").to_string();
                match tag.as_str() {
                    "entry" => {
                        if !id_buf.is_empty() {
                            let arxiv_id = extract_arxiv_id(&id_buf);
                            papers.push(Paper {
                                id: arxiv_id,
                                title: title_buf.trim().to_string(),
                                authors: authors.clone(),
                                published: published_buf.clone(),
                                summary: summary_buf.trim().to_string(),
                                url: url_buf.clone(),
                                categories: categories.clone(),
                            });
                        }
                        in_entry = false;
                        in_author = false;
                    }
                    "author" if in_entry => {
                        if !author_name_buf.is_empty() {
                            authors.push(author_name_buf.clone());
                        }
                        in_author = false;
                    }
                    _ => {}
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(papers)
}
