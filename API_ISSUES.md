# API Issues

Current known API gaps across `mu-epub`, `mu-epub-render`, and `mu-epub-embedded-graphics`.

## High Priority

1. `EgRenderer` does not expose font-face registration workflow.
- Why this matters: `FontBackend::register_faces(...)` exists, but consumers cannot drive runtime face registration through a stable renderer API.
- Current refs: `crates/mu-epub-embedded-graphics/src/lib.rs:54`, `crates/mu-epub-embedded-graphics/src/lib.rs:215`.
- Proposed API:
  - `EgRenderer::backend_mut(&mut self) -> &mut B`
  - `EgRenderer::register_faces(&mut self, faces: &[FontFaceRegistration<'_>]) -> usize`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-embedded-graphics/src/lib.rs:337`
  - `crates/mu-epub-embedded-graphics/src/lib.rs:342`
- Tests:
  - `crates/mu-epub-embedded-graphics/src/lib.rs:718` (`renderer_register_faces_forwards_to_backend`)
  - `crates/mu-epub-embedded-graphics/src/lib.rs:747` (`backend_mut_exposes_font_backend_registration`)

2. `RenderEngine::prepare_chapter_with(...)` is stream-fed but page emission is not incremental.
- Why this matters: pages are emitted only at `finish`, so memory still grows with chapter size.
- Current refs: `crates/mu-epub-render/src/render_layout.rs:235`, `crates/mu-epub-render/src/render_layout.rs:245`.
- Proposed API:
  - `LayoutSession::push_item_with_pages(item, on_page)`
  - `RenderEngine::prepare_chapter_with(...)` should forward full pages as they close.
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_layout.rs:243` (`push_item_with_pages`)
  - `crates/mu-epub-render/src/render_engine.rs:53` (`prepare_chapter_with` now forwarding pages incrementally)
- Tests:
  - `crates/mu-epub-render/src/render_layout.rs:879` (`incremental_push_item_with_pages_matches_batch_layout`)

3. No page-range / lazy pagination API.
- Why this matters: consumers may need only one page or a small range.
- Current refs: `crates/mu-epub-render/src/render_engine.rs:42`, `crates/mu-epub-render/src/render_engine.rs:53`.
- Proposed API:
  - `RenderEngine::prepare_chapter_page_range(book, chapter, start, end)`
  - `RenderEngine::prepare_chapter_iter(...) -> impl Iterator<Item = RenderPage>`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_engine.rs:78` (`prepare_chapter_page_range`)
  - `crates/mu-epub-render/src/render_engine.rs:100` (`prepare_chapter_iter`)
  - `crates/mu-epub-render/src/render_engine.rs:119` (`prepare_chapter_iter_streaming`)
  - `crates/mu-epub-render/src/render_engine.rs:114` (`RenderPageIter`)
  - `crates/mu-epub-render/src/render_engine.rs:177` (`RenderPageStreamIter`)
- Tests:
  - `crates/mu-epub-render/tests/docs/pagination.rs:45` (`prepare_chapter_page_range_matches_full_slice`)
  - `crates/mu-epub-render/tests/docs/pagination.rs:62` (`prepare_chapter_iter_matches_full_render`)
  - `crates/mu-epub-render/tests/docs/pagination.rs:77` (`prepare_chapter_iter_streaming_matches_full_render`)
  - `crates/mu-epub-render/tests/docs/pagination.rs:94` (`prepare_chapter_iter_streaming_reports_errors`)

## Medium Priority

4. `ttf-backend` feature is present but not yet a full production text path.
- Why this matters: API suggests pluggability, but behavior is still largely fallback-oriented.
- Current refs: `crates/mu-epub-embedded-graphics/src/lib.rs:165`.
- Proposed API:
  - Explicit status in docs.
  - `TtfBackendOptions` with limits and fallback policy.
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-embedded-graphics/src/lib.rs:183` (`TtfBackendOptions`)
  - `crates/mu-epub-embedded-graphics/src/lib.rs:241` (`TtfFontBackend::status`)
- Tests:
  - `crates/mu-epub-embedded-graphics/src/lib.rs:956` (`ttf_backend_exposes_options_and_status`)
  - `crates/mu-epub-embedded-graphics/src/lib.rs:970` (`ttf_backend_registration_enforces_limits`)

5. Page chrome policy customization is too limited.
- Why this matters: header/footer/progress are rendered, but placement/style policy is mostly fixed.
- Current refs: `crates/mu-epub-render/src/render_layout.rs:55`, `crates/mu-epub-embedded-graphics/src/lib.rs:341`.
- Proposed API:
  - `PageChromeConfig` (positions, text style, progress geometry).
  - Pass into `LayoutConfig`/`EgRenderConfig`.
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_ir.rs:147` (`PageChromeConfig`)
  - `crates/mu-epub-render/src/render_layout.rs:57` (`LayoutConfig.page_chrome`)
  - `crates/mu-epub-embedded-graphics/src/lib.rs:299` (`EgRenderConfig.page_chrome`)
- Tests:
  - `crates/mu-epub-render/src/render_layout.rs:775` (`page_chrome_policy_controls_emitted_markers`)
  - `crates/mu-epub-embedded-graphics/src/lib.rs:885` (`page_chrome_config_changes_progress_geometry`)
  - `crates/mu-epub-embedded-graphics/src/lib.rs:917` (`page_chrome_config_can_suppress_renderer_chrome_drawing`)

6. Error context is richer but not strongly typed for selector/declaration indices.
- Why this matters: machine consumers (CI dashboards/tools) need structured indexing fields.
- Current refs: `src/render_prep.rs:124`.
- Proposed API:
  - Add typed fields:
    - `selector_index: Option<usize>`
    - `declaration_index: Option<usize>`
    - `token_offset: Option<usize>`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `src/render_prep.rs:124` (`RenderPrepErrorContext` typed fields)
  - `src/render_prep.rs:170` (`with_selector_index`)
  - `src/render_prep.rs:186` (`with_declaration_index`)
  - `src/render_prep.rs:194` (`with_token_offset`)
- Tests:
  - `src/render_prep.rs:1722` (`style_tokenize_error_sets_token_offset_context`)
  - `src/render_prep.rs:1736` (`render_prep_error_context_supports_typed_indices`)

## Low Priority

7. Backward trace API remains dual (`prepare_chapter_with_trace` + `prepare_chapter_with_trace_context`).
- Why this matters: surface area duplication.
- Current refs: `src/render_prep.rs:1017`, `src/render_prep.rs:1041`.
- Proposed API:
  - Keep `prepare_chapter_with_trace_context` as primary.
  - Deprecate legacy optional-trace callback in a future release.
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `src/render_prep.rs:1062` (`prepare_chapter_with_trace_context`)
  - `src/render_prep.rs:1086` (`prepare_chapter_with_trace` is deprecated and forwards to context trace)

8. No dedicated migration/examples doc for advanced render pipeline usage.
- Why this matters: API is powerful but discovery is harder than needed.
- Proposed addition:
  - `docs/render-api.md` with minimal and advanced examples.
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `docs/render-api.md:1`

## Suggested Execution Order

1. Expose backend registration (`EgRenderer` mutable/registration APIs).
2. Add true incremental page emission in layout session + engine bridge.
3. Add page-range/lazy pagination APIs.
4. Add chrome config APIs.
5. Strengthen typed error indexing fields.
6. Improve docs and deprecate redundant trace API later.

## Additional API Gaps (Requested)

These additions align the renderer stack for production reader workflows while keeping layering clean between content layout and app policy.

### High Priority

9. Split content rendering from chrome/overlay rendering.
- Why this matters: content pagination should be deterministic and reusable while overlays (header/footer/progress/clock/battery) remain app-driven.
- Proposed API:
  - `RenderPage { content_commands, annotations, metrics }`
  - `PageMetrics { chapter_index, chapter_page_index, chapter_page_count, global_page_index, global_page_count_estimate, progress_chapter, progress_book }`
  - `OverlayComposer::compose(&self, metrics: &PageMetrics, viewport: Size) -> Vec<DrawCommand>`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_ir.rs:5` (`RenderPage` split channels + metrics/annotations)
  - `crates/mu-epub-render/src/render_ir.rs:174` (`OverlayComposer`)
  - `crates/mu-epub-embedded-graphics/src/lib.rs:396` (`render_content` uses content channel)
  - `crates/mu-epub-embedded-graphics/src/lib.rs:420` (`render_overlay` uses chrome/overlay channels)
- Tests:
  - `crates/mu-epub-embedded-graphics/src/lib.rs:1035` (`split_and_single_stream_render_paths_are_compatible`)

10. First-class reading progress model.
- Why this matters: resume/sync/position restore currently depend on page numbers that are unstable across layout changes.
- Proposed API:
  - `current_position(&self) -> ReadingPosition`
  - `seek_position(&mut self, pos: &ReadingPosition) -> Result<()>`
  - `chapter_progress(&self) -> f32`
  - `book_progress(&self) -> f32`
  - `ReadingPosition` should support CFI-like anchors plus fallback offsets.
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `src/book.rs:190` (`ReadingPosition`)
  - `src/book.rs:252` (`current_position`)
  - `src/book.rs:257` (`seek_position`)
  - `src/book.rs:272` (`chapter_progress`)
  - `src/book.rs:284` (`book_progress`)
- Tests:
  - `src/book.rs:1369` (`test_reading_session_resolve_locator_and_progress`)
  - `src/book.rs:1403` (`test_reading_session_seek_position_out_of_bounds`)

11. Locator and TOC navigation primitives.
- Why this matters: consumers need semantic jumps (`href`, fragment, toc id), not only index-driven navigation.
- Proposed API:
  - `enum Locator { Chapter(usize), Href(String), Fragment(String), TocId(String), Position(ReadingPosition) }`
  - `resolve_locator(&mut self, loc: Locator) -> Result<ResolvedLocation>`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `src/book.rs:203` (`Locator`)
  - `src/book.rs:216` (`ResolvedLocation`)
  - `src/book.rs:293` (`resolve_locator`)
- Tests:
  - `src/book.rs:1369` (`test_reading_session_resolve_locator_and_progress`)

12. Stable pagination profile id.
- Why this matters: persisted positions should be invalidated or migrated when layout-affecting config changes.
- Proposed API:
  - `struct PaginationProfileId([u8; 32]);`
  - `pagination_profile_id(&self) -> PaginationProfileId`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_ir.rs:99` (`PaginationProfileId`)
  - `crates/mu-epub-render/src/render_engine.rs:99` (`pagination_profile_id`)
- Tests:
  - `crates/mu-epub-render/tests/docs/pagination.rs:113` (`pagination_profile_id_is_stable_for_same_options`)

13. Incremental reflow with cancellation.
- Why this matters: settings changes need interruption without blocking UI event loops.
- Proposed API:
  - `trait CancelToken { fn is_cancelled(&self) -> bool; }`
  - `prepare_chapter_with_cancel(..., cancel: &impl CancelToken, ...)`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_engine.rs:11` (`CancelToken`)
  - `crates/mu-epub-render/src/render_engine.rs:132` (`prepare_chapter_with_cancel`)
- Tests:
  - `crates/mu-epub-render/tests/docs/pagination.rs:129` (`prepare_chapter_with_cancel_stops_early`)

14. Font fallback chain as first-class API.
- Why this matters: consumers need visibility into fallback decisions (missing glyph, weight/style mismatch, policy rejection).
- Proposed API:
  - `FontFallbackPolicy { preferred_families, allow_embedded_fonts, synthetic_bold, synthetic_italic }`
  - trace output should include rejected candidates with reason codes.
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `src/render_prep.rs:737` (`FontPolicy` expanded fallback policy)
  - `src/render_prep.rs:764` (`FontFallbackPolicy` alias)
  - `src/render_prep.rs:897` (`allow_embedded_fonts` handling in trace path)
  - `src/render_prep.rs:945` (`preferred family reasoning in trace`)

15. Structured diagnostics stream.
- Why this matters: non-fatal behavior (unsupported CSS, fallback usage, reflow timing) should be observable for tuning and QA.
- Proposed API:
  - `enum RenderDiagnostic { CssUnsupported{...}, FontFallback{...}, ReflowTimeMs(u32), ... }`
  - `set_diagnostic_sink(&mut self, sink: impl FnMut(RenderDiagnostic) + 'static)`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_engine.rs:27` (`RenderDiagnostic`)
  - `crates/mu-epub-render/src/render_engine.rs:82` (`set_diagnostic_sink`)
- Tests:
  - `crates/mu-epub-render/tests/docs/pagination.rs:180` (`diagnostic_sink_receives_reflow_timing`)

### Medium Priority

16. Typography/hyphenation policy surface.
- Why this matters: quality text layout requires explicit control over hyphenation, widow/orphan behavior, and justification policy.
- Proposed API:
  - `TypographyConfig { hyphenation, widow_orphan_control, justification, hanging_punctuation }`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_ir.rs:404` (`TypographyConfig`)
  - `crates/mu-epub-render/src/render_layout.rs:60` (`LayoutConfig.typography`)
  - `crates/mu-epub-render/src/render_layout.rs:367` (hyphenation policy hook)
  - `crates/mu-epub-render/src/render_layout.rs:473` (justification policy hook)

17. Image and block object policy.
- Why this matters: renderer is text-first today; predictable object behavior needs clear policy knobs.
- Proposed API:
  - `ObjectLayoutConfig { max_inline_image_height_ratio, float_support, svg_mode, alt_text_fallback }`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_ir.rs:484` (`ObjectLayoutConfig`)
  - `crates/mu-epub-render/src/render_layout.rs:62` (`LayoutConfig.object_layout`)

18. Theme-aware render intents.
- Why this matters: e-ink and constrained displays often require grayscale/dither/contrast tuning independent of layout.
- Proposed API:
  - `RenderIntent { grayscale_mode, dither, contrast_boost }`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_ir.rs:193` (`RenderIntent`)
  - `crates/mu-epub-render/src/render_layout.rs:64` (`LayoutConfig.render_intent`)

19. Overlay slots instead of renderer-specific chrome.
- Why this matters: generic slot API supports many UI overlays without EPUB-specific coupling.
- Proposed API:
  - `enum OverlaySlot { TopLeft, TopCenter, TopRight, BottomLeft, BottomCenter, BottomRight, Custom(Rectangle) }`
  - `OverlayItem { slot, z, content }`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-render/src/render_ir.rs:127` (`OverlaySlot`)
  - `crates/mu-epub-render/src/render_ir.rs:154` (`OverlayItem`)
  - `crates/mu-epub-render/src/render_engine.rs:254` (`prepare_chapter_with_overlay_composer`)
- Tests:
  - `crates/mu-epub-render/tests/docs/pagination.rs:157` (`overlay_composer_attaches_overlay_items`)

20. Backend capability flags.
- Why this matters: apps can degrade gracefully when a backend lacks TTF/images/SVG/full justification.
- Proposed API:
  - `BackendCapabilities { ttf, images, svg, justification }`
  - `capabilities(&self) -> BackendCapabilities`
- Status: `Resolved (2026-02-09)`.
- Implemented refs:
  - `crates/mu-epub-embedded-graphics/src/lib.rs:55` (`BackendCapabilities`)
  - `crates/mu-epub-embedded-graphics/src/lib.rs:383` (`EgRenderer::capabilities`)
- Tests:
  - `crates/mu-epub-embedded-graphics/src/lib.rs:896` (`mono_backend_capabilities_match_expected_flags`)

### Integration Notes

21. Keep layering strict:
- `mu-epub`: parsing/resources/style/font discovery and styled output.
- `mu-epub-render`: layout/pagination and backend-agnostic IR generation.
- `mu-epub-embedded-graphics`: IR execution, backend fonts, and capability reporting.

22. Place chapter/book progress in metrics and anchors, not backend renderers.
- Reason: this keeps progress deterministic and independent of a specific draw target.

23. Ensure resolved font identity (`font_id`) remains in IR text commands.
- Reason: draw backends must not re-guess face from weight/italic at paint time.
