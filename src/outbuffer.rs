//! [`OutBuffer`]: where the encoder writes, and the adapters that carry its
//! output to a [`core::fmt::Write`] or, with the crate feature `std`, to a
//! `std::io::Write`.

use alloc::boxed::Box;
use alloc::string::String;

use crate::rule::BoxError;

/// Abstract output assembly: where an encoder appends what it writes.
///
/// [`String`] implements it, and so the plain case costs nothing: the error
/// check disappears when the compiler inlines the call. The trait exists so
/// that output can stream — to a formatter ([`FmtOut`]), to a file or a
/// socket (`IoOut`, with the crate feature `std`) — without assembling the
/// whole string first.
///
/// The error type is the fixed [`BoxError`], rather than an associated type
/// that would spread through every signature of the crate. An I/O error is
/// carried through as it is, boxed on the error path alone.
pub trait OutBuffer {
    /// Appends `s` to the output.
    ///
    /// # Errors
    ///
    /// Whatever the sink reports; the encoder passes it on as
    /// [`EncodeError::Output`](crate::EncodeError::Output).
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

/// An [`OutBuffer`] that writes to a [`core::fmt::Write`] — a
/// [`core::fmt::Formatter`], a `String`, anything that formats.
///
/// ```
/// use untechxt::{FmtOut, OutBuffer};
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

/// An [`OutBuffer`] that writes to a [`std::io::Write`] — a file, a socket,
/// standard output. The encoder's text is written as UTF-8.
///
/// Wrap the writer in a [`std::io::BufWriter`] where each write would
/// otherwise reach the operating system: the encoder writes in small pieces.
///
/// Only with the crate feature `std`.
#[cfg(feature = "std")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IoOut<W: std::io::Write>(pub W);

#[cfg(feature = "std")]
impl<W: std::io::Write> OutBuffer for IoOut<W> {
    fn push_str(&mut self, s: &str) -> Result<(), BoxError> {
        self.0.write_all(s.as_bytes()).map_err(|err| -> BoxError { Box::new(err) })
    }
}
