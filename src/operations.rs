// ─────────────────────────────────────────────────────────────────────────────
// Operations — bulk file actions on scan results
// ─────────────────────────────────────────────────────────────────────────────

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use crate::domain::Side;
use crate::scanner::ScanResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Zip,
    Move,
    Delete,
    Export,
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Operation::Zip => "Zip",
            Operation::Move => "Move",
            Operation::Delete => "Delete",
            Operation::Export => "Export list",
        })
    }
}

pub fn run_operation(
    op: Operation,
    dir: &str,
    results: &[ScanResult],
    filter_side: Side,
    output: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let targets: Vec<&ScanResult> = results
        .iter()
        .filter(|r| r.effective_side() == filter_side)
        .collect();

    match op {
        Operation::Zip => {
            use zip::write::FileOptions;
            let mut w = zip::ZipWriter::new(fs::File::create(output)?);
            let opts = FileOptions::default();
            let mut n = 0usize;
            for r in &targets {
                let src = Path::new(dir).join(&r.jar_name);
                if src.is_file() {
                    let mut buf = Vec::new();
                    fs::File::open(&src)?.read_to_end(&mut buf)?;
                    w.start_file(&r.jar_name, opts)?;
                    w.write_all(&buf)?;
                    n += 1;
                }
            }
            w.finish()?;
            Ok(n)
        }
        Operation::Move => {
            fs::create_dir_all(output)?;
            let mut n = 0usize;
            for r in &targets {
                let src = Path::new(dir).join(&r.jar_name);
                let dst = Path::new(output).join(&r.jar_name);
                if src.is_file() {
                    if fs::rename(&src, &dst).is_err() {
                        fs::copy(&src, &dst)?;
                        fs::remove_file(&src)?;
                    }
                    n += 1;
                }
            }
            Ok(n)
        }
        Operation::Delete => {
            let mut n = 0usize;
            for r in &targets {
                let p = Path::new(dir).join(&r.jar_name);
                if p.is_file() {
                    fs::remove_file(p)?;
                    n += 1;
                }
            }
            Ok(n)
        }
        Operation::Export => {
            let mut f = fs::File::create(output)?;
            let mut n = 0usize;
            for r in &targets {
                writeln!(f, "{}", r.jar_name)?;
                n += 1;
            }
            Ok(n)
        }
    }
}
