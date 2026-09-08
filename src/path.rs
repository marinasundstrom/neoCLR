//! Lexical path operations using host separators; no filesystem access.
fn separator(c: char) -> bool {
    c == '/' || (cfg!(windows) && c == '\\')
}

pub(crate) fn combine(left: &str, right: &str) -> String {
    let drive_rooted = cfg!(windows)
        && right.as_bytes().get(1) == Some(&b':')
        && right.as_bytes()[0].is_ascii_alphabetic();
    if left.is_empty() || right.starts_with(separator) || drive_rooted {
        right.to_owned()
    } else if right.is_empty() {
        left.to_owned()
    } else if left.ends_with(separator) {
        format!("{left}{right}")
    } else {
        format!("{left}{}{right}", std::path::MAIN_SEPARATOR)
    }
}

pub(crate) fn file_name(path: &str) -> String {
    #[cfg(windows)]
    let prefix = match std::path::Path::new(path).components().next() {
        Some(std::path::Component::Prefix(prefix)) => prefix.as_os_str().len(),
        _ => 0,
    };
    #[cfg(not(windows))]
    let prefix = 0;
    let start = path.rfind(separator).map_or(0, |i| i + 1).max(prefix);
    path[start..].to_owned()
}
