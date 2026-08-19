// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `fs` module implementation
//!
//! Provides file system operations (sync and async).

use crate::error::{NodeError, Result};
use crate::globals::buffer::Buffer;
use spacey_spidermonkey::Value;
use std::collections::HashMap;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::SystemTime;

/// Create the fs module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // Constants
    let mut constants = HashMap::new();

    // File access constants
    constants.insert("F_OK".to_string(), Value::Number(0.0));
    constants.insert("R_OK".to_string(), Value::Number(4.0));
    constants.insert("W_OK".to_string(), Value::Number(2.0));
    constants.insert("X_OK".to_string(), Value::Number(1.0));

    // File open flags
    constants.insert("O_RDONLY".to_string(), Value::Number(0.0));
    constants.insert("O_WRONLY".to_string(), Value::Number(1.0));
    constants.insert("O_RDWR".to_string(), Value::Number(2.0));
    constants.insert("O_CREAT".to_string(), Value::Number(64.0));
    constants.insert("O_EXCL".to_string(), Value::Number(128.0));
    constants.insert("O_TRUNC".to_string(), Value::Number(512.0));
    constants.insert("O_APPEND".to_string(), Value::Number(1024.0));

    exports.insert("constants".to_string(), Value::NativeObject(constants));

    // fs.promises - async API using promises
    exports.insert("promises".to_string(), create_promises_module());

    Value::NativeObject(exports)
}

/// Create the fs.promises module
fn create_promises_module() -> Value {
    let exports = HashMap::new();
    // Async functions would be registered here
    Value::NativeObject(exports)
}

// ============================================================================
// Synchronous API
// ============================================================================

/// fs.readFileSync(path, options?)
pub fn read_file_sync(path: &str, encoding: Option<&str>) -> Result<Value> {
    let content = fs::read(path)?;

    match encoding {
        Some("utf8") | Some("utf-8") => {
            Ok(Value::String(String::from_utf8_lossy(&content).to_string()))
        }
        Some("base64") => Ok(Value::String(base64::Engine::encode(
            &base64::prelude::BASE64_STANDARD,
            &content,
        ))),
        Some("hex") => Ok(Value::String(hex::encode(&content))),
        Some(enc) => Err(NodeError::type_error(format!("Unknown encoding: {}", enc))),
        None => {
            // Return as Buffer (represented as array-like object)
            let mut arr: HashMap<String, Value> = content
                .iter()
                .enumerate()
                .map(|(i, &b)| (i.to_string(), Value::Number(b as f64)))
                .collect();
            arr.insert("length".to_string(), Value::Number(content.len() as f64));
            Ok(Value::NativeObject(arr))
        }
    }
}

/// fs.writeFileSync(path, data, options?)
pub fn write_file_sync(path: &str, data: &[u8], options: Option<WriteOptions>) -> Result<()> {
    let options = options.unwrap_or_default();

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(!options.flag.contains('a'))
        .append(options.flag.contains('a'))
        .open(path)?;

    file.write_all(data)?;

    if let Some(mode) = options.mode {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }

    Ok(())
}

/// fs.appendFileSync(path, data, options?)
pub fn append_file_sync(path: &str, data: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;

    file.write_all(data)?;
    Ok(())
}

/// fs.existsSync(path)
pub fn exists_sync(path: &str) -> bool {
    Path::new(path).exists()
}

/// fs.accessSync(path, mode?)
pub fn access_sync(path: &str, mode: Option<u32>) -> Result<()> {
    let path = Path::new(path);
    let mode = mode.unwrap_or(0); // F_OK

    if !path.exists() {
        return Err(NodeError::Fs(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "ENOENT: no such file or directory",
        )));
    }

    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();

    // Check permissions
    if mode & 4 != 0 {
        // R_OK
        // Simplified check - real implementation would check actual read permission
    }
    if mode & 2 != 0 {
        // W_OK
        if permissions.readonly() {
            return Err(NodeError::Fs(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "EACCES: permission denied",
            )));
        }
    }

    Ok(())
}

/// fs.statSync(path)
pub fn stat_sync(path: &str) -> Result<Stats> {
    let metadata = fs::metadata(path)?;
    Ok(Stats::from_metadata(&metadata, path))
}

/// fs.lstatSync(path) - like stat but doesn't follow symlinks
pub fn lstat_sync(path: &str) -> Result<Stats> {
    let metadata = fs::symlink_metadata(path)?;
    Ok(Stats::from_metadata(&metadata, path))
}

/// fs.mkdirSync(path, options?)
pub fn mkdir_sync(path: &str, options: Option<MkdirOptions>) -> Result<()> {
    let options = options.unwrap_or_default();

    if options.recursive {
        fs::create_dir_all(path)?;
    } else {
        fs::create_dir(path)?;
    }

    if let Some(mode) = options.mode {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }

    Ok(())
}

/// fs.rmdirSync(path, options?)
pub fn rmdir_sync(path: &str, options: Option<RmdirOptions>) -> Result<()> {
    let options = options.unwrap_or_default();

    if options.recursive {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_dir(path)?;
    }

    Ok(())
}

/// fs.rmSync(path, options?)
pub fn rm_sync(path: &str, options: Option<RmOptions>) -> Result<()> {
    let options = options.unwrap_or_default();
    let path = Path::new(path);

    if !path.exists() {
        if options.force {
            return Ok(());
        }
        return Err(NodeError::Fs(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "ENOENT: no such file or directory",
        )));
    }

    if path.is_dir() {
        if options.recursive {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_dir(path)?;
        }
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

/// fs.readdirSync(path, options?)
pub fn readdir_sync(path: &str, options: Option<ReaddirOptions>) -> Result<Vec<Value>> {
    let options = options.unwrap_or_default();
    let entries = fs::read_dir(path)?;

    let mut result = Vec::new();
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if options.with_file_types {
            let file_type = entry.file_type()?;
            let mut dirent = HashMap::new();
            dirent.insert("name".to_string(), Value::String(name));
            dirent.insert("isFile".to_string(), Value::Boolean(file_type.is_file()));
            dirent.insert(
                "isDirectory".to_string(),
                Value::Boolean(file_type.is_dir()),
            );
            dirent.insert(
                "isSymbolicLink".to_string(),
                Value::Boolean(file_type.is_symlink()),
            );
            result.push(Value::NativeObject(dirent));
        } else {
            result.push(Value::String(name));
        }
    }

    Ok(result)
}

/// fs.renameSync(oldPath, newPath)
pub fn rename_sync(old_path: &str, new_path: &str) -> Result<()> {
    fs::rename(old_path, new_path)?;
    Ok(())
}

/// fs.copyFileSync(src, dest, mode?)
pub fn copy_file_sync(src: &str, dest: &str, mode: Option<u32>) -> Result<()> {
    let mode = mode.unwrap_or(0);

    // COPYFILE_EXCL = 1 - fail if dest exists
    if mode & 1 != 0 && Path::new(dest).exists() {
        return Err(NodeError::Fs(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "EEXIST: file already exists",
        )));
    }

    fs::copy(src, dest)?;
    Ok(())
}

/// fs.unlinkSync(path)
pub fn unlink_sync(path: &str) -> Result<()> {
    fs::remove_file(path)?;
    Ok(())
}

/// fs.chmodSync(path, mode)
pub fn chmod_sync(path: &str, mode: u32) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

/// fs.symlinkSync(target, path, type?)
pub fn symlink_sync(target: &str, path: &str) -> Result<()> {
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, path)?;

    #[cfg(windows)]
    {
        let target_path = Path::new(target);
        if target_path.is_dir() {
            std::os::windows::fs::symlink_dir(target, path)?;
        } else {
            std::os::windows::fs::symlink_file(target, path)?;
        }
    }

    Ok(())
}

/// fs.readlinkSync(path)
pub fn readlink_sync(path: &str) -> Result<String> {
    let target = fs::read_link(path)?;
    Ok(target.to_string_lossy().to_string())
}

/// fs.realpathSync(path)
pub fn realpath_sync(path: &str) -> Result<String> {
    let real = fs::canonicalize(path)?;
    Ok(real.to_string_lossy().to_string())
}

/// fs.truncateSync(path, len?)
pub fn truncate_sync(path: &str, len: Option<u64>) -> Result<()> {
    let file = File::options().write(true).open(path)?;
    file.set_len(len.unwrap_or(0))?;
    Ok(())
}

// ============================================================================
// Types
// ============================================================================

/// File statistics
#[derive(Debug, Clone)]
pub struct Stats {
    pub dev: u64,
    pub ino: u64,
    pub mode: u32,
    pub nlink: u64,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u64,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,
    pub atime_ms: f64,
    pub mtime_ms: f64,
    pub ctime_ms: f64,
    pub birthtime_ms: f64,
    is_file: bool,
    is_directory: bool,
    is_symbolic_link: bool,
}

impl Stats {
    /// Create Stats from Metadata
    pub fn from_metadata(metadata: &Metadata, _path: &str) -> Self {
        let atime = metadata.accessed().ok();
        let mtime = metadata.modified().ok();
        let ctime = metadata.created().ok();

        fn to_ms(time: Option<SystemTime>) -> f64 {
            time.and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs_f64() * 1000.0)
                .unwrap_or(0.0)
        }

        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;

        Self {
            #[cfg(unix)]
            dev: metadata.dev(),
            #[cfg(not(unix))]
            dev: 0,

            #[cfg(unix)]
            ino: metadata.ino(),
            #[cfg(not(unix))]
            ino: 0,

            #[cfg(unix)]
            mode: metadata.mode(),
            #[cfg(not(unix))]
            mode: 0,

            #[cfg(unix)]
            nlink: metadata.nlink(),
            #[cfg(not(unix))]
            nlink: 0,

            #[cfg(unix)]
            uid: metadata.uid(),
            #[cfg(not(unix))]
            uid: 0,

            #[cfg(unix)]
            gid: metadata.gid(),
            #[cfg(not(unix))]
            gid: 0,

            #[cfg(unix)]
            rdev: metadata.rdev(),
            #[cfg(not(unix))]
            rdev: 0,

            size: metadata.len(),

            #[cfg(unix)]
            blksize: metadata.blksize(),
            #[cfg(not(unix))]
            blksize: 4096,

            #[cfg(unix)]
            blocks: metadata.blocks(),
            #[cfg(not(unix))]
            blocks: 0,

            atime_ms: to_ms(atime),
            mtime_ms: to_ms(mtime),
            ctime_ms: to_ms(ctime),
            birthtime_ms: to_ms(ctime),

            is_file: metadata.is_file(),
            is_directory: metadata.is_dir(),
            is_symbolic_link: metadata.file_type().is_symlink(),
        }
    }

    /// Convert to JavaScript Value
    pub fn to_value(&self) -> Value {
        let mut obj = HashMap::new();
        obj.insert("dev".to_string(), Value::Number(self.dev as f64));
        obj.insert("ino".to_string(), Value::Number(self.ino as f64));
        obj.insert("mode".to_string(), Value::Number(self.mode as f64));
        obj.insert("nlink".to_string(), Value::Number(self.nlink as f64));
        obj.insert("uid".to_string(), Value::Number(self.uid as f64));
        obj.insert("gid".to_string(), Value::Number(self.gid as f64));
        obj.insert("rdev".to_string(), Value::Number(self.rdev as f64));
        obj.insert("size".to_string(), Value::Number(self.size as f64));
        obj.insert("blksize".to_string(), Value::Number(self.blksize as f64));
        obj.insert("blocks".to_string(), Value::Number(self.blocks as f64));
        obj.insert("atimeMs".to_string(), Value::Number(self.atime_ms));
        obj.insert("mtimeMs".to_string(), Value::Number(self.mtime_ms));
        obj.insert("ctimeMs".to_string(), Value::Number(self.ctime_ms));
        obj.insert("birthtimeMs".to_string(), Value::Number(self.birthtime_ms));
        Value::NativeObject(obj)
    }

    pub fn is_file(&self) -> bool {
        self.is_file
    }

    pub fn is_directory(&self) -> bool {
        self.is_directory
    }

    pub fn is_symbolic_link(&self) -> bool {
        self.is_symbolic_link
    }
}

/// Options for writeFileSync
#[derive(Debug, Default)]
pub struct WriteOptions {
    pub encoding: Option<String>,
    pub mode: Option<u32>,
    pub flag: String,
}

impl WriteOptions {
    pub fn new() -> Self {
        Self {
            encoding: Some("utf8".to_string()),
            mode: Some(0o666),
            flag: "w".to_string(),
        }
    }
}

/// Options for mkdirSync
#[derive(Debug, Default)]
pub struct MkdirOptions {
    pub recursive: bool,
    pub mode: Option<u32>,
}

/// Options for rmdirSync
#[derive(Debug, Default)]
pub struct RmdirOptions {
    pub recursive: bool,
}

/// Options for rmSync
#[derive(Debug, Default)]
pub struct RmOptions {
    pub force: bool,
    pub recursive: bool,
}

/// Options for readdirSync
#[derive(Debug, Default)]
pub struct ReaddirOptions {
    pub encoding: Option<String>,
    pub with_file_types: bool,
    pub recursive: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_write_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let path_str = file_path.to_str().unwrap();

        // Write
        write_file_sync(path_str, b"hello world", None).unwrap();

        // Read
        let content = read_file_sync(path_str, Some("utf8")).unwrap();
        assert_eq!(content, Value::String("hello world".to_string()));
    }

    #[test]
    fn test_exists_sync() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("exists.txt");
        let path_str = file_path.to_str().unwrap();

        assert!(!exists_sync(path_str));
        write_file_sync(path_str, b"test", None).unwrap();
        assert!(exists_sync(path_str));
    }

    #[test]
    fn test_mkdir_rmdir() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("subdir");
        let path_str = sub_dir.to_str().unwrap();

        mkdir_sync(path_str, None).unwrap();
        assert!(exists_sync(path_str));

        rmdir_sync(path_str, None).unwrap();
        assert!(!exists_sync(path_str));
    }
}
