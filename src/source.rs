//! Explicit file-based source composition; the string assembler never reads files.
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Read source, expanding `.include "relative/path.neoil"` in declaration order.
/// Paths resolve beside the including file. Cycles and nesting beyond 64 files fail.
/// Returned assembler line numbers refer to the expanded source.
pub fn read_source(path: impl AsRef<Path>) -> io::Result<String> {
    expand(path.as_ref(), &mut Vec::new())
}

fn expand(path: &Path, active: &mut Vec<PathBuf>) -> io::Result<String> {
    let canonical = fs::canonicalize(path)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
    if active.contains(&canonical) || active.len() >= 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{}: include cycle or nesting limit exceeded",
                path.display()
            ),
        ));
    }
    active.push(canonical);
    let source = fs::read_to_string(path)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
    let mut output = String::new();
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.split_whitespace().next() == Some(".include") {
            let operand = trimmed[8..].trim();
            let invalid = || {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "{}:{}: expected .include \"path\"",
                        path.display(),
                        index + 1
                    ),
                )
            };
            let rest = operand.strip_prefix('"').ok_or_else(invalid)?;
            let (name, trailing) = rest.split_once('"').ok_or_else(invalid)?;
            if name.is_empty() || !(trailing.trim().is_empty() || trailing.trim().starts_with(';'))
            {
                return Err(invalid());
            }
            let included = path.parent().unwrap_or(Path::new(".")).join(name);
            output.push_str(&expand(&included, active)?);
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    active.pop();
    Ok(output)
}
