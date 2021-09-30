use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// Splits [`Path`] into two parts separated by `target`. The `target` itself is included
/// in the end of first part.
///
/// ```
/// # use std::path::Path;
/// # use bazelci_agent::utils::split_path_inclusive;
///
/// let path = Path::new("a/b/c");
/// let (first, second) = split_path_inclusive(path, "b").unwrap();
/// assert_eq!(first, Path::new("a/b"));
/// assert_eq!(second, Path::new("c"));
/// ```
///
pub fn split_path_inclusive(path: &Path, target: &str) -> Option<(PathBuf, PathBuf)> {
    let mut iter = path.iter();

    let mut first = PathBuf::new();
    let mut found = false;
    while let Some(comp) = iter.next() {
        first.push(Path::new(comp));
        if comp == target {
            found = true;
            break;
        }
    }

    if found {
        let second: PathBuf = iter.collect();
        Some((first, second))
    } else {
        None
    }
}

pub fn load_file(path_or_url: &str) -> Result<String> {
    if path_or_url.starts_with("http") {
        return load_http(path_or_url);
    }

    let path = Path::new(path_or_url);
    Ok(std::fs::read_to_string(path)?)
}

pub fn load_http(url: &str) -> Result<String> {
    let resp = reqwest::blocking::get(url)?;

    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow!("{}", status).context(format!("Failed to load url `{}`", url)));
    }

    Ok(resp.text()?)
}