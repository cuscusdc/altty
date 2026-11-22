//! Minimal HTTP/1.1 file server for streaming tests (std only, one thread
//! per connection, `Connection: close`). Supports `Range: bytes=a-b` and can
//! misbehave on demand: latency, throttling, connections cut mid-body,
//! 403 "expired" URLs, servers that ignore Range, total outage.

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

/// One served request, for assertions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub path: String,
    pub range: Option<(u64, Option<u64>)>,
    pub status: u16,
    pub user_agent: Option<String>,
}

#[derive(Default)]
pub struct Behaviour {
    pub files: HashMap<String, (Vec<u8>, &'static str)>,
    /// Paths answered with 403.
    pub expired: HashSet<String>,
    /// Path → number of requests served normally before it starts
    /// answering 403 (a signed URL that expires during playback).
    pub expire_after: HashMap<String, u32>,
    pub latency: Duration,
    /// Body bytes per second (`None` = as fast as possible).
    pub rate: Option<u64>,
    /// Cut the next `drops` responses after `drop_after` body bytes.
    pub drop_after: u64,
    pub drops: u32,
    /// Close every connection without answering (network outage).
    pub down: bool,
    /// Ignore Range headers (always 200 with the whole file).
    pub no_ranges: bool,
    pub hits: Vec<Hit>,
    served: HashMap<String, u32>,
}

pub struct TestServer {
    pub base: String,
    state: Arc<Mutex<Behaviour>>,
    stop: Arc<AtomicBool>,
}

impl TestServer {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(Mutex::new(Behaviour::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let (st, sp) = (Arc::clone(&state), Arc::clone(&stop));
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                if sp.load(Ordering::Relaxed) {
                    break;
                }
                let Ok(conn) = conn else { continue };
                let st = Arc::clone(&st);
                std::thread::spawn(move || {
                    let _ = handle(conn, &st);
                });
            }
        });
        Self { base, state, stop }
    }

    pub fn state(&self) -> MutexGuard<'_, Behaviour> {
        self.state.lock().unwrap()
    }

    pub fn add(&self, path: &str, data: Vec<u8>, content_type: &'static str) -> String {
        self.state().files.insert(path.to_owned(), (data, content_type));
        format!("{}{path}?sig=abc", self.base)
    }

    pub fn hits(&self, path: &str) -> Vec<Hit> {
        self.state().hits.iter().filter(|h| h.path == path).cloned().collect()
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // Wake the accept loop.
        let _ = TcpStream::connect(self.base.trim_start_matches("http://"));
    }
}

fn handle(conn: TcpStream, state: &Mutex<Behaviour>) -> std::io::Result<()> {
    conn.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut reader = BufReader::new(conn.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let target = line.split_whitespace().nth(1).unwrap_or("/").to_owned();
    let path = target.split('?').next().unwrap_or("/").to_owned();
    let mut range = None;
    let mut user_agent = None;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h == "\r\n" {
            break;
        }
        let (name, value) = h.split_once(':').unwrap_or((&h, ""));
        let value = value.trim();
        if name.eq_ignore_ascii_case("range") {
            range = parse_range(value);
        } else if name.eq_ignore_ascii_case("user-agent") {
            user_agent = Some(value.to_owned());
        }
    }

    let (latency, rate, down) = {
        let st = state.lock().unwrap();
        (st.latency, st.rate, st.down)
    };
    if down {
        return Ok(()); // connection closed without a response
    }
    std::thread::sleep(latency);

    let mut st = state.lock().unwrap();
    let served = *st.served.entry(path.clone()).and_modify(|n| *n += 1).or_insert(1);
    let expired = st.expired.contains(&path) || st.expire_after.get(&path).is_some_and(|&k| served > k);
    let file = st.files.get(&path).cloned();
    let no_ranges = st.no_ranges;
    let cut = if st.drops > 0 {
        st.drops -= 1;
        Some(st.drop_after)
    } else {
        None
    };

    let mut out = conn;
    let (status, head, body): (u16, String, Vec<u8>) = match file {
        _ if expired => (403, String::new(), b"expired".to_vec()),
        None => (404, String::new(), b"not found".to_vec()),
        Some((data, ctype)) => {
            let len = data.len() as u64;
            match range {
                Some((a, b)) if !no_ranges => {
                    if a >= len {
                        (416, format!("Content-Range: bytes */{len}\r\n"), Vec::new())
                    } else {
                        let b = b.unwrap_or(len - 1).min(len - 1);
                        (
                            206,
                            format!("Content-Type: {ctype}\r\nContent-Range: bytes {a}-{b}/{len}\r\nAccept-Ranges: bytes\r\n"),
                            data[a as usize..=b as usize].to_vec(),
                        )
                    }
                }
                _ => (200, format!("Content-Type: {ctype}\r\n"), data),
            }
        }
    };
    st.hits.push(Hit {
        path,
        range,
        status,
        user_agent,
    });
    drop(st);

    let reason = match status {
        200 => "OK",
        206 => "Partial Content",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "Range Not Satisfiable",
    };
    write!(
        out,
        "HTTP/1.1 {status} {reason}\r\n{head}Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    let limit = cut.map_or(body.len(), |c| (c as usize).min(body.len()));
    for chunk in body[..limit].chunks(8 * 1024) {
        out.write_all(chunk)?;
        if let Some(rate) = rate {
            std::thread::sleep(Duration::from_secs_f64(chunk.len() as f64 / rate as f64));
        }
    }
    out.flush()?;
    // Dropping `out` closes the connection (cut responses end early).
    Ok(())
}

fn parse_range(v: &str) -> Option<(u64, Option<u64>)> {
    let r = v.strip_prefix("bytes=")?;
    let (a, b) = r.split_once('-')?;
    let a = a.trim().parse().ok()?;
    let b = b.trim();
    let b = if b.is_empty() { None } else { Some(b.parse().ok()?) };
    Some((a, b))
}
