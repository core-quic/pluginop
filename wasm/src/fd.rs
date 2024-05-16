//! Plugin-side interface to file system API.

use std::{
    io::{Read, Write},
    path::Path,
};

use pluginop_common::{WASMLen, WASMPtr};
use std::convert::TryFrom;

extern "C" {
    fn create_file_from_plugin(path_ptr: u32, path_len: u32) -> i64;
    fn write_file_from_plugin(fd: i64, ptr: u32, len: u32) -> i64;
}

pub enum FileDescriptorType {
    File(i64),
    Network,
}

/// A structure enabling a plugin to read from or write to an external entity, whether it is using
/// the network (a.k.a. a socket) or is local to the host (a.k.a. a file).
pub struct FileDescriptor {
    fd: FileDescriptorType,
}

impl FileDescriptor {
    pub fn open<P: AsRef<Path>>(_path: P) -> std::io::Result<Self> {
        todo!()
    }

    pub fn create(path: &str) -> std::io::Result<Self> {
        match unsafe { create_file_from_plugin(path.as_ptr() as WASMPtr, path.len() as WASMLen) } {
            fd if fd >= 0 => Ok(FileDescriptor {
                fd: FileDescriptorType::File(fd),
            }),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot create file",
            )),
        }
    }
}

impl Read for FileDescriptor {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

impl Write for FileDescriptor {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.fd {
            FileDescriptorType::File(fd) => {
                match u32::try_from(unsafe {
                    write_file_from_plugin(fd, buf.as_ptr() as u32, buf.len() as u32)
                }) {
                    Ok(written) => Ok(written as usize),
                    Err(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "error when writing",
                    )),
                }
            }
            FileDescriptorType::Network => todo!("write not implemented on network"),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // crate::print("Plugin calling flush, why?");
        Ok(())
    }
}
