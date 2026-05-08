use crate::analyzer::{self, DomainStats, Overview};
use crate::har::Entry;

const BODY_TRUNCATE: usize = 2000;

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...(truncated, {} bytes total)", &s[..max], s.len())
    }
}

fn format_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// ── overview ──

pub fn overview_md(ov: &Overview, creator: &str) -> String {
    let mut out = String::new();
    out.push_str("# HAR Overview\n\n");
    out.push_str(&format!("- **Creator**: {creator}\n"));
    out.push_str(&format!("- **Total requests**: {}\n", ov.total_requests));
    out.push_str(&format!(
        "- **Time range**: {} → {}\n",
        ov.time_range.0, ov.time_range.1
    ));
    out.push_str(&format!(
        "- **Total time**: {:.0} ms\n",
        ov.total_time_ms
    ));
    out.push_str(&format!(
        "- **Total size**: {}\n",
        format_size(ov.total_size)
    ));

    out.push_str("\n## Methods\n\n");
    out.push_str("| Method | Count |\n|--------|-------|\n");
    let mut methods: Vec<_> = ov.methods.iter().collect();
    methods.sort_by(|a, b| b.1.cmp(a.1));
    for (m, c) in methods {
        out.push_str(&format!("| {m} | {c} |\n"));
    }

    out.push_str("\n## Status Codes\n\n");
    out.push_str("| Status | Count |\n|--------|-------|\n");
    let mut codes: Vec<_> = ov.status_codes.iter().collect();
    codes.sort_by_key(|(s, _)| **s);
    for (s, c) in codes {
        out.push_str(&format!("| {s} | {c} |\n"));
    }

    out.push_str("\n## Domains\n\n");
    out.push_str("| Domain | Requests |\n|--------|----------|\n");
    let mut domains: Vec<_> = ov.domains.iter().collect();
    domains.sort_by(|a, b| b.1.cmp(a.1));
    for (d, c) in domains {
        out.push_str(&format!("| {d} | {c} |\n"));
    }

    out
}

pub fn overview_json(ov: &Overview, creator: &str) -> String {
    let v = serde_json::json!({
        "creator": creator,
        "total_requests": ov.total_requests,
        "time_range": { "start": ov.time_range.0, "end": ov.time_range.1 },
        "total_time_ms": ov.total_time_ms,
        "total_size_bytes": ov.total_size,
        "methods": ov.methods,
        "status_codes": ov.status_codes,
        "domains": ov.domains,
    });
    serde_json::to_string_pretty(&v).unwrap()
}

// ── list ──

pub fn list_md(entries: &[(usize, &Entry)]) -> String {
    let mut out = String::new();
    out.push_str("# Request List\n\n");
    out.push_str("| # | Method | Status | Domain | Path | Time | Size |\n");
    out.push_str("|---|--------|--------|--------|------|------|------|\n");
    for (i, e) in entries {
        let domain = analyzer::url_domain(&e.request.url);
        let path = analyzer::url_path(&e.request.url);
        let size = if e.response.content.size >= 0 {
            format_size(e.response.content.size)
        } else {
            "-".into()
        };
        out.push_str(&format!(
            "| {i} | {} | {} | {domain} | `{path}` | {:.0}ms | {size} |\n",
            e.request.method, e.response.status, e.time
        ));
    }
    out
}

pub fn list_json(entries: &[(usize, &Entry)]) -> String {
    let arr: Vec<_> = entries
        .iter()
        .map(|(i, e)| {
            serde_json::json!({
                "id": i,
                "method": e.request.method,
                "status": e.response.status,
                "domain": analyzer::url_domain(&e.request.url),
                "path": analyzer::url_path(&e.request.url),
                "time_ms": e.time,
                "size": e.response.content.size,
            })
        })
        .collect();
    serde_json::to_string_pretty(&arr).unwrap()
}

// ── detail ──

pub fn detail_md(id: usize, e: &Entry) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Request #{id}\n\n"));
    out.push_str(&format!(
        "**{} {}** {}\n\n",
        e.request.method, e.request.http_version, e.request.url
    ));
    out.push_str(&format!("- **Status**: {} {}\n", e.response.status, e.response.status_text));
    out.push_str(&format!("- **Time**: {:.0} ms\n", e.time));
    out.push_str(&format!("- **Started**: {}\n", e.started_date_time));
    if let Some(ip) = &e.server_ip_address {
        if !ip.is_empty() {
            out.push_str(&format!("- **Server IP**: {ip}\n"));
        }
    }
    if !e.response.content.mime_type.is_empty() {
        out.push_str(&format!("- **Content-Type**: {}\n", e.response.content.mime_type));
    }
    if e.response.content.size >= 0 {
        out.push_str(&format!("- **Body size**: {}", format_size(e.response.content.size)));
        if e.response.content.compression > 0 {
            out.push_str(&format!(" (compressed, saved {})", format_size(e.response.content.compression)));
        }
        out.push('\n');
    }
    if !e.response.redirect_url.is_empty() {
        out.push_str(&format!("- **Redirect to**: {}\n", e.response.redirect_url));
    }

    // Timings
    out.push_str("\n## Timings\n\n");
    out.push_str("| Phase | ms |\n|-------|----|\n");
    let timings = [
        ("DNS", e.timings.dns),
        ("Connect", e.timings.connect),
        ("SSL", e.timings.ssl),
        ("Send", e.timings.send),
        ("Wait", e.timings.wait),
        ("Receive", e.timings.receive),
        ("Blocked", e.timings.blocked),
    ];
    for (name, val) in timings {
        let v = if val >= 0.0 { format!("{val:.0}") } else { "-".into() };
        out.push_str(&format!("| {name} | {v} |\n"));
    }

    // Request headers
    if !e.request.headers.is_empty() {
        out.push_str("\n## Request Headers\n\n");
        for h in &e.request.headers {
            out.push_str(&format!("- **{}**: {}\n", h.name, h.value));
        }
    }

    // Query string
    if !e.request.query_string.is_empty() {
        out.push_str("\n## Query Parameters\n\n");
        out.push_str("| Name | Value |\n|------|-------|\n");
        for q in &e.request.query_string {
            out.push_str(&format!("| `{}` | `{}` |\n", q.name, q.value));
        }
    }

    // Post data
    if let Some(pd) = &e.request.post_data {
        out.push_str("\n## Request Body\n\n");
        out.push_str(&format!("- **Content-Type**: {}\n", pd.mime_type));
        if let Some(text) = &pd.text {
            out.push_str("\n```\n");
            out.push_str(&truncate(text, BODY_TRUNCATE));
            out.push_str("\n```\n");
        }
        if !pd.params.is_empty() {
            out.push_str("\n| Param | Value |\n|-------|-------|\n");
            for p in &pd.params {
                out.push_str(&format!("| `{}` | `{}` |\n", p.name, truncate(&p.value, 200)));
            }
        }
    }

    // Request cookies
    if !e.request.cookies.is_empty() {
        out.push_str("\n## Request Cookies\n\n");
        for c in &e.request.cookies {
            let val = &c.value;
            out.push_str(&format!("- **{}**: `{}`\n", c.name, truncate(val, 200)));
        }
    }

    // Response headers
    if !e.response.headers.is_empty() {
        out.push_str("\n## Response Headers\n\n");
        for h in &e.response.headers {
            out.push_str(&format!("- **{}**: {}\n", h.name, h.value));
        }
    }

    // Response cookies
    if !e.response.cookies.is_empty() {
        out.push_str("\n## Response Cookies\n\n");
        for c in &e.response.cookies {
            let val = &c.value;
            out.push_str(&format!("- **{}**: `{}`\n", c.name, truncate(val, 200)));
        }
    }

    // Body
    if let Some(body) = analyzer::decode_body(&e.response.content) {
        out.push_str("\n## Response Body\n\n");
        out.push_str("```\n");
        out.push_str(&truncate(&body, BODY_TRUNCATE));
        out.push_str("\n```\n");
    }

    out
}

pub fn detail_json(id: usize, e: &Entry) -> String {
    let body = analyzer::decode_body(&e.response.content);
    let post_data = e.request.post_data.as_ref().map(|pd| {
        serde_json::json!({
            "mime_type": pd.mime_type,
            "text": pd.text.as_ref().map(|t| truncate(t, BODY_TRUNCATE)),
            "params": pd.params.iter().map(|p| serde_json::json!({"name": p.name, "value": p.value})).collect::<Vec<_>>(),
        })
    });
    let redirect_url = if e.response.redirect_url.is_empty() {
        None
    } else {
        Some(&e.response.redirect_url)
    };
    let v = serde_json::json!({
        "id": id,
        "method": e.request.method,
        "url": e.request.url,
        "http_version": e.request.http_version,
        "status": e.response.status,
        "status_text": e.response.status_text,
        "time_ms": e.time,
        "started": e.started_date_time,
        "server_ip": e.server_ip_address,
        "content_type": e.response.content.mime_type,
        "body_size": e.response.content.size,
        "compression": e.response.content.compression,
        "redirect_url": redirect_url,
        "timings": {
            "dns": e.timings.dns,
            "connect": e.timings.connect,
            "ssl": e.timings.ssl,
            "send": e.timings.send,
            "wait": e.timings.wait,
            "receive": e.timings.receive,
            "blocked": e.timings.blocked,
        },
        "request_headers": e.request.headers.iter().map(|h| (h.name.clone(), serde_json::Value::String(h.value.clone()))).collect::<serde_json::Map<String, serde_json::Value>>(),
        "query_params": e.request.query_string.iter().map(|q| (q.name.clone(), serde_json::Value::String(q.value.clone()))).collect::<serde_json::Map<String, serde_json::Value>>(),
        "post_data": post_data,
        "response_headers": e.response.headers.iter().map(|h| (h.name.clone(), serde_json::Value::String(h.value.clone()))).collect::<serde_json::Map<String, serde_json::Value>>(),
        "response_body": body.map(|b| truncate(&b, BODY_TRUNCATE)),
    });
    serde_json::to_string_pretty(&v).unwrap()
}

// ── headers ──

pub fn headers_md(name: &str, entries: &[(usize, &Entry)]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Header: {name}\n\n"));
    out.push_str("| # | Method | Status | Domain | Header Value |\n");
    out.push_str("|---|--------|--------|--------|-------------|\n");
    for (i, e) in entries {
        let domain = analyzer::url_domain(&e.request.url);
        let values: Vec<String> = e
            .request
            .headers
            .iter()
            .chain(e.response.headers.iter())
            .filter(|h| h.name.to_lowercase().contains(&name.to_lowercase()))
            .map(|h| format!("{}: {}", h.name, truncate(&h.value, 100)))
            .collect();
        for v in values {
            out.push_str(&format!(
                "| {i} | {} | {} | {domain} | {v} |\n",
                e.request.method, e.response.status
            ));
        }
    }
    out
}

pub fn headers_json(name: &str, entries: &[(usize, &Entry)]) -> String {
    let arr: Vec<_> = entries
        .iter()
        .flat_map(|(i, e)| {
            let req_hits: Vec<_> = e
                .request
                .headers
                .iter()
                .filter(|h| h.name.to_lowercase().contains(&name.to_lowercase()))
                .map(|h| {
                    serde_json::json!({
                        "id": i,
                        "source": "request",
                        "name": h.name,
                        "value": h.value,
                    })
                })
                .collect();
            let resp_hits: Vec<_> = e
                .response
                .headers
                .iter()
                .filter(|h| h.name.to_lowercase().contains(&name.to_lowercase()))
                .map(|h| {
                    serde_json::json!({
                        "id": i,
                        "source": "response",
                        "name": h.name,
                        "value": h.value,
                    })
                })
                .collect();
            req_hits.into_iter().chain(resp_hits)
        })
        .collect();
    serde_json::to_string_pretty(&arr).unwrap()
}

// ── domains ──

pub fn domains_md(stats: &[DomainStats]) -> String {
    let mut out = String::new();
    out.push_str("# Domain Statistics\n\n");
    out.push_str("| Domain | Requests | Avg Time | Errors | Total Size |\n");
    out.push_str("|--------|----------|----------|--------|------------|\n");
    for s in stats {
        out.push_str(&format!(
            "| {} | {} | {:.0}ms | {} | {} |\n",
            s.domain, s.count, s.avg_time_ms, s.errors, format_size(s.total_size)
        ));
    }
    out
}

pub fn domains_json(stats: &[DomainStats]) -> String {
    let arr: Vec<_> = stats
        .iter()
        .map(|s| {
            serde_json::json!({
                "domain": s.domain,
                "requests": s.count,
                "avg_time_ms": s.avg_time_ms,
                "errors": s.errors,
                "total_size_bytes": s.total_size,
            })
        })
        .collect();
    serde_json::to_string_pretty(&arr).unwrap()
}

// ── timeline ──

pub fn timeline_md(entries: &[(usize, &Entry)]) -> String {
    let mut out = String::new();
    out.push_str("# Timeline\n\n");
    for (i, e) in entries {
        let domain = analyzer::url_domain(&e.request.url);
        let path = analyzer::url_path(&e.request.url);
        out.push_str(&format!(
            "- `{}` **{}** {} {} {} {:.0}ms\n",
            e.started_date_time, e.request.method, e.response.status, domain, path, e.time
        ));
        if *i != entries.last().map(|(id, _)| *id).unwrap_or(0) {
            // no separator needed
        }
    }
    out
}

pub fn timeline_json(entries: &[(usize, &Entry)]) -> String {
    let arr: Vec<_> = entries
        .iter()
        .map(|(i, e)| {
            serde_json::json!({
                "id": i,
                "time": e.started_date_time,
                "method": e.request.method,
                "status": e.response.status,
                "domain": analyzer::url_domain(&e.request.url),
                "path": analyzer::url_path(&e.request.url),
                "duration_ms": e.time,
            })
        })
        .collect();
    serde_json::to_string_pretty(&arr).unwrap()
}
