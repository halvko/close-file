//! Allows to close a file without silently dropping errors
//!
//! Errors can happen when closing a file, indicating that the file was not (completely) written.
//! The standard library currently simply discards such error when the std::io::File goes out of
//! scope.
//!
//! This crate allows to close the file and handle potential errors.
//!
//! ```
//! use close_file::Closable;
//! use std::io::Write;
//!
//! let mut f = std::fs::File::create("temp").unwrap();
//! f.write_all("Hello, world!".as_bytes()).unwrap();
//! if !f.close().is_ok() {
//!     // crash, print error and/or try writing the file again
//! }
//! ```
//!
//! The close() function consumes the File. If the operation failed, an error containing the
//! underlying I/O error and the file descriptor/handle of the file is returned. Depending on your
//! system and the error, closing the file may be retried, but in most cases the best solution is
//! to try to rewrite the file.
#[cfg(unix)]
use std::os::fd::RawFd;

#[cfg(windows)]
use std::os::windows::io::RawHandle;

use std::io;
use std::fmt;

/// Wraps any I/O error that can happen while closing a file
pub struct CloseError {
    io_error: io::Error,
    #[cfg(unix)]
    fd: RawFd,
    #[cfg(windows)]
    handle: RawHandle,
}

impl CloseError {
    /// Returns the file descriptor assigned to the file
    ///
    /// This should only be used in very rare cases. Check you OS documentation before use.
    /// 
    /// OBS: This function is OS specific for unix
    #[cfg(unix)]
    pub fn raw_fd(&self) -> RawFd {
        self.fd
    }

    /// Returns the file descriptor assigned to the file
    ///
    /// This should only be used in very rare cases. Check you OS documentation before use.
    /// 
    /// OBS: This function is OS specific for windows systems
    #[cfg(windows)]
    pub fn raw_handle(&self) -> RawHandle {
        self.handle
    }

    /// Returns the error produced when the file was closed.
    pub fn as_io_error(&self) -> &io::Error {
        &self.io_error
    }
}

impl std::error::Error for CloseError {}

pub trait Closable {
    fn close(self) -> Result<(), CloseError>;
}

impl fmt::Display for CloseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.io_error, f)
    }
}

impl fmt::Debug for CloseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.io_error, f)
    }
}

#[cfg(unix)]
mod imp {
    use std::os::unix::prelude::*;
    use std::{io, fs};
    use crate::CloseError;

    impl crate::Closable for fs::File {
        fn close(self) -> Result<(), CloseError> {
            let fd = self.into_raw_fd();
            let rc = unsafe {
                libc::close(fd)
            };
            if rc == -1 {
                Ok(())
            } else {
                Err(CloseError { io_error: io::Error::last_os_error(), fd })
            }
        }
    }
}

#[cfg(windows)]
mod imp {
    use std::os::windows::prelude::*;
    use std::{io, fs};
    use crate::CloseError;

    impl crate::Closable for fs::File {
        fn close(self) -> Result<(), CloseError> {
            let handle = self.into_raw_handle();
            let rc = unsafe {
                kernel32::CloseHandle(handle)
            };
            if rc != 0 {
                Ok(())
            } else {
                Err(CloseError { io_error: io::Error::last_os_error(), handle })
            }
        }
    }
}
