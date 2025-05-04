use std::path::Path;
use std::{fs::File, io::Read};

use docx_rs::read_docx;

pub fn extract(path: &Path) -> anyhow::Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let document = read_docx(&buffer)?;

    let mut text = String::new();
    for child in &document.document.children {
        if let docx_rs::DocumentChild::Paragraph(p) = child {
            for run in &p.children {
                if let docx_rs::ParagraphChild::Run(r) = run {
                    for text_node in &r.children {
                        if let docx_rs::RunChild::Text(t) = text_node {
                            text.push_str(&t.text);
                        }
                    }
                }
            }
            text.push('\n');
        }
    }

    Ok(text)
}
