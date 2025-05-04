use anyhow::Result;
use pdf_extract::extract_text;
use std::path::Path;

pub fn extract(path: &Path) -> Result<String> {
    let content = extract_text(path)?;
    Ok(content.replace(|c: char| c.is_control(), ""))
}
