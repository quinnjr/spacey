// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Joseph R. Quinn

//! Node.js `os` module implementation

use spacey_spidermonkey::Value;
use std::collections::HashMap;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

/// Create the os module exports
pub fn create_module() -> Value {
    let mut exports = HashMap::new();

    // os.EOL - end of line character
    exports.insert(
        "EOL".to_string(),
        Value::String(if cfg!(windows) { "\r\n" } else { "\n" }.to_string()),
    );

    // os.constants
    exports.insert("constants".to_string(), create_constants());

    // os.devNull
    exports.insert(
        "devNull".to_string(),
        Value::String(
            if cfg!(windows) {
                "\\\\.\\nul"
            } else {
                "/dev/null"
            }
            .to_string(),
        ),
    );

    Value::NativeObject(exports)
}

/// Create os.constants
fn create_constants() -> Value {
    let mut constants = HashMap::new();

    // Signal constants
    let mut signals = HashMap::new();
    signals.insert("SIGHUP".to_string(), Value::Number(1.0));
    signals.insert("SIGINT".to_string(), Value::Number(2.0));
    signals.insert("SIGQUIT".to_string(), Value::Number(3.0));
    signals.insert("SIGILL".to_string(), Value::Number(4.0));
    signals.insert("SIGTRAP".to_string(), Value::Number(5.0));
    signals.insert("SIGABRT".to_string(), Value::Number(6.0));
    signals.insert("SIGBUS".to_string(), Value::Number(7.0));
    signals.insert("SIGFPE".to_string(), Value::Number(8.0));
    signals.insert("SIGKILL".to_string(), Value::Number(9.0));
    signals.insert("SIGUSR1".to_string(), Value::Number(10.0));
    signals.insert("SIGSEGV".to_string(), Value::Number(11.0));
    signals.insert("SIGUSR2".to_string(), Value::Number(12.0));
    signals.insert("SIGPIPE".to_string(), Value::Number(13.0));
    signals.insert("SIGALRM".to_string(), Value::Number(14.0));
    signals.insert("SIGTERM".to_string(), Value::Number(15.0));
    constants.insert("signals".to_string(), Value::NativeObject(signals));

    // Error constants
    let mut errno = HashMap::new();
    errno.insert("EPERM".to_string(), Value::Number(1.0));
    errno.insert("ENOENT".to_string(), Value::Number(2.0));
    errno.insert("ESRCH".to_string(), Value::Number(3.0));
    errno.insert("EINTR".to_string(), Value::Number(4.0));
    errno.insert("EIO".to_string(), Value::Number(5.0));
    errno.insert("ENXIO".to_string(), Value::Number(6.0));
    errno.insert("ENOEXEC".to_string(), Value::Number(8.0));
    errno.insert("EBADF".to_string(), Value::Number(9.0));
    errno.insert("ECHILD".to_string(), Value::Number(10.0));
    errno.insert("EAGAIN".to_string(), Value::Number(11.0));
    errno.insert("ENOMEM".to_string(), Value::Number(12.0));
    errno.insert("EACCES".to_string(), Value::Number(13.0));
    errno.insert("EFAULT".to_string(), Value::Number(14.0));
    errno.insert("EBUSY".to_string(), Value::Number(16.0));
    errno.insert("EEXIST".to_string(), Value::Number(17.0));
    errno.insert("ENODEV".to_string(), Value::Number(19.0));
    errno.insert("ENOTDIR".to_string(), Value::Number(20.0));
    errno.insert("EISDIR".to_string(), Value::Number(21.0));
    errno.insert("EINVAL".to_string(), Value::Number(22.0));
    errno.insert("EMFILE".to_string(), Value::Number(24.0));
    errno.insert("ENOTTY".to_string(), Value::Number(25.0));
    errno.insert("EFBIG".to_string(), Value::Number(27.0));
    errno.insert("ENOSPC".to_string(), Value::Number(28.0));
    errno.insert("ESPIPE".to_string(), Value::Number(29.0));
    errno.insert("EROFS".to_string(), Value::Number(30.0));
    errno.insert("EMLINK".to_string(), Value::Number(31.0));
    errno.insert("EPIPE".to_string(), Value::Number(32.0));
    errno.insert("EDOM".to_string(), Value::Number(33.0));
    errno.insert("ERANGE".to_string(), Value::Number(34.0));
    constants.insert("errno".to_string(), Value::NativeObject(errno));

    Value::NativeObject(constants)
}

/// os.arch()
pub fn arch() -> String {
    if cfg!(target_arch = "x86_64") {
        "x64".to_string()
    } else if cfg!(target_arch = "x86") {
        "ia32".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "arm64".to_string()
    } else if cfg!(target_arch = "arm") {
        "arm".to_string()
    } else if cfg!(target_arch = "powerpc64") {
        "ppc64".to_string()
    } else if cfg!(target_arch = "s390x") {
        "s390x".to_string()
    } else {
        std::env::consts::ARCH.to_string()
    }
}

/// os.platform()
pub fn platform() -> String {
    if cfg!(target_os = "windows") {
        "win32".to_string()
    } else if cfg!(target_os = "macos") {
        "darwin".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "freebsd") {
        "freebsd".to_string()
    } else if cfg!(target_os = "openbsd") {
        "openbsd".to_string()
    } else if cfg!(target_os = "android") {
        "android".to_string()
    } else {
        std::env::consts::OS.to_string()
    }
}

/// os.type()
pub fn os_type() -> String {
    if cfg!(target_os = "windows") {
        "Windows_NT".to_string()
    } else if cfg!(target_os = "macos") {
        "Darwin".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else {
        System::name().unwrap_or_else(|| "Unknown".to_string())
    }
}

/// os.release()
pub fn release() -> String {
    System::os_version().unwrap_or_else(|| "unknown".to_string())
}

/// os.version()
pub fn version() -> String {
    System::long_os_version().unwrap_or_else(|| "unknown".to_string())
}

/// os.hostname()
pub fn hostname() -> String {
    System::host_name().unwrap_or_else(|| "localhost".to_string())
}

/// os.homedir()
pub fn homedir() -> String {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| {
            std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_default()
        })
}

/// os.tmpdir()
pub fn tmpdir() -> String {
    std::env::temp_dir().to_string_lossy().to_string()
}

/// os.uptime()
pub fn uptime() -> u64 {
    System::uptime()
}

/// os.totalmem()
pub fn totalmem() -> u64 {
    let s =
        System::new_with_specifics(RefreshKind::new().with_memory(MemoryRefreshKind::everything()));
    s.total_memory()
}

/// os.freemem()
pub fn freemem() -> u64 {
    let s =
        System::new_with_specifics(RefreshKind::new().with_memory(MemoryRefreshKind::everything()));
    s.free_memory()
}

/// os.cpus()
pub fn cpus() -> Vec<Value> {
    let s = System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));

    s.cpus()
        .iter()
        .map(|cpu| {
            let mut info = HashMap::new();
            info.insert("model".to_string(), Value::String(cpu.brand().to_string()));
            info.insert("speed".to_string(), Value::Number(cpu.frequency() as f64));

            let mut times = HashMap::new();
            times.insert("user".to_string(), Value::Number(0.0));
            times.insert("nice".to_string(), Value::Number(0.0));
            times.insert("sys".to_string(), Value::Number(0.0));
            times.insert("idle".to_string(), Value::Number(0.0));
            times.insert("irq".to_string(), Value::Number(0.0));
            info.insert("times".to_string(), Value::NativeObject(times));

            Value::NativeObject(info)
        })
        .collect()
}

/// os.loadavg()
pub fn loadavg() -> [f64; 3] {
    let la = System::load_average();
    [la.one, la.five, la.fifteen]
}

/// os.networkInterfaces()
pub fn network_interfaces() -> HashMap<String, Vec<Value>> {
    // Basic implementation - real one would use system APIs
    HashMap::new()
}

/// os.userInfo()
pub fn user_info() -> Value {
    let mut info = HashMap::new();

    info.insert(
        "username".to_string(),
        Value::String(
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "unknown".to_string()),
        ),
    );

    info.insert("homedir".to_string(), Value::String(homedir()));

    info.insert(
        "shell".to_string(),
        Value::String(std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(windows) {
                "cmd.exe".to_string()
            } else {
                "/bin/sh".to_string()
            }
        })),
    );

    #[cfg(unix)]
    {
        info.insert(
            "uid".to_string(),
            Value::Number(nix::unistd::getuid().as_raw() as f64),
        );
        info.insert(
            "gid".to_string(),
            Value::Number(nix::unistd::getgid().as_raw() as f64),
        );
    }

    #[cfg(not(unix))]
    {
        info.insert("uid".to_string(), Value::Number(-1.0));
        info.insert("gid".to_string(), Value::Number(-1.0));
    }

    Value::NativeObject(info)
}

/// os.endianness()
pub fn endianness() -> &'static str {
    if cfg!(target_endian = "big") {
        "BE"
    } else {
        "LE"
    }
}

/// os.machine()
pub fn machine() -> String {
    std::env::consts::ARCH.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform() {
        let p = platform();
        assert!(!p.is_empty());
    }

    #[test]
    fn test_arch() {
        let a = arch();
        assert!(!a.is_empty());
    }

    #[test]
    fn test_homedir() {
        let h = homedir();
        assert!(!h.is_empty());
    }

    #[test]
    fn test_tmpdir() {
        let t = tmpdir();
        assert!(!t.is_empty());
    }

    #[test]
    fn test_hostname() {
        let h = hostname();
        assert!(!h.is_empty());
    }

    #[test]
    fn test_totalmem() {
        let mem = totalmem();
        assert!(mem > 0);
    }

    #[test]
    fn test_cpus() {
        let c = cpus();
        // Should have at least one CPU
        assert!(!c.is_empty());
    }
}
