# Changelog

All notable changes to this project are documented in this file.

The format is based on Keep a Changelog, and this project follows Semantic Versioning.

## [0.2.0] - 2026-02-08

### Added
- Added regression tests for parser API error typing, layout top-margin defaults, and ZIP error alias behavior.
- Added `EpubMetadata::opf_path` and extraction tests proving `container.xml` rootfile paths are preserved.
- Added crate-root re-exports for `ZipError` and `ZipErrorKind`.
- Added GitHub Actions CI for formatting, linting, build checks (`std` and `no_std`), tests (including ignored tests), and docs.

### Changed
- Unified ZIP error API naming around `ZipError = ZipErrorKind`.
- `EpubError::Zip` now uses `ZipError` alias directly.
- Standardized layout defaults so `LayoutEngine::new()` starts at `DEFAULT_TOP_MARGIN`.
- Updated bug tracker: all previously deferred items are now marked resolved with linked regression coverage.

### Fixed
- Mixed inline formatting preservation across line layout (`Line` span handling path now covered by regressions).
- Metadata extraction now stores parsed OPF path instead of discarding the parsed `container.xml` rootfile value.

