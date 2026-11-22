//! HTTP transport. [`UreqTransport`] is the real one (ureq + rustls/ring,
//! the same stack kazzu-audio uses); tests inject a fixture transport.

use std::time::Duration;

use crate::error::{Result, YtmError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl HttpRequest {
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: Method::Get,
            url: url.into(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn post_json(url: impl Into<String>, body: String) -> Self {
        Self {
            method: Method::Post,
            url: url.into(),
            headers: vec![("Content-Type".into(), "application/json".into())],
            body: Some(body),
        }
    }

    pub fn header(mut self, name: &str, value: impl Into<String>) -> Self {
        self.headers.push((name.to_owned(), value.into()));
        self
    }

    /// Value of a header (case-insensitive).
    pub fn header_value(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub retry_after: Option<Duration>,
}

/// Something that performs HTTP requests synchronously.
pub trait Transport: Send + Sync {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse>;
}

/// Largest response body accepted (innertube pages are < 2 MiB).
const MAX_BODY: u64 = 32 * 1024 * 1024;

/// Generic browser-like User-Agent used when a request sets none, so that no
/// library/version string identifies the app.
pub(crate) const DEFAULT_UA: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:140.0) Gecko/20100101 Firefox/140.0";

/// Real network transport. No cookie store, no proxy auto-detection.
pub struct UreqTransport {
    agent: ureq::Agent,
}

impl UreqTransport {
    pub fn new() -> Self {
        Self::with_timeouts(Duration::from_secs(10), Duration::from_secs(30))
    }

    pub fn with_timeouts(connect: Duration, total: Duration) -> Self {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(connect))
            .timeout_global(Some(total))
            .user_agent(DEFAULT_UA)
            .build();
        Self {
            agent: ureq::Agent::new_with_config(config),
        }
    }
}

impl Default for UreqTransport {
    fn default() -> Self {
        Self::new()
    }
}

fn map_err(e: ureq::Error, url: &str) -> YtmError {
    let host = url
        .split("://")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or(url);
    match e {
        ureq::Error::Timeout(t) => YtmError::Network(format!("timeout ({t:?}) talking to {host}")),
        ureq::Error::HostNotFound => YtmError::Network(format!("cannot resolve {host} (offline?)")),
        ureq::Error::ConnectionFailed => YtmError::Network(format!("cannot connect to {host}")),
        ureq::Error::BodyExceedsLimit(n) => YtmError::UpstreamChanged(format!("response from {host} larger than {n} bytes")),
        other => YtmError::Network(format!("{host}: {other}")),
    }
}

impl Transport for UreqTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse> {
        let result = match req.method {
            Method::Get => {
                let mut b = self.agent.get(&req.url);
                for (k, v) in &req.headers {
                    b = b.header(k.as_str(), v.as_str());
                }
                b.call()
            }
            Method::Post => {
                let mut b = self.agent.post(&req.url);
                for (k, v) in &req.headers {
                    b = b.header(k.as_str(), v.as_str());
                }
                b.send(req.body.as_deref().unwrap_or(""))
            }
        };
        let resp = result.map_err(|e| map_err(e, &req.url))?;
        let status = resp.status().as_u16();
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<u64>().ok())
            .map(Duration::from_secs);
        let mut body = resp.into_body();
        let bytes = body
            .with_config()
            .limit(MAX_BODY)
            .read_to_vec()
            .map_err(|e| map_err(e, &req.url))?;
        Ok(HttpResponse {
            status,
            body: String::from_utf8_lossy(&bytes).into_owned(),
            retry_after,
        })
    }
}
