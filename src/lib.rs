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
//! f.write_all("Hello, world!".as_bytes());
//! f.close();
//! ```
//!
//! The close() function consumes the File. However, on Windows, a failed close operation may be
//! retried. For this case the returned CloseError contains the original File.

use std::{io, fs};
use std::fmt;

// TODO allows to close a file without silently discarding the error

/// Wraps any I/O error that can happen during closing the file
pub struct CloseError {
    previous: io::Error,
    file: Option<fs::File>,
}

impl CloseError {
    /// Returns the original file handle
    ///
    /// This contains the original File, but only if the close operation can be retried safely,
    /// otherwise it's None.
    pub fn into_file_handle(self) -> Option<fs::File> {
        self.file
    }

    pub fn unwrap(self) -> io::Error {
        self.previous
    }
}

impl std::error::Error for CloseError {}

pub trait Closable {
    fn close(self) -> Result<(), CloseError>;
}

impl fmt::Display for CloseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.previous, f)
    }
}

impl fmt::Debug for CloseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.previous, f)
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
            if unsafe { libc::close(fd) } != 0 {
                return Err(CloseError {
                    previous: io::Error::last_os_error(),
                    file: Some(unsafe { fs::File::from_raw_fd(fd) }),
                });
            }
            Ok(())
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
                Err(CloseError { previous: io::Error::last_os_error(), file: Some(unsafe { fs::File::from_raw_handle(handle) }) })
            }
        }
    }
}
