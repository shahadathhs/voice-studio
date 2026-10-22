use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};

use super::ModelFile;
use crate::engines::Control;
use crate::{Error, Result};

const CHUNK_BYTES: usize = 256 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub downloaded: u64,
    pub total: u64,
}

/// Downloads `file` to `dest`, resuming a previous partial download when the server allows it.
///
/// Data is written to `<dest>.part` and moved into place only after its size and SHA-256 match, so
/// `dest` is never a partial or corrupt file. Cancelling (the callback returning
/// [`Control::Cancel`]) keeps the `.part` file so the next attempt resumes.
pub fn download_file(
    file: &ModelFile,
    dest: &Path,
    on_progress: &mut dyn FnMut(Progress) -> Control,
) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let part = part_path(dest);
    let start = resumable_bytes(&part, file.size_bytes)?;
    report(on_progress, start, file.size_bytes)?;
    if start < file.size_bytes {
        fetch(file, &part, start, on_progress)?;
    }
    finish(file, &part, dest)
}

fn part_path(dest: &Path) -> PathBuf {
    let mut name = dest.as_os_str().to_owned();
    name.push(".part");
    PathBuf::from(name)
}

/// Bytes of a usable partial download; a partial file larger than expected is discarded.
fn resumable_bytes(part: &Path, expected: u64) -> Result<u64> {
    match fs::metadata(part) {
        Ok(meta) if meta.len() <= expected => Ok(meta.len()),
        Ok(_) => {
            fs::remove_file(part)?;
            Ok(0)
        }
        Err(_) => Ok(0),
    }
}

fn report(
    on_progress: &mut dyn FnMut(Progress) -> Control,
    downloaded: u64,
    total: u64,
) -> Result<()> {
    match on_progress(Progress { downloaded, total }) {
        Control::Continue => Ok(()),
        Control::Cancel => Err(Error::Cancelled),
    }
}

fn fetch(
    file: &ModelFile,
    part: &Path,
    start: u64,
    on_progress: &mut dyn FnMut(Progress) -> Control,
) -> Result<()> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .build()
        .into();
    let mut request = agent.get(&file.url);
    if start > 0 {
        request = request.header("Range", format!("bytes={start}-"));
    }
    let response = request.call().map_err(|e| Error::Network(e.to_string()))?;
    // A server that ignores the Range header sends the whole file, so start over.
    let resumed = start > 0 && response.status().as_u16() == 206;
    let mut downloaded = if resumed { start } else { 0 };
    let mut out = OpenOptions::new()
        .create(true)
        .write(true)
        .append(resumed)
        .truncate(!resumed)
        .open(part)?;

    let mut body = response.into_body().into_reader();
    let mut buffer = vec![0u8; CHUNK_BYTES];
    loop {
        let read = body
            .read(&mut buffer)
            .map_err(|e| Error::Network(e.to_string()))?;
        if read == 0 {
            break;
        }
        out.write_all(&buffer[..read])?;
        downloaded += read as u64;
        // Reported on every chunk so cancelling is immediate; callers throttle what they display.
        out.flush()?;
        report(on_progress, downloaded, file.size_bytes)?;
    }
    Ok(())
}

fn finish(file: &ModelFile, part: &Path, dest: &Path) -> Result<()> {
    let size = fs::metadata(part)?.len();
    if size != file.size_bytes {
        return Err(Error::Network(format!(
            "incomplete download of {}: got {size} of {} bytes; run it again to resume",
            file.name, file.size_bytes
        )));
    }
    if !sha256_file(part)?.eq_ignore_ascii_case(&file.sha256) {
        fs::remove_file(part)?;
        return Err(Error::Network(format!(
            "checksum mismatch for {}; the partial download was discarded",
            file.name
        )));
    }
    fs::rename(part, dest)?;
    Ok(())
}

/// Lower-case hex SHA-256 of a file, read in chunks.
pub(super) fn sha256_file(path: &Path) -> Result<String> {
    let mut reader = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; CHUNK_BYTES];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(to_hex(&hasher.finalize()))
}

fn to_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // Writing to a String cannot fail.
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}
