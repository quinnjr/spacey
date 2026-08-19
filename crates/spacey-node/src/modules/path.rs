// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `path` module implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

/// Create the path module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // path.sep - path segment separator
    exports.insert("sep".to_string(), Value::String(MAIN_SEPARATOR.to_string()));

    // path.delimiter - path list delimiter (: on Unix, ; on Windows)
    exports.insert(
        "delimiter".to_string(),
        Value::String(if cfg!(windows) { ";" } else { ":" }.to_string()),
    );

    // path.posix - POSIX-specific implementation
    exports.insert("posix".to_string(), create_posix_module());

    // path.win32 - Windows-specific implementation
    exports.insert("win32".to_string(), create_win32_module());

    Value::NativeObject(exports)
}

/// Create POSIX path module
fn create_posix_module() -> Value {
    let mut exports = HashMap::new();
    exports.insert("sep".to_string(), Value::String("/".to_string()));
    exports.insert("delimiter".to_string(), Value::String(":".to_string()));
    Value::NativeObject(exports)
}

/// Create Win32 path module
fn create_win32_module() -> Value {
    let mut exports = HashMap::new();
    exports.insert("sep".to_string(), Value::String("\\".to_string()));
    exports.insert("delimiter".to_string(), Value::String(";".to_string()));
    Value::NativeObject(exports)
}

/// path.basename(path, ext?)
pub fn basename(path: &str, ext: Option<&str>) -> String {
    let p = Path::new(path);
    let name = p
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    if let Some(ext) = ext {
        if name.ends_with(ext) {
            name[..name.len() - ext.len()].to_string()
        } else {
            name
        }
    } else {
        name
    }
}

/// path.dirname(path)
pub fn dirname(path: &str) -> String {
    let p = Path::new(path);
    p.parent()
        .map(|s| {
            let dir = s.to_string_lossy().to_string();
            if dir.is_empty() { ".".to_string() } else { dir }
        })
        .unwrap_or_else(|| ".".to_string())
}

/// path.extname(path)
pub fn extname(path: &str) -> String {
    let p = Path::new(path);
    p.extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default()
}

/// path.isAbsolute(path)
pub fn is_absolute(path: &str) -> bool {
    Path::new(path).is_absolute()
}

/// path.join(...paths)
pub fn join(paths: &[&str]) -> String {
    let mut result = PathBuf::new();
    for p in paths {
        if Path::new(p).is_absolute() {
            result = PathBuf::from(p);
        } else {
            result.push(p);
        }
    }
    normalize(&result.to_string_lossy())
}

/// path.normalize(path)
pub fn normalize(path: &str) -> String {
    let mut components: Vec<&str> = Vec::new();
    let is_absolute = path.starts_with('/') || (cfg!(windows) && path.chars().nth(1) == Some(':'));

    for component in path.split(['/', '\\']) {
        match component {
            "" | "." => continue,
            ".." => {
                if !components.is_empty() && components.last() != Some(&"..") {
                    components.pop();
                } else if !is_absolute {
                    components.push("..");
                }
            }
            c => components.push(c),
        }
    }

    let sep = if cfg!(windows) { "\\" } else { "/" };
    let result = components.join(sep);

    if is_absolute {
        if cfg!(windows) && path.chars().nth(1) == Some(':') {
            // Windows absolute path with drive letter
            format!("{}{}{}", &path[..2], sep, result)
        } else {
            format!("{}{}", sep, result)
        }
    } else if result.is_empty() {
        ".".to_string()
    } else {
        result
    }
}

/// path.parse(path)
pub fn parse(path: &str) -> ParsedPath {
    let p = Path::new(path);

    let root = if p.is_absolute() {
        if cfg!(windows) && path.chars().nth(1) == Some(':') {
            path[..3].to_string()
        } else {
            "/".to_string()
        }
    } else {
        String::new()
    };

    let dir = dirname(path);
    let base = basename(path, None);
    let ext = extname(path);
    let name = if ext.is_empty() {
        base.clone()
    } else {
        base[..base.len() - ext.len()].to_string()
    };

    ParsedPath {
        root,
        dir,
        base,
        ext,
        name,
    }
}

/// Parsed path object
#[derive(Debug, Clone)]
pub struct ParsedPath {
    /// Root (e.g., "/" or "C:\\")
    pub root: String,
    /// Directory (e.g., "/home/user")
    pub dir: String,
    /// Base name with extension (e.g., "file.txt")
    pub base: String,
    /// Extension including dot (e.g., ".txt")
    pub ext: String,
    /// Name without extension (e.g., "file")
    pub name: String,
}

impl ParsedPath {
    /// Convert to a Value for JavaScript
    pub fn to_value(&self) -> Value {
        let mut obj = HashMap::new();
        obj.insert("root".to_string(), Value::String(self.root.clone()));
        obj.insert("dir".to_string(), Value::String(self.dir.clone()));
        obj.insert("base".to_string(), Value::String(self.base.clone()));
        obj.insert("ext".to_string(), Value::String(self.ext.clone()));
        obj.insert("name".to_string(), Value::String(self.name.clone()));
        Value::NativeObject(obj)
    }
}

/// path.format(pathObject)
pub fn format(parsed: &ParsedPath) -> String {
    let dir = if parsed.dir.is_empty() {
        &parsed.root
    } else {
        &parsed.dir
    };

    let base = if parsed.base.is_empty() {
        format!("{}{}", parsed.name, parsed.ext)
    } else {
        parsed.base.clone()
    };

    if dir.is_empty() {
        base
    } else {
        let sep = if cfg!(windows) { "\\" } else { "/" };
        if dir.ends_with(sep) {
            format!("{}{}", dir, base)
        } else {
            format!("{}{}{}", dir, sep, base)
        }
    }
}

/// path.relative(from, to)
pub fn relative(from: &str, to: &str) -> String {
    let from_path = PathBuf::from(normalize(from));
    let to_path = PathBuf::from(normalize(to));

    // Make both absolute
    let from_abs = if from_path.is_absolute() {
        from_path
    } else {
        std::env::current_dir().unwrap_or_default().join(from_path)
    };

    let to_abs = if to_path.is_absolute() {
        to_path
    } else {
        std::env::current_dir().unwrap_or_default().join(to_path)
    };

    // Find common prefix
    let from_components: Vec<_> = from_abs.components().collect();
    let to_components: Vec<_> = to_abs.components().collect();

    let mut common_len = 0;
    for (a, b) in from_components.iter().zip(to_components.iter()) {
        if a == b {
            common_len += 1;
        } else {
            break;
        }
    }

    // Build relative path
    let mut result = PathBuf::new();

    // Add ".." for each remaining component in "from"
    for _ in common_len..from_components.len() {
        result.push("..");
    }

    // Add remaining components from "to"
    for component in to_components.iter().skip(common_len) {
        result.push(component);
    }

    let result_str = result.to_string_lossy().to_string();
    if result_str.is_empty() {
        ".".to_string()
    } else {
        result_str
    }
}

/// path.resolve(...paths)
pub fn resolve(paths: &[&str]) -> String {
    let mut result = std::env::current_dir().unwrap_or_default();

    for p in paths {
        let path = Path::new(p);
        if path.is_absolute() {
            result = path.to_path_buf();
        } else {
            result.push(path);
        }
    }

    normalize(&result.to_string_lossy())
}

/// path.toNamespacedPath(path) - Windows only, returns path unchanged on POSIX
pub fn to_namespaced_path(path: &str) -> String {
    if cfg!(windows) && Path::new(path).is_absolute() {
        // Convert to extended-length path
        format!("\\\\?\\{}", path.replace('/', "\\"))
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basename() {
        assert_eq!(basename("/foo/bar/baz.txt", None), "baz.txt");
        assert_eq!(basename("/foo/bar/baz.txt", Some(".txt")), "baz");
        assert_eq!(basename("/foo/bar/", None), "bar");
    }

    #[test]
    fn test_dirname() {
        assert_eq!(dirname("/foo/bar/baz.txt"), "/foo/bar");
        assert_eq!(dirname("/foo/bar"), "/foo");
        assert_eq!(dirname("foo"), ".");
    }

    #[test]
    fn test_extname() {
        assert_eq!(extname("file.txt"), ".txt");
        assert_eq!(extname("file.tar.gz"), ".gz");
        assert_eq!(extname("file"), "");
        assert_eq!(extname(".hidden"), "");
    }

    #[test]
    fn test_join() {
        assert_eq!(join(&["foo", "bar", "baz"]), "foo/bar/baz");
        assert_eq!(join(&["/foo", "bar", "baz"]), "/foo/bar/baz");
        assert_eq!(join(&["foo", "../bar"]), "bar");
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("/foo/bar//baz/asdf/quux/.."), "/foo/bar/baz/asdf");
        assert_eq!(normalize("foo/bar/../baz"), "foo/baz");
    }

    #[test]
    fn test_parse() {
        let parsed = parse("/home/user/file.txt");
        assert_eq!(parsed.root, "/");
        assert_eq!(parsed.dir, "/home/user");
        assert_eq!(parsed.base, "file.txt");
        assert_eq!(parsed.ext, ".txt");
        assert_eq!(parsed.name, "file");
    }

    #[test]
    fn test_is_absolute() {
        assert!(is_absolute("/foo/bar"));
        assert!(!is_absolute("foo/bar"));
        assert!(!is_absolute("./foo"));
    }
}
