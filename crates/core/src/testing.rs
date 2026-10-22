//! Test support shared by the core and desktop tests. Enabled with the `testing` feature.
#![allow(clippy::unwrap_used, clippy::missing_panics_doc)]

use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use sha2::{Digest, Sha256};

use crate::models::ModelFile;

/// Serves `data` with HTTP Range support and records the start offset of every request.
pub struct TestServer {
    addr: SocketAddr,
    pub range_starts: Arc<Mutex<Vec<Option<u64>>>>,
}

impl TestServer {
    pub fn start(data: Vec<u8>) -> TestServer {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let range_starts = Arc::new(Mutex::new(Vec::new()));
        let (data, log) = (Arc::new(data), Arc::clone(&range_starts));
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let (data, log) = (Arc::clone(&data), Arc::clone(&log));
                thread::spawn(move || serve(stream, &data, &log));
            }
        });
        TestServer { addr, range_starts }
    }

    pub fn url(&self) -> String {
        format!("http://{}/model.bin", self.addr)
    }
}

fn serve(mut stream: TcpStream, data: &[u8], log: &Mutex<Vec<Option<u64>>>) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut range_start = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
            range_start = value.trim().trim_end_matches('-').parse().ok();
        }
    }
    log.lock().unwrap().push(range_start);
    let start = usize::try_from(range_start.unwrap_or(0)).unwrap();
    let body = &data[start..];
    let status = if range_start.is_some() {
        "206 Partial Content"
    } else {
        "200 OK"
    };
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

pub fn sample_data() -> Vec<u8> {
    (0..600_000u32)
        .map(|i| u8::try_from(i.wrapping_mul(2_654_435_761) >> 24).unwrap())
        .collect()
}

/// A catalog file entry that points at `server` and describes `data`.
pub fn file_for(server: &TestServer, data: &[u8]) -> ModelFile {
    ModelFile {
        name: "model.bin".into(),
        url: server.url(),
        sha256: hex(&Sha256::digest(data)),
        size_bytes: data.len() as u64,
    }
}
