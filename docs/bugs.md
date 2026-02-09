# Bug Tracker

All known issues found during pre-release audit. Organized by severity.

---

## P0 — Correctness Bugs (wrong output on real EPUBs)

### BUG-001: XML entities not unescaped in tokenizer — ✅ RESOLVED
- **File:** `src/tokenizer.rs`
- **Impact:** `&amp;` → literal `&amp;` in output instead of `&`. Affects nearly every EPUB.
- **Root cause:** Used `reader.decoder().decode(&e)` instead of `e.unescape()`. Also missed `Event::GeneralRef` for entity references.
- **Fix:** Added `Event::GeneralRef` handler that decodes entity refs via `quick_xml::escape::unescape()`.
- **Regression test:** `bug_001_xml_entity_amp`, `bug_001_xml_entity_lt_gt`, `bug_001_xml_entity_numeric`

### BUG-002: Heading bold bleeds into all subsequent text — ✅ RESOLVED
- **File:** `src/layout.rs`
- **Impact:** After any heading, all body text renders bold for the rest of the chapter.
- **Root cause:** `Token::Heading` set `bold_active = true` but nothing reset it.
- **Fix:** Introduced `heading_bold` flag separate from `bold_active`. Reset on `ParagraphBreak`.
- **Regression test:** `bug_002_heading_bold_does_not_bleed`

### BUG-003: `FontMetrics::text_width` uses byte length, not char count — ✅ RESOLVED
- **File:** `src/layout.rs`
- **Impact:** Non-ASCII text (em-dashes, curly quotes, accented chars) measures wrong, breaking line wrapping.
- **Fix:** Changed `text.len()` to `text.chars().count()`.
- **Regression test:** `bug_003_text_width_char_count_not_bytes`, `bug_003_text_width_em_dash`

### BUG-004: Single `TextStyle` per `Line` — mixed formatting destroyed — ✅ RESOLVED
- **File:** `src/layout.rs`
- **Impact:** "normal **bold** text" on one line records only the last word's style.
- **Root cause:** `Line { text: String, style: TextStyle }` — one style for the entire line.
- **Fix:** Refactored to `Line { spans: Vec<TextSpan> }` and added span-aware layout/flush logic.
- **Regression test:** `mixed_formatting_preserved`, `mixed_formatting_multiple_transitions`, `mixed_formatting_with_line_wrapping`

## P0 — API Issues (breaking to fix after publishing) — ALL RESOLVED

### BUG-005: All parsers return `Result<_, String>` instead of `EpubError` — ✅ RESOLVED
- **Fix:** Public parser APIs now return typed errors (`EpubError` or module-specific typed errors).
- **Regression test:** `parser_apis_use_epub_error`

### BUG-006: No `std::error::Error` impl on any error type — ✅ RESOLVED
- **Fix:** Added `#[cfg(feature = "std")] impl std::error::Error` for `EpubError`, `ZipError`, `TokenizeError`.
- **Also:** Added `Display` impls for `ZipError` and `ZipErrorKind`.
- **Regression test:** `bug_006_epub_error_implements_std_error`, `bug_006_tokenize_error_implements_std_error`, `bug_006_zip_error_implements_std_error`

### BUG-007: `Spine::current` and `Spine::items` are `pub` fields — ✅ RESOLVED
- **Fix:** Made fields private, added `pub fn items(&self) -> &[SpineItem]` accessor.

### BUG-008: `layout_tokens` takes `Vec<Token>` by value — ✅ RESOLVED
- **Fix:** Changed to `&[Token]`. Updated all callers.

### BUG-009: No `#[non_exhaustive]` on public enums — ✅ RESOLVED
- **Fix:** Added `#[non_exhaustive]` to all public enums: `Token`, `TokenizeError`, `EpubError`, `ZipErrorKind`, `ZipError`, `TextStyle`, `FontSize`, `FontWeight`, `FontStyle`, `TextAlign`, `CssSelector`, `LineHeight`.

## P1 — Safety / Resource Issues — ALL RESOLVED

### BUG-010: Unbounded allocation from untrusted ZIP sizes — ✅ RESOLVED
- **Fix:** Added explicit runtime-configurable ZIP limits (`ZipLimits`) and limit-aware constructors. Applications can opt into strict caps for `read_file()` and `validate_mimetype()`.

### BUG-011: `MAX_MANIFEST_ITEMS = 64` too low — ✅ RESOLVED
- **Fix:** Increased to 1024. Added `MAX_SUBJECTS = 64` and `MAX_GUIDE_REFS = 64`.

### BUG-012: ZIP filename at exactly `MAX_FILENAME_LEN` silently dropped — ✅ RESOLVED
- **Fix:** Changed `<` to `<=`. Added skip branch for oversized filenames.

### BUG-013: No CRC check on stored (uncompressed) ZIP files — ✅ RESOLVED
- **Fix:** Added CRC32 verification to `METHOD_STORED` branch.

## P1 — Correctness Issues — ALL RESOLVED

### BUG-014: CSS `line-height: 1.5` (unitless) misinterpreted as 1.5px — ✅ RESOLVED
- **Fix:** Added `LineHeight` enum with `Px(f32)` / `Multiplier(f32)`. Bare numbers parsed as multiplier.
- **Regression test:** `bug_014_css_unitless_line_height`

### BUG-015: Navigation labels truncated for formatted anchors — ✅ RESOLVED
- **Fix:** Changed label assignment from overwrite to concatenate in both `parse_nav_xhtml()` and `parse_ncx()`.
- **Regression test:** `bug_015_nav_label_formatted_anchor`

### BUG-016: Metadata `ends_with("title")` matches unrelated elements — ✅ RESOLVED
- **Fix:** Replaced `ends_with(X)` with exact `"X" | "dc:X"` matches for all 9 Dublin Core fields.
- **Regression test:** `bug_016_metadata_subtitle_not_matched_as_title`

### BUG-017: `EpubMetadata` defaults to `"Unknown Title"` / `"Unknown Author"` — ✅ RESOLVED
- **Fix:** Changed defaults to `String::new()`. Missing metadata is now distinguishable.
- **Regression test:** `bug_017_missing_title_is_distinguishable`

## P2 — Cleanup / Polish — ALL RESOLVED

### BUG-018: Duplicate error types — ✅ RESOLVED
- **Fix:** Unified ZIP error naming by introducing crate-level alias `ZipError = ZipErrorKind` and using it consistently.
- **Regression test:** `zip_error_alias_matches_kind`

### BUG-019: Dead code and unused params — ✅ RESOLVED
- Removed `_dc_ns` in `metadata.rs`
- Removed unused `_uncompressed_size` param from `read_file_at_offset`
- Made `debug_list_entries` private

### BUG-020: `LayoutConfig` has dead fields — ✅ RESOLVED
- Removed `right_margin` and `bottom_margin` fields.

### BUG-021: Three conflicting default configurations in layout — ✅ RESOLVED
- **Fix:** `LayoutEngine::new()` now initializes `top_margin/current_y` from `DEFAULT_TOP_MARGIN` for default consistency.
- **Regression test:** `layout_new_uses_default_top_margin`

### BUG-022: README inaccuracies — ✅ RESOLVED
- Fixed `std` default to "yes", removed non-existent `parser` feature.

### BUG-023: No crate-root re-exports — ✅ RESOLVED
- Added `pub use` for `CssStyle`, `Stylesheet`, `EpubError`, `EpubMetadata`, `Navigation`, `Spine`, `Token`, `TokenizeError`.

### BUG-024: Missing common trait impls — ✅ RESOLVED
- Added `Copy` to `FontSize`
- Added `Eq` to all types with Eq-compatible fields
- Added `Display` to `ZipError` and `ZipErrorKind`

### BUG-025: `From<String>` blanket impl on `EpubError` too broad — ✅ RESOLVED
- Removed `From<String>` and `From<&str>` impls on `EpubError`.

### BUG-026: `extract_metadata` parses container.xml but discards result — ✅ RESOLVED
- **Fix:** `extract_metadata()` stores parsed rootfile path in `EpubMetadata::opf_path`.
- **Regression test:** `test_extract_metadata_uses_container_xml_path`, `test_extract_metadata_different_rootfile_path`

### BUG-027: Silent truncation at 256 ZIP entries / 256 spine items — ✅ RESOLVED
- Added `log::warn!` when entry count exceeds `MAX_CD_ENTRIES`.
