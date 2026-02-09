//! Unified error types for mu-epub
//!
//! Provides a top-level `EpubError` that wraps module-specific errors,
//! plus `From` impls so `?` works across module boundaries.

extern crate alloc;

use alloc::string::{String, ToString};
use core::fmt;

/// Top-level error type for mu-epub operations
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EpubError {
    /// ZIP archive error
    Zip(ZipError),
    /// XML/XHTML parsing error
    Parse(String),
    /// Invalid EPUB structure (missing required files, broken references, etc.)
    InvalidEpub(String),
    /// Navigation parsing error
    Navigation(String),
    /// CSS parsing error
    Css(String),
    /// I/O error (description only, since `std::io::Error` is not `Clone`)
    Io(String),
    /// Chapter index requested is out of bounds
    ChapterOutOfBounds {
        /// Requested chapter index.
        index: usize,
        /// Total number of chapters available.
        chapter_count: usize,
    },
    /// Spine references a manifest item that does not exist
    ManifestItemMissing {
        /// Missing manifest `id` referenced by spine `idref`.
        idref: String,
    },
    /// Chapter content could not be decoded as UTF-8
    ChapterNotUtf8 {
        /// Chapter href/path in the EPUB archive.
        href: String,
    },
}

impl fmt::Display for EpubError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpubError::Zip(kind) => write!(f, "ZIP error: {}", kind),
            EpubError::Parse(msg) => write!(f, "Parse error: {}", msg),
            EpubError::InvalidEpub(msg) => write!(f, "Invalid EPUB: {}", msg),
            EpubError::Navigation(msg) => write!(f, "Navigation error: {}", msg),
            EpubError::Css(msg) => write!(f, "CSS error: {}", msg),
            EpubError::Io(msg) => write!(f, "I/O error: {}", msg),
            EpubError::ChapterOutOfBounds {
                index,
                chapter_count,
            } => write!(
                f,
                "Chapter index {} out of bounds (chapter count: {})",
                index, chapter_count
            ),
            EpubError::ManifestItemMissing { idref } => {
                write!(f, "Spine item '{}' does not exist in manifest", idref)
            }
            EpubError::ChapterNotUtf8 { href } => {
                write!(f, "Chapter content is not valid UTF-8: {}", href)
            }
        }
    }
}

/// ZIP-specific error variants
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ZipErrorKind {
    /// File not found in archive
    FileNotFound,
    /// Invalid ZIP format
    InvalidFormat,
    /// Unsupported compression method
    UnsupportedCompression,
    /// Decompression failed
    DecompressError,
    /// CRC32 mismatch
    CrcMismatch,
    /// I/O error during ZIP operations
    IoError,
    /// Central directory full (exceeded max entries)
    CentralDirFull,
    /// Buffer too small for decompressed content
    BufferTooSmall,
    /// File exceeds maximum allowed size
    FileTooLarge,
    /// Invalid or missing mimetype file
    InvalidMimetype(String),
    /// ZIP64 structures are present but unsupported
    UnsupportedZip64,
}

/// Public ZIP error type alias used across the crate API.
pub type ZipError = ZipErrorKind;

impl fmt::Display for ZipErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZipErrorKind::FileNotFound => write!(f, "file not found in archive"),
            ZipErrorKind::InvalidFormat => write!(f, "invalid ZIP format"),
            ZipErrorKind::UnsupportedCompression => write!(f, "unsupported compression method"),
            ZipErrorKind::DecompressError => write!(f, "decompression failed"),
            ZipErrorKind::CrcMismatch => write!(f, "CRC32 checksum mismatch"),
            ZipErrorKind::IoError => write!(f, "I/O error"),
            ZipErrorKind::CentralDirFull => write!(f, "central directory full"),
            ZipErrorKind::BufferTooSmall => write!(f, "buffer too small"),
            ZipErrorKind::FileTooLarge => write!(f, "file too large"),
            ZipErrorKind::InvalidMimetype(msg) => write!(f, "invalid mimetype: {}", msg),
            ZipErrorKind::UnsupportedZip64 => write!(f, "ZIP64 is not supported"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for EpubError {}

#[cfg(feature = "std")]
impl std::error::Error for ZipErrorKind {}

impl From<crate::tokenizer::TokenizeError> for EpubError {
    fn from(err: crate::tokenizer::TokenizeError) -> Self {
        EpubError::Parse(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epub_error_display() {
        let err = EpubError::Parse("bad xml".into());
        assert_eq!(format!("{}", err), "Parse error: bad xml");
    }

    #[test]
    fn test_zip_error_kind_debug() {
        let kind = ZipErrorKind::FileNotFound;
        assert_eq!(format!("{:?}", kind), "FileNotFound");
    }

    #[test]
    fn test_invalid_mimetype_error() {
        let err = EpubError::Zip(ZipErrorKind::InvalidMimetype("wrong content type".into()));
        let display = format!("{}", err);
        assert!(display.contains("ZIP error"));
    }
}
