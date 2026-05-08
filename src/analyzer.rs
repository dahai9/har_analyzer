use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::har::{self, Entry, Har};

pub fn load_har(path: &Path) -> Result<(Har, Vec<Entry>), String> {
    let data = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;
    let har: Har =
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse HAR: {e}"))?;

    let mut entries = Vec::new();
    let mut skipped = 0;
    for v in &har.log.raw_entries {
        match har::parse_entry(v) {
            Some(e) => entries.push(e),
            None => skipped += 1,
        }
    }
    if skipped > 0 {
        eprintln!("Note: skipped {skipped} non-standard entries");
    }
    Ok((har, entries))
}

pub fn url_path(url: &str) -> &str {
    url.find("://")
        .and_then(|i| url[i + 3..].find('/').map(|j| &url[i + 3 + j..]))
        .unwrap_or("/")
}

pub fn url_domain(url: &str) -> &str {
    url.find("://")
        .map(|i| &url[i + 3..])
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("")
}

pub struct Overview {
    pub creator: String,
    pub total_requests: usize,
    pub time_range: (String, String),
    pub methods: HashMap<String, usize>,
    pub status_codes: HashMap<u16, usize>,
    pub domains: HashMap<String, usize>,
    pub total_size: i64,
    pub total_time_ms: f64,
}

pub fn analyze_overview(entries: &[Entry]) -> Overview {
    let mut methods: HashMap<String, usize> = HashMap::new();
    let mut status_codes: HashMap<u16, usize> = HashMap::new();
    let mut domains: HashMap<String, usize> = HashMap::new();
    let mut total_size: i64 = 0;

    for e in entries {
        *methods.entry(e.request.method.clone()).or_default() += 1;
        *status_codes.entry(e.response.status).or_default() += 1;
        let domain = url_domain(&e.request.url).to_string();
        *domains.entry(domain).or_default() += 1;
        total_size += e.response.content.size;
    }

    let first_time = entries
        .first()
        .map(|e| e.started_date_time.clone())
        .unwrap_or_default();
    let last_time = entries
        .last()
        .map(|e| e.started_date_time.clone())
        .unwrap_or_default();
    let total_time_ms: f64 = entries.iter().map(|e| e.time).sum();

    Overview {
        creator: String::new(),
        total_requests: entries.len(),
        time_range: (first_time, last_time),
        methods,
        status_codes,
        domains,
        total_size,
        total_time_ms,
    }
}

pub struct DomainStats {
    pub domain: String,
    pub count: usize,
    pub avg_time_ms: f64,
    pub errors: usize,
    pub total_size: i64,
}

pub fn analyze_domains(entries: &[Entry]) -> Vec<DomainStats> {
    let mut map: HashMap<String, Vec<&Entry>> = HashMap::new();
    for e in entries {
        let d = url_domain(&e.request.url).to_string();
        map.entry(d).or_default().push(e);
    }
    let mut stats: Vec<DomainStats> = map
        .into_iter()
        .map(|(domain, ents)| {
            let count = ents.len();
            let avg_time_ms = ents.iter().map(|e| e.time).sum::<f64>() / count as f64;
            let errors = ents.iter().filter(|e| e.response.status >= 400).count();
            let total_size = ents.iter().map(|e| e.response.content.size).sum();
            DomainStats {
                domain,
                count,
                avg_time_ms,
                errors,
                total_size,
            }
        })
        .collect();
    stats.sort_by(|a, b| b.count.cmp(&a.count));
    stats
}

pub fn filter_entries<'a>(
    entries: &'a [Entry],
    domain: Option<&str>,
    method: Option<&str>,
    status: Option<u16>,
) -> Vec<(usize, &'a Entry)> {
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            if let Some(d) = domain {
                if !url_domain(&e.request.url).contains(d) {
                    return false;
                }
            }
            if let Some(m) = method {
                if e.request.method != m {
                    return false;
                }
            }
            if let Some(s) = status {
                if e.response.status != s {
                    return false;
                }
            }
            true
        })
        .collect()
}

pub fn filter_errors(entries: &[Entry]) -> Vec<(usize, &Entry)> {
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.response.status >= 400)
        .collect()
}

pub fn search_entries<'a>(entries: &'a [Entry], keyword: &str) -> Vec<(usize, &'a Entry)> {
    let kw = keyword.to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            e.request.url.to_lowercase().contains(&kw)
                || e.request.headers.iter().any(|h| {
                    h.name.to_lowercase().contains(&kw) || h.value.to_lowercase().contains(&kw)
                })
                || e.response.headers.iter().any(|h| {
                    h.name.to_lowercase().contains(&kw) || h.value.to_lowercase().contains(&kw)
                })
                || e.response
                    .content
                    .text
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&kw))
                    .unwrap_or(false)
        })
        .collect()
}

pub fn decode_body(content: &crate::har::Content) -> Option<String> {
    let text = content.text.as_ref()?;
    if text.is_empty() {
        return None;
    }
    match content.encoding.as_deref() {
        Some("base64") => {
            use base64::Engine;
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(text)
                .ok()?;
            use flate2::read::GzDecoder;
            use std::io::Read;
            let mut decoder = GzDecoder::new(&decoded[..]);
            let mut decompressed = String::new();
            if decoder.read_to_string(&mut decompressed).is_ok() {
                Some(decompressed)
            } else {
                String::from_utf8(decoded).ok()
            }
        }
        _ => {
            use flate2::read::GzDecoder;
            use std::io::Read;
            let mut decoder = GzDecoder::new(text.as_bytes());
            let mut decompressed = String::new();
            if decoder.read_to_string(&mut decompressed).is_ok() {
                Some(decompressed)
            } else {
                Some(text.clone())
            }
        }
    }
}
