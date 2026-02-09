use mu_epub::{EpubBook, RenderPrep, RenderPrepError, RenderPrepOptions};
use std::fmt;
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::render_ir::{OverlayContent, OverlaySize, PaginationProfileId, RenderPage};
use crate::render_layout::{LayoutConfig, LayoutEngine};

/// Cancellation hook for long-running layout operations.
pub trait CancelToken {
    fn is_cancelled(&self) -> bool;
}

/// Never-cancel token for default call paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeverCancel;

impl CancelToken for NeverCancel {
    fn is_cancelled(&self) -> bool {
        false
    }
}

/// Runtime diagnostics from render preparation/layout.
#[derive(Clone, Debug, PartialEq)]
pub enum RenderDiagnostic {
    ReflowTimeMs(u32),
    Cancelled,
}

type DiagnosticSink = Arc<Mutex<Option<Box<dyn FnMut(RenderDiagnostic) + Send + 'static>>>>;

/// Render-engine options.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderEngineOptions {
    /// Prep options passed to `RenderPrep`.
    pub prep: RenderPrepOptions,
    /// Layout options used to produce pages.
    pub layout: LayoutConfig,
}

impl RenderEngineOptions {
    /// Build options for a target display size.
    pub fn for_display(width: i32, height: i32) -> Self {
        Self {
            prep: RenderPrepOptions::default(),
            layout: LayoutConfig::for_display(width, height),
        }
    }
}

/// Render engine for chapter -> page conversion.
#[derive(Clone)]
pub struct RenderEngine {
    opts: RenderEngineOptions,
    layout: LayoutEngine,
    diagnostic_sink: DiagnosticSink,
}

impl fmt::Debug for RenderEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderEngine")
            .field("opts", &self.opts)
            .field("layout", &self.layout)
            .finish_non_exhaustive()
    }
}

impl RenderEngine {
    /// Create a render engine.
    pub fn new(opts: RenderEngineOptions) -> Self {
        Self {
            layout: LayoutEngine::new(opts.layout),
            opts,
            diagnostic_sink: Arc::new(Mutex::new(None)),
        }
    }

    /// Register or replace the diagnostics sink.
    pub fn set_diagnostic_sink<F>(&mut self, sink: F)
    where
        F: FnMut(RenderDiagnostic) + Send + 'static,
    {
        if let Ok(mut slot) = self.diagnostic_sink.lock() {
            *slot = Some(Box::new(sink));
        }
    }

    fn emit_diagnostic(&self, diagnostic: RenderDiagnostic) {
        if let Ok(mut slot) = self.diagnostic_sink.lock() {
            if let Some(sink) = slot.as_mut() {
                sink(diagnostic);
            }
        }
    }

    /// Stable fingerprint for all layout-affecting settings.
    pub fn pagination_profile_id(&self) -> PaginationProfileId {
        let payload = format!("{:?}|{:?}", self.opts.prep, self.opts.layout);
        PaginationProfileId::from_bytes(payload.as_bytes())
    }

    /// Prepare and layout a chapter into render pages.
    pub fn prepare_chapter<R: std::io::Read + std::io::Seek>(
        &self,
        book: &mut EpubBook<R>,
        chapter_index: usize,
    ) -> Result<Vec<RenderPage>, RenderEngineError> {
        let mut pages = Vec::new();
        self.prepare_chapter_with_cancel(book, chapter_index, &NeverCancel, |page| {
            pages.push(page)
        })?;
        Ok(pages)
    }

    /// Prepare and layout a chapter and stream each page.
    pub fn prepare_chapter_with<R, F>(
        &self,
        book: &mut EpubBook<R>,
        chapter_index: usize,
        on_page: F,
    ) -> Result<(), RenderEngineError>
    where
        R: std::io::Read + std::io::Seek,
        F: FnMut(RenderPage),
    {
        self.prepare_chapter_with_cancel(book, chapter_index, &NeverCancel, on_page)
    }

    /// Prepare and layout a chapter while honoring cancellation.
    pub fn prepare_chapter_with_cancel<R, C, F>(
        &self,
        book: &mut EpubBook<R>,
        chapter_index: usize,
        cancel: &C,
        mut on_page: F,
    ) -> Result<(), RenderEngineError>
    where
        R: std::io::Read + std::io::Seek,
        C: CancelToken,
        F: FnMut(RenderPage),
    {
        let started = Instant::now();
        if cancel.is_cancelled() {
            self.emit_diagnostic(RenderDiagnostic::Cancelled);
            return Err(RenderEngineError::Cancelled);
        }
        let mut prep = RenderPrep::new(self.opts.prep).with_serif_default();
        prep = prep.with_embedded_fonts_from_book(book)?;
        let mut session = self.layout.start_session();
        prep.prepare_chapter_with(book, chapter_index, |item| {
            if cancel.is_cancelled() {
                return;
            }
            session.push_item_with_pages(item, &mut on_page)
        })?;
        if cancel.is_cancelled() {
            self.emit_diagnostic(RenderDiagnostic::Cancelled);
            return Err(RenderEngineError::Cancelled);
        }
        session.finish(&mut on_page);
        let elapsed = started.elapsed().as_millis().min(u32::MAX as u128) as u32;
        self.emit_diagnostic(RenderDiagnostic::ReflowTimeMs(elapsed));
        Ok(())
    }

    /// Prepare and layout a chapter, returning pages within `[start, end)`.
    ///
    /// Range indices are zero-based over the emitted chapter page sequence.
    /// Returned `RenderPage::page_number` values remain 1-based chapter page numbers.
    pub fn prepare_chapter_page_range<R: std::io::Read + std::io::Seek>(
        &self,
        book: &mut EpubBook<R>,
        chapter_index: usize,
        start: usize,
        end: usize,
    ) -> Result<Vec<RenderPage>, RenderEngineError> {
        if start >= end {
            return Ok(Vec::new());
        }
        let mut page_index = 0usize;
        let mut pages = Vec::new();
        self.prepare_chapter_with(book, chapter_index, |page| {
            if (start..end).contains(&page_index) {
                pages.push(page);
            }
            page_index += 1;
        })?;
        Ok(pages)
    }

    /// Prepare and layout a chapter and return pages as an iterator.
    ///
    /// This iterator is eager: pages are prepared first, then iterated.
    pub fn prepare_chapter_iter<R: std::io::Read + std::io::Seek>(
        &self,
        book: &mut EpubBook<R>,
        chapter_index: usize,
    ) -> Result<RenderPageIter, RenderEngineError> {
        let pages = self.prepare_chapter(book, chapter_index)?;
        Ok(RenderPageIter {
            inner: pages.into_iter(),
        })
    }

    /// Prepare and layout a chapter as a streaming iterator.
    ///
    /// Unlike `prepare_chapter_iter`, this method streams pages incrementally from a
    /// worker thread using a bounded channel (`capacity=1`) for backpressure.
    /// It requires ownership of the book so the worker can read resources directly.
    pub fn prepare_chapter_iter_streaming<R>(
        &self,
        mut book: EpubBook<R>,
        chapter_index: usize,
    ) -> RenderPageStreamIter
    where
        R: std::io::Read + std::io::Seek + Send + 'static,
    {
        let (tx, rx) = sync_channel(1);
        let engine = self.clone();

        std::thread::spawn(move || {
            let mut receiver_closed = false;
            let result = engine.prepare_chapter_with(&mut book, chapter_index, |page| {
                if receiver_closed {
                    return;
                }
                if tx.send(StreamMessage::Page(page)).is_err() {
                    receiver_closed = true;
                }
            });

            if receiver_closed {
                return;
            }
            match result {
                Ok(()) => {
                    let _ = tx.send(StreamMessage::Done);
                }
                Err(err) => {
                    let _ = tx.send(StreamMessage::Error(err));
                }
            }
        });

        RenderPageStreamIter {
            rx,
            finished: false,
        }
    }

    /// Prepare with an overlay composer that maps page metrics into overlay items.
    pub fn prepare_chapter_with_overlay_composer<R, O, F>(
        &self,
        book: &mut EpubBook<R>,
        chapter_index: usize,
        viewport: OverlaySize,
        composer: &O,
        mut on_page: F,
    ) -> Result<(), RenderEngineError>
    where
        R: std::io::Read + std::io::Seek,
        O: crate::render_ir::OverlayComposer,
        F: FnMut(RenderPage),
    {
        self.prepare_chapter_with(book, chapter_index, |mut page| {
            let overlays = composer.compose(&page.metrics, viewport);
            for item in overlays {
                page.overlay_items.push(item.clone());
                if let OverlayContent::Command(cmd) = item.content {
                    page.push_overlay_command(cmd);
                }
            }
            page.sync_commands();
            on_page(page);
        })
    }
}

/// Stable page iterator wrapper returned by `RenderEngine::prepare_chapter_iter`.
#[derive(Debug)]
pub struct RenderPageIter {
    inner: std::vec::IntoIter<RenderPage>,
}

impl Iterator for RenderPageIter {
    type Item = RenderPage;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for RenderPageIter {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl std::iter::FusedIterator for RenderPageIter {}

enum StreamMessage {
    Page(RenderPage),
    Error(RenderEngineError),
    Done,
}

/// Streaming page iterator produced by `RenderEngine::prepare_chapter_iter_streaming`.
#[derive(Debug)]
pub struct RenderPageStreamIter {
    rx: Receiver<StreamMessage>,
    finished: bool,
}

impl Iterator for RenderPageStreamIter {
    type Item = Result<RenderPage, RenderEngineError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        match self.rx.recv() {
            Ok(StreamMessage::Page(page)) => Some(Ok(page)),
            Ok(StreamMessage::Error(err)) => {
                self.finished = true;
                Some(Err(err))
            }
            Ok(StreamMessage::Done) | Err(_) => {
                self.finished = true;
                None
            }
        }
    }
}

/// Render engine error.
#[derive(Debug)]
pub enum RenderEngineError {
    /// Render prep failed.
    Prep(RenderPrepError),
    /// Layout run was cancelled.
    Cancelled,
}

impl core::fmt::Display for RenderEngineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Prep(err) => write!(f, "render prep failed: {}", err),
            Self::Cancelled => write!(f, "render cancelled"),
        }
    }
}

impl std::error::Error for RenderEngineError {}

impl From<RenderPrepError> for RenderEngineError {
    fn from(value: RenderPrepError) -> Self {
        Self::Prep(value)
    }
}
