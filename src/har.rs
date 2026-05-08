#![allow(dead_code)]

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Har {
    pub log: Log,
}

#[derive(Debug, Deserialize)]
pub struct Log {
    pub version: String,
    pub creator: Creator,
    #[serde(default, rename = "entries")]
    pub raw_entries: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Creator {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub server_ip_address: Option<String>,
    pub started_date_time: String,
    pub time: f64,
    pub timings: Timings,
    pub request: Request,
    pub response: Response,
}

#[derive(Debug, Clone)]
pub struct Timings {
    pub connect: f64,
    pub send: f64,
    pub dns: f64,
    pub ssl: f64,
    pub wait: f64,
    pub blocked: f64,
    pub receive: f64,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub body_size: i64,
    pub headers_size: i64,
    pub cookies: Vec<Cookie>,
    pub headers: Vec<Header>,
    pub query_string: Vec<QueryString>,
    pub http_version: String,
    pub url: String,
    pub post_data: Option<PostData>,
}

#[derive(Debug, Clone)]
pub struct PostData {
    pub mime_type: String,
    pub text: Option<String>,
    pub params: Vec<PostParam>,
}

#[derive(Debug, Clone)]
pub struct PostParam {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub content: Content,
    pub body_size: i64,
    pub headers_size: i64,
    pub cookies: Vec<Cookie>,
    pub status_text: String,
    pub headers: Vec<Header>,
    pub http_version: String,
    pub redirect_url: String,
}

#[derive(Debug, Clone)]
pub struct Content {
    pub text: Option<String>,
    pub size: i64,
    pub mime_type: String,
    pub encoding: Option<String>,
    pub compression: i64,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct QueryString {
    pub name: String,
    pub value: String,
}

// Helper to get a string from JSON value
fn get_str(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn get_f64(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
}

fn get_i64(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(|v| v.as_i64()).unwrap_or(-1)
}

fn parse_headers(v: &Value) -> Vec<Header> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|h| {
                    Some(Header {
                        name: h.get("name")?.as_str()?.to_string(),
                        value: h.get("value")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_cookies(v: &Value) -> Vec<Cookie> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    Some(Cookie {
                        name: c.get("name")?.as_str()?.to_string(),
                        value: c.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_query_string(v: &Value) -> Vec<QueryString> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|q| {
                    Some(QueryString {
                        name: q.get("name")?.as_str()?.to_string(),
                        value: q.get("value")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_timings(v: &Value) -> Timings {
    Timings {
        connect: get_f64(v, "connect"),
        send: get_f64(v, "send"),
        dns: get_f64(v, "dns"),
        ssl: get_f64(v, "ssl"),
        wait: get_f64(v, "wait"),
        blocked: get_f64(v, "blocked"),
        receive: get_f64(v, "receive"),
    }
}

fn parse_content(v: &Value) -> Content {
    Content {
        text: v.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()),
        size: get_i64(v, "size"),
        mime_type: get_str(v, "mimeType"),
        encoding: v.get("encoding").and_then(|e| e.as_str()).map(|s| s.to_string()),
        compression: get_i64(v, "compression"),
    }
}

fn parse_post_data(v: &Value) -> Option<PostData> {
    if v.is_null() {
        return None;
    }
    Some(PostData {
        mime_type: get_str(v, "mimeType"),
        text: v.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()),
        params: v
            .get("params")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| {
                        Some(PostParam {
                            name: p.get("name")?.as_str()?.to_string(),
                            value: p.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
    })
}

fn parse_request(v: &Value) -> Option<Request> {
    let url = v.get("url")?.as_str()?;
    Some(Request {
        method: get_str(v, "method"),
        body_size: get_i64(v, "bodySize"),
        headers_size: get_i64(v, "headersSize"),
        cookies: parse_cookies(v.get("cookies").unwrap_or(&Value::Null)),
        headers: parse_headers(v.get("headers").unwrap_or(&Value::Null)),
        query_string: parse_query_string(v.get("queryString").unwrap_or(&Value::Null)),
        http_version: get_str(v, "httpVersion"),
        url: url.to_string(),
        post_data: parse_post_data(v.get("postData").unwrap_or(&Value::Null)),
    })
}

fn parse_response(v: &Value) -> Option<Response> {
    let status = v.get("status")?.as_u64()? as u16;
    let content = parse_content(v.get("content").unwrap_or(&Value::Null));
    Some(Response {
        status,
        content,
        body_size: get_i64(v, "bodySize"),
        headers_size: get_i64(v, "headersSize"),
        cookies: parse_cookies(v.get("cookies").unwrap_or(&Value::Null)),
        status_text: get_str(v, "statusText"),
        headers: parse_headers(v.get("headers").unwrap_or(&Value::Null)),
        http_version: get_str(v, "httpVersion"),
        redirect_url: get_str(v, "redirectURL"),
    })
}

pub fn parse_entry(v: &Value) -> Option<Entry> {
    let request = parse_request(v.get("request")?)?;
    let response = parse_response(v.get("response")?)?;
    Some(Entry {
        server_ip_address: v.get("serverIPAddress").and_then(|v| v.as_str()).map(|s| s.to_string()),
        started_date_time: get_str(v, "startedDateTime"),
        time: get_f64(v, "time"),
        timings: parse_timings(v.get("timings").unwrap_or(&Value::Null)),
        request,
        response,
    })
}
