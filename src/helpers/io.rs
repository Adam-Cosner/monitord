/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Contains helpers for reading from and writing to files

use rustix::fd::{AsFd, AsRawFd, BorrowedFd};
use rustix::fs::{Mode, OFlags};

pub fn read_bin(fd: BorrowedFd) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    let chunk_size = 4096;

    for _ in 0..4096 {
        let start = buf.len();
        buf.resize(start + chunk_size, 0u8);

        let read_slice = &mut buf[start..];
        match rustix::io::read(fd, read_slice) {
            Ok(0) => {
                buf.truncate(start);
                break;
            }
            Ok(bytes_read) => {
                buf.truncate(start + bytes_read);
            }
            Err(rustix::io::Errno::INTR) => {
                // sys call interrupted, try again
                buf.truncate(start);
                continue;
            }
            Err(e) => {
                tracing::warn!("read_bin: read error: {}", e);
                return None;
            }
        }
    }
    Some(buf)
}

pub fn write_bin(fd: BorrowedFd, buf: &[u8]) -> anyhow::Result<()> {
    // Allow retries in case of syscall interrupts
    for _ in 0..16 {
        match rustix::io::write(fd, buf) {
            Ok(bytes_written) => {
                if bytes_written != buf.len() {
                    anyhow::bail!("write_bin: write was not completed!");
                }
                tracing::trace!(
                    "successfully wrote {bytes_written} bytes to fd {}",
                    fd.as_raw_fd()
                );
                break;
            }
            Err(rustix::io::Errno::INTR) => {
                continue;
            }
            Err(e) => {
                anyhow::bail!("write_bin: write error: {}", e);
            }
        }
    }
    Ok(())
}

/// Reads a string from a given fd, trimming whitespace and converting to a `String`.
pub fn read_string(fd: BorrowedFd) -> Option<String> {
    read_bin(fd).map(|buf| String::from_utf8_lossy(buf.as_slice()).trim().to_string())
}

/// Writes a string to a given fd
pub fn write_string(fd: BorrowedFd, text: &str) -> anyhow::Result<()> {
    write_bin(fd, text.as_bytes())
}

/// Reads a string from a given path relative to fd, trimming whitespace and converting to a `String`.
pub fn readat_string(fd: BorrowedFd, path: &str) -> Option<String> {
    rustix::fs::openat(fd, path, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())
        .ok()
        .and_then(|fd| read_string(fd.as_fd()))
}

/// Reads a string from a given path, trimming whitespace and converting to a `String`.
pub fn read_string_path<P: rustix::path::Arg>(path: P) -> Option<String> {
    rustix::fs::open(path, OFlags::RDONLY | OFlags::CLOEXEC, Mode::empty())
        .ok()
        .and_then(|fd| read_string(fd.as_fd()))
}

/// Reads a 32-bit unsigned integer from a given fd.
pub fn read_u32(fd: BorrowedFd) -> Option<u32> {
    read_string(fd).and_then(|s| s.parse::<u32>().ok())
}

/// Reads a 32-bit unsigned integer from a given path relative to fd.
pub fn readat_u32(fd: BorrowedFd, path: &str) -> Option<u32> {
    readat_string(fd, path).and_then(|s| s.parse::<u32>().ok())
}

/// Reads a 32-bit unsigned integer from a given path.
pub fn read_u32_path<P: rustix::path::Arg>(path: P) -> Option<u32> {
    read_string_path(path).and_then(|s| s.parse::<u32>().ok())
}

/// Reads a 64-bit unsigned integer from a given fd.
pub fn read_u64(fd: BorrowedFd) -> Option<u64> {
    read_string(fd).and_then(|s| s.parse::<u64>().ok())
}

/// Reads a 64-bit unsigned integer from a given path relative to fd.
pub fn readat_u64(fd: BorrowedFd, path: &str) -> Option<u64> {
    readat_string(fd, path).and_then(|s| s.parse::<u64>().ok())
}

#[allow(dead_code)]
/// Reads a 64-bit unsigned integer from a given path.
pub fn read_u64_path<P: rustix::path::Arg>(path: P) -> Option<u64> {
    read_string_path(path).and_then(|s| s.parse::<u64>().ok())
}

#[allow(dead_code)]
/// Reads a hexadecimal value from a given fd, converting it to a `u64`.
pub fn read_hex(fd: BorrowedFd) -> Option<u64> {
    read_string(fd)
        .as_ref()
        .and_then(|s| s.strip_prefix("0x"))
        .and_then(|s| u64::from_str_radix(&s, 16).ok())
}

#[allow(dead_code)]
/// Reads a hexadecimal value from a given path relative to fd, converting it to a `u64`.
pub fn readat_hex(fd: BorrowedFd, path: &str) -> Option<u64> {
    readat_string(fd, path)
        .as_ref()
        .and_then(|s| s.strip_prefix("0x"))
        .and_then(|s| u64::from_str_radix(&s, 16).ok())
}

#[allow(dead_code)]
/// Reads a hexadecimal value from a given path, converting it to a `u64`.
pub fn read_hex_path<P: rustix::path::Arg>(path: P) -> Option<u64> {
    read_string_path(path)
        .as_ref()
        .and_then(|s| s.strip_prefix("0x"))
        .and_then(|s| u64::from_str_radix(&s, 16).ok())
}
