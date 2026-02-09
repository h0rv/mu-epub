# EPUB Spec Compliance

Target: EPUB 3.2 subset (structure is forward-compatible with 3.3).
EPUB 2.0 fallbacks (NCX, guide element) are implemented for compatibility with
older files.

Status key: **done** | **partial** | **--** (not started) | **n/a** (out of scope for v1)

## Container (OCF)

| Feature                          | Status  | Notes                                                        |
|----------------------------------|---------|--------------------------------------------------------------|
| ZIP container reading            | done    | Streaming, bounded buffer, EOCD, Stored + DEFLATE, CRC32    |
| `mimetype` file validation       | done    | `validate_mimetype()` checks content = `application/epub+zip` |
| `META-INF/container.xml` parsing | done    | Extracts rootfile `full-path`                                |
| Encryption (`encryption.xml`)    | n/a     |                                                              |
| Digital signatures               | n/a     |                                                              |

## Package Document (OPF)

| Feature                                | Status  | Notes                                                   |
|----------------------------------------|---------|---------------------------------------------------------|
| `<metadata>` (title, author, language) | done    | dc:title, dc:creator, dc:language                       |
| Full Dublin Core metadata              | done    | date, publisher, rights, description, subjects, identifier |
| `<manifest>` (resource list)           | done    | id, href, media-type, properties (max 64)               |
| `<spine>` (reading order)              | done    | idref, id, linear, properties (max 256)                 |
| Cover image detection                  | done    | EPUB 2.0 meta tag + EPUB 3.x `cover-image`             |
| EPUB-specific metadata                 | done    | `dcterms:modified`, `rendition:layout`                  |
| `<guide>` (EPUB 2.0, deprecated)       | done    | Parses `<reference>` with type, title, href             |
| Media overlay references               | n/a     |                                                         |

## Navigation

| Feature                          | Status  | Notes                                                    |
|----------------------------------|---------|----------------------------------------------------------|
| XHTML nav (`epub:type="toc"`)    | done    | Nested `<ol>/<li>/<a>` parsing with hierarchical output  |
| NCX (`toc.ncx`)                  | done    | `<navMap>` with nested `<navPoint>` support              |
| Page list                        | done    | XHTML `epub:type="page-list"` + NCX `<pageList>`        |
| Landmarks                        | done    | XHTML `epub:type="landmarks"`                            |

## Content Documents

| Feature                          | Status  | Notes                                                    |
|----------------------------------|---------|----------------------------------------------------------|
| XHTML tokenization (SAX)         | done    | No DOM; quick-xml pull parser                            |
| Paragraphs (`<p>`)               | done    | Emits `ParagraphBreak`                                   |
| Headings (`<h1>`-`<h6>`)         | done    | Emits `Heading(level)`                                   |
| Emphasis (`<em>`, `<i>`)         | done    | Emits `Emphasis(bool)`, nesting supported                |
| Strong (`<strong>`, `<b>`)       | done    | Emits `Strong(bool)`, nesting supported                  |
| Line breaks (`<br>`)             | done    | Emits `LineBreak`                                        |
| Block containers (`<div>`)       | done    | Treated as block (emits `ParagraphBreak`)                |
| Inline containers (`<span>`)     | done    | Transparent, text extracted                              |
| Skipped elements                 | done    | script, style, head, nav, header, footer, aside, noscript |
| Lists (`<ul>`, `<ol>`, `<li>`)   | done    | `ListStart(ordered)`, `ListItemStart/End`, `ListEnd`     |
| Links (`<a>`)                    | done    | `LinkStart(href)` / `LinkEnd`; no-href treated as generic |
| Images (`<img>`)                 | done    | `Image { src, alt }`; missing src skipped                |
| Tables                           | n/a     |                                                          |
| SVG content documents            | n/a     |                                                          |
| MathML                           | n/a     |                                                          |
| JavaScript / forms               | n/a     |                                                          |
| Audio / video                    | n/a     |                                                          |

## CSS (Subset)

| Feature                          | Status  | Notes                                                    |
|----------------------------------|---------|----------------------------------------------------------|
| `font-size` (px, em)             | done    | `FontSize::Px` / `FontSize::Em`                         |
| `font-family`                    | done    | Strips quotes, first family                              |
| `font-weight` (normal, bold)     | done    | Also numeric: 400=normal, 700/800/900=bold               |
| `font-style` (normal, italic)    | done    | Also `oblique` maps to italic                            |
| `text-align`                     | done    | left, center, right, justify                             |
| `line-height`                    | done    | px values                                                |
| `margin-top`, `margin-bottom`    | done    | px values; `margin` shorthand (single value)             |
| Inline styles                    | done    | `parse_inline_style()` for `style=""` attributes         |
| Tag / class selectors            | done    | Tag, `.class`, `tag.class` selectors                     |
| Stylesheet resolution            | done    | `Stylesheet::resolve()` cascades matching rules          |
| Complex selectors                | n/a     |                                                          |
| Floats / positioning / grid      | n/a     |                                                          |

## Layout

| Feature                          | Status  | Notes                                                    |
|----------------------------------|---------|----------------------------------------------------------|
| Greedy line breaking             | done    | Word-level greedy in `layout.rs`                         |
| Multi-page pagination            | done    | Page/Line/TextStyle model                                |
| Heading spacing                  | done    | Extra space before headings, always bold                 |
| Paragraph spacing                | done    | Half-line gap between paragraphs                         |
| Style tracking                   | done    | Normal, Bold, Italic, BoldItalic                         |
| List layout                      | done    | Bullets (•) / numbered (1. 2. 3.), nested indentation    |
| Image placeholders               | done    | `[Image: alt]` or `[Image]` placeholder text             |
| Link rendering                   | done    | Text flows normally; link tokens are informational       |
| Page map persistence             | --      |                                                          |
| Fixed layouts                    | n/a     |                                                          |
| Spreads (two-page view)          | n/a     |                                                          |
| Bidirectional text (RTL)         | --      | Behind `epub_full` flag                                  |
| Ruby annotations                 | n/a     |                                                          |

## Error Handling

| Feature                          | Status  | Notes                                                    |
|----------------------------------|---------|----------------------------------------------------------|
| Unified `EpubError` type         | done    | Wraps ZIP, parse, navigation, CSS, I/O errors            |
| `From` conversions               | done    | `ZipError`, `TokenizeError`, `String`, `&str` → `EpubError` |

## Fonts

| Feature                          | Status  | Notes                                          |
|----------------------------------|---------|-------------------------------------------------|
| Built-in fonts                   | --      | Only a monospace 10x20 stub in `FontMetrics`   |
| User fonts from storage           | --      | Size cap ~200KB                                |
| Embedded fonts from EPUB         | --      | Size cap enforced                              |
| Font fallback chain              | --      |                                                |
| Complex script shaping           | --      | Behind `epub_full` flag                        |

## Test Coverage

| Module       | Unit tests | Notes                                         |
|--------------|------------|-----------------------------------------------|
| zip          | 12         | ZIP reading, mimetype validation, error types |
| metadata     | 17         | DC fields, cover, guide, EPUB3 meta           |
| spine        | 19         | Parsing, navigation, progress, edge cases     |
| tokenizer    | 33         | All HTML elements, edge cases, nesting        |
| navigation   | 20         | XHTML nav, NCX, page list, landmarks          |
| css          | 29         | Parsing, selectors, cascading, resolution     |
| layout       | 30         | Pagination, lists, images, styles, boundaries |
| error        | 5          | Display, conversions                          |
| integration  | 23         | End-to-end pipeline (15 require EPUB fixture) |
| **Total**    | **188+**   |                                               |

## References

- EPUB 3.3: https://www.w3.org/TR/epub-33/
- EPUB 3.3 Reading Systems: https://www.w3.org/TR/epub-rs-33/
- EPUBCheck (validator): https://github.com/w3c/epubcheck
- W3C test suite: https://w3c.github.io/epub-tests/
