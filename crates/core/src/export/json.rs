use crate::{Result, Transcript};

pub fn render(t: &Transcript) -> Result<String> {
    let mut s = serde_json::to_string_pretty(t)?;
    s.push('\n');
    Ok(s)
}
