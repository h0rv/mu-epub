# Architecture

Goal: A production EPUB reader library with real typography, chapter navigation,
page counts, font size/family switching, and persistence -- all within tight
memory limits. Designed for `no_std` environments with `alloc`, but works on
any platform. Full language support is gated behind an optional feature flag;
the default path is Latin-only for fast bring-up.

Non-goals (v1): full CSS2/3 layout, JavaScript, SVG, MathML, audio/video,
complex floats/tables.

## Pipeline

```
EPUB (.epub file)
  |
  v
1. Streaming ZIP reader (bounded buffer, miniz_oxide)
  |
  v
2. container.xml -> content.opf (quick-xml, SAX-style)
  |
  v
3. XHTML tokenizer -> per-chapter token cache (.tok files)
  |
  v
4. Layout engine: tokens -> line breaks -> page map (.pg files)
  |
  v
5. Renderer: glyph LRU cache -> framebuffer
```

Decoupling parsing from layout via the token cache means reflow (font size
change, font switch) only re-runs steps 4-5.

## Library Stack

| Purpose         | Library                              | Notes                                |
|-----------------|--------------------------------------|--------------------------------------|
| ZIP             | `rc-zip` + `miniz_oxide`             | Sans-I/O state machine, `no_std`     |
| XML/XHTML       | `quick-xml`                          | Pull parser, SAX-style streaming     |
| Fonts (latin)   | `fontdue`                            | `no_std` rasterizer, simple layout   |
| Fonts (full)    | `ttf-parser` + `rustybuzz` + fontdue | Complex script shaping               |
| Cache format    | `postcard`                           | `no_std`, compact, fixed schema      |

Alternatives considered but deferred: `swash` (heavier but higher quality),
`rkyv` (zero-copy but tricky versioning), `msgpack-rust` (self-describing but
larger).

## Data Model

### Token stream (structure cache)

Each chapter serialized as a stream of:
- `TextRun { text, style_id }`
- `ParagraphBreak`
- `Heading { level }`
- `ListItem { level }`
- `EmphasisOn/Off`, `StrongOn/Off`
- `ImageRef { path, width, height }`
- `SoftBreak`, `HardBreak`

Format: `postcard` with a versioned header. Per-chapter target: <20KB.

### Page map

Per chapter, stored persistently:
- `page_index -> token_offset`
- `token_offset -> text_offset`

Enables fast page turns, total page counts, and resume after reboot.

## CSS Subset (v1)

Supported: `font-size` (px, em), `font-family`, `font-weight` (normal/bold),
`font-style` (normal/italic), `text-align`, `line-height`, `margin-top/bottom`.

Selectors: tag, class, and inline style only.

Everything else is ignored.

## Fonts

1. Built-in fonts (default): bundled with the application, always available.
2. User fonts: loaded from storage if under the size cap (~200KB).
3. Embedded fonts: from EPUB resources if under the size cap.
4. Fallback: if a font is too large or fails to load, use built-in.

Most TTF parsers need the font blob in RAM, so large embedded fonts are not
feasible without a desktop preprocessing step (subset to used codepoints).

### Feature flags

- `epub_latin` (default): `fontdue` only, minimal Unicode handling.
- `epub_full` (optional): `rustybuzz` + bidi + Unicode line breaking.

## Pagination

- On open: parse metadata, build token cache for the first chapter only.
- Page maps built incrementally: current chapter immediately, adjacent chapters
  in the background when idle.
- Total page count becomes accurate once all maps are built.
- Page maps persist to storage; subsequent opens are instant.

## Memory Budget

| Component              | Budget    |
|------------------------|-----------|
| ZIP + XML buffers      | 8-16 KB   |
| Chapter token cache    | 20-32 KB  |
| Glyph cache            | 24-32 KB  |
| Layout state           | 8 KB      |
| Metadata + spine       | 4-8 KB    |
| **Total (excl. framebuffer)** | **<120 KB** |

## Performance Targets

| Operation        | Target  |
|------------------|---------|
| Open (first)     | <2s     |
| Open (cached)    | <1s     |
| Page turn        | <200ms  |
| Font size change | <5s     |

## Implementation Phases

0. **Stabilize** -- disable embedded fonts, move large buffers off stack,
   remove parsing recursion, add heap/stack watermarks.
1. **Streaming EPUB core** -- `EpubArchive` with streaming ZIP, parse
   `container.xml` and `content.opf`, build spine + metadata. Target <80KB peak.
2. **XHTML tokenizer + cache** -- SAX parse XHTML into tokens, resolve basic
   CSS, write `.tok` files per chapter.
3. **Layout + page map** -- greedy line breaking, single-page render from token
   stream, per-chapter `.pg` page maps. Target <200ms page turn.
4. **Fonts + shaping** -- `epub_latin` path first, font size/family switching,
   then `epub_full` behind a feature flag.
5. **UX integration** -- TOC from `nav.xhtml` / `toc.ncx`, progress
   save/load, bookmarks.
6. **Performance + power** -- background pagination, chapter prefetch, partial
   display refresh tuning.

## Risks

- Large embedded fonts: enforce size cap, fall back, optional desktop subsetting.
- Complex scripts: shaping adds CPU/RAM; `latin-only` compile-time profile.
- CSS complexity: only a subset is supported.
- Storage performance: keep reads aligned, cache headers.

## Future: Desktop Preprocessor

Optional tool that reads an EPUB, extracts used codepoints, subsets fonts, and
writes a cache bundle (tokens + page maps + subset fonts). The device uses the
bundle if present, otherwise falls back to on-device fonts with size caps. Not
required for initial implementation, but the cache schema should accommodate it.
