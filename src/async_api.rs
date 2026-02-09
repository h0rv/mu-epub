//! Optional async helpers for high-level EPUB opening.
//!
//! This module is available with the `async` feature.

extern crate alloc;

use alloc::vec::Vec;
use core::result::Result;
use std::io::Cursor;
use std::path::Path;

use crate::book::{EpubBook, EpubBookOptions};
use crate::error::EpubError;

/// Read an EPUB file asynchronously and open it as an `EpubBook`.
///
/// This helper reads the file into memory and uses `EpubBook::from_reader`.
pub async fn open_epub_file_async<P: AsRef<Path>>(
    path: P,
) -> Result<EpubBook<Cursor<Vec<u8>>>, EpubError> {
    open_epub_file_async_with_options(path, EpubBookOptions::default()).await
}

/// Read an EPUB file asynchronously and open it as an `EpubBook` with options.
pub async fn open_epub_file_async_with_options<P: AsRef<Path>>(
    path: P,
    options: EpubBookOptions,
) -> Result<EpubBook<Cursor<Vec<u8>>>, EpubError> {
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| EpubError::Io(e.to_string()))?;
    EpubBook::from_reader_with_options(Cursor::new(bytes), options)
}
