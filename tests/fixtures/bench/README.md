# Public-Domain Benchmark Fixture Corpus

This folder contains public-domain EPUB files downloaded from Project Gutenberg
for benchmarking and parser robustness testing.

## Sources

| File | Title | Project Gutenberg URL |
|---|---|---|
| `pg84-frankenstein.epub` | *Frankenstein* (Mary Shelley, #84) | `https://www.gutenberg.org/ebooks/84.epub.images` |
| `pg1342-pride-and-prejudice.epub` | *Pride and Prejudice* (Jane Austen, #1342) | `https://www.gutenberg.org/ebooks/1342.epub.images` |
| `pg1661-sherlock-holmes.epub` | *The Adventures of Sherlock Holmes* (Arthur Conan Doyle, #1661) | `https://www.gutenberg.org/ebooks/1661.epub.images` |
| `pg2701-moby-dick.epub` | *Moby Dick* (Herman Melville, #2701) | `https://www.gutenberg.org/ebooks/2701.epub.images` |

## Integrity (SHA-256)

```text
f3ce6db78dfdc098bff1893ae97f34fffe29fbb622464e5d95c1ead68ab7380d  pg1342-pride-and-prejudice.epub
a8d80ba6f60d1cf79c82af7624005a24b7f52157f10313d0f2e0f9fe289ff017  pg1661-sherlock-holmes.epub
7e8e2b1903c0dac6a288bd1b737f0ba051590bcdaedd83a74be6771b18cc8c9f  pg2701-moby-dick.epub
6ab7f2adbc0008e51e65bc534db625f6fe7856f6f445807c76f981f62dc4bf90  pg84-frankenstein.epub
```

## Notes

- These files are not used by default unit tests.
- They are intended for manual and benchmark runs.
