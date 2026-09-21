# untechxt

Encode Unicode text as LaTeX source, and learn what the document's preamble
needs for it to print.

A LaTeX document cannot always take a character as it stands: an old
installation reads no UTF-8, a font has no glyph for the character, or the
character is one of LaTeX's own special characters (`%`, `&`, `#`). `untechxt`
replaces each such character by the LaTeX that prints it — `é` by `\'e`, `α` by
`\ensuremath{\alpha}`, `—` by `{\textemdash}` — leaves ordinary characters
alone, and reports which packages and declarations the result needs. It is a
Rust library, `no_std`, and a redesign of the `latexencode` module of
[pylatexenc](https://github.com/phfaist/pylatexenc), by the same author.

```rust
use untechxt::{default_rules, encode, Encoder};

// One call, every default setting.
assert_eq!(encode("Café — naïve"), r#"Caf\'e {\textemdash} na\"ive"#);

// An encoder, and what its output needs in the preamble.
let encoder = Encoder::new(default_rules());
let (body, report) = encoder.encode_with_report("𝟙 and ⅓").unwrap();
assert_eq!(body, r"\ensuremath{\mathds{1}} and \nicefrac{1}{3}");

let mut preamble = String::new();
report.needs.write_preamble(&mut preamble).unwrap();
assert_eq!(preamble, "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n");
```

## What it offers

- **A builtin table of 1549 characters**, each with the LaTeX that prints it,
  the LaTeX mode that LaTeX is valid in, and the preamble it needs. Every entry
  compiles and sets the glyph its character stands for.
- **Preamble needs.** An encoded value carries the set of *chunks* — packages
  with their options, or declarations no package makes — that a document must
  hold. They accumulate over a whole document and are written out as a
  preamble.
- **Text mode and math mode.** The mode a value is valid in is part of the
  table, not of the string: `\alpha` is wrapped in `\ensuremath{…}` for text
  output and written bare for math output, where a text value is the one that
  gets wrapped.
- **Speed.** Runs of plain ASCII are copied in bulk without decoding a
  character or calling a rule; the builtin table is compiled into a static
  layout with nothing to do at load time; a statically typed rule chain lets
  the compiler inline and unroll the whole loop.
- **Streaming output.** Encoding appends to a sink of the caller's, so a long
  document never has to exist as one string.

## `no_std`

The crate is `#![no_std]` and uses `alloc`: it allocates strings and vectors
and nothing else. Its one dependency is `unicode-normalization`. The default
feature `std` adds a single item, the `IoOut` adapter for `std::io::Write`;
`cargo build --no-default-features` is the `no_std` build.

## Extending it

Nothing in the pipeline is fixed:

- **Rules.** A rule is offered a position in the input and answers with the
  LaTeX that replaces what it consumed there, or with "not mine". Rules are
  tried in order and the first match wins, so a rule before the default rules
  overrides them and a rule after them fills in what they lack. A rule is a
  lookup table (builtin, built at run time, or compiled from your own data at
  compile time), a closure, or any type that implements the trait — including
  one that calls into another language.
- **Protection.** What is written around a value so that it cannot merge with
  the text that follows, and so that it is valid in the output's mode. The
  standard strategy is configurable at run time; a strategy of your own is a
  trait implementation.
- **Unknown characters.** Keep, ignore, replace, spell the code point out,
  fail, or call your own function.
- **Output, reports, normalization.** The sink the LaTeX is written to, what
  is done with the preamble needs and the unknown characters, and what the
  input is normalized to before any rule sees it, are each a trait with the
  obvious implementations provided.

The crate documentation describes all of it; `cargo doc --open`, or
[docs.rs](https://docs.rs/untechxt).

## Where the data comes from

The builtin table began as a copy of pylatexenc's `defaults` conversion table
([pylatexenc](https://github.com/phfaist/pylatexenc), MIT), whose character map
was in turn adapted from
[latexcodec](https://pypi.python.org/pypi/latexcodec) (MIT). It has been
maintained by hand here since: a spelling that does not compile, or that sets
the wrong glyph, is corrected, and every departure from pylatexenc's spelling
is listed with its reason in the module documentation of
`src/builtin/default_table.rs` — which is also where both MIT notices are
reproduced in full, as they must travel with the data. What each entry needs in
the preamble is this library's own; pylatexenc records nothing about packages.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
