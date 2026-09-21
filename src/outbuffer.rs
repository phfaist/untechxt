//! Output buffers: the [`OutBuffer`] trait that the encoder appends its
//! output to, and the adapters [`FmtOut`], for a [`core::fmt::Write`], and
//! `IoOut`, for a `std::io::Write` with the crate feature `std`.

use alloc::boxed::Box;
use alloc::string::String;

use crate::BoxError;

/// A destination that the encoder appends its output to.
///
/// [`String`] implements this trait, so the plain case costs nothing extra:
/// the error check can disappear when the compiler inlines the call. The
/// trait exists so that the output can stream, to a formatter with
/// [`FmtOut`] or to a file or a socket with `IoOut` (with the crate feature
/// `std`), without assembling the whole string first.
///
/// The error type is the fixed [`BoxError`], rather than an associated type
/// that would spread through every signature of the crate. An I/O error is
/// carried through unchanged, boxed only on the error path.
pub trait OutBuffer {
    /// Appends `s` to the output.
    ///
    /// # Errors
    ///
    /// Whatever the underlying destination reports. The encoder passes it
    /// on as [`EncodeError::Output`](crate::EncodeError::Output).
    fn push_str(&mut self, s: &str) -> Result<(), BoxError>;

    /// Appends the character `c` to the output. The default writes its UTF-8
    /// through [`push_str`](OutBuffer::push_str).
    ///
    /// # Errors
    ///
    /// As for [`push_str`](OutBuffer::push_str).
    fn push_char(&mut self, c: char) -> Result<(), BoxError> {
        let mut buf = [0u8; 4];
        self.push_str(c.encode_utf8(&mut buf))
    }
}

impl OutBuffer for String {
    fn push_str(&mut self, s: &str) -> Result<(), BoxError> {
        String::push_str(self, s);
        Ok(())
    }

    fn push_char(&mut self, c: char) -> Result<(), BoxError> {
        String::push(self, c);
        Ok(())
    }
}

/// An [`OutBuffer`] that writes to a [`core::fmt::Write`], such as a
/// [`core::fmt::Formatter`] or a `String`.
///
/// ```
/// use untechxt::outbuffer::{FmtOut, OutBuffer};
///
/// let mut out = FmtOut(String::new());
/// out.push_str("Caf").unwrap();
/// out.push_char('e').unwrap();
/// assert_eq!(out.0, "Cafe");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FmtOut<W: core::fmt::Write>(pub W);

impl<W: core::fmt::Write> OutBuffer for FmtOut<W> {
    fn push_str(&mut self, s: &str) -> Result<(), BoxError> {
        self.0.write_str(s).map_err(|err| -> BoxError { Box::new(err) })
    }

    fn push_char(&mut self, c: char) -> Result<(), BoxError> {
        self.0.write_char(c).map_err(|err| -> BoxError { Box::new(err) })
    }
}

/// An [`OutBuffer`] that writes to a [`std::io::Write`], such as a file, a
/// socket, or standard output. The output is written as UTF-8.
///
/// Wrap the writer in a [`std::io::BufWriter`] where each write would
/// otherwise reach the operating system, because the encoder writes in
/// small pieces.
///
/// Available only with the crate feature `std`.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IoOut<W: std::io::Write>(pub W);

#[cfg(feature = "std")]
impl<W: std::io::Write> OutBuffer for IoOut<W> {
    fn push_str(&mut self, s: &str) -> Result<(), BoxError> {
        self.0.write_all(s.as_bytes()).map_err(|err| -> BoxError { Box::new(err) })
    }
}
