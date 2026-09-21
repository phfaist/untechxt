# untechxt

Unicode-to-LaTeX encoder.  This library turns text such as `Café — α ≤ β` into
the LaTeX source `Caf\'e {\textemdash} \ensuremath{\alpha} \ensuremath{\leq}
\ensuremath{\beta}`.  It is a Rust (`no_std`+alloc) rewrite/redesign of
[pylatexenc](https://github.com/phfaist/pylatexenc)'s `latexencode` module.

LaTeX source typically only tolerates a restricted set of characters in its
input, and some of those characters have a special meaning (notably the
escape character `\\`). This library provides an *encoding* of an entire
input string, meaning that it replaces each character that LaTeX either
rejects or would take special action on, by some LaTeX code that displays
that character.

This library also reports any preamble definitions/usepackage commands you
should include to accompany the generated latex-encoded content.  This library
is also highly extensible so you can define your own encoding rules and behavior
hooks, e.g., for unknown characters.


## Quick start

Use the `encode()` function, which runs the encoder with some reasonable default
settings and a built-in symbol encoding table.

```rust
use untechxt::encode;
//!
assert_eq!(encode("Café — naïve"), r#"Caf\'e {\textemdash} na\"ive"#);
assert_eq!(encode("100% & more"), r"100\% \& more");
```

The encoding is handled by an `Encoder` object.  Its `encode_with_report()`
method reports any required preamble definitions for your LaTeX document, along
with the encoded latex content:

```rust
use untechxt::{default_rules, Encoder};

// An encoder, and what its output needs in the preamble.
let encoder = Encoder::new(default_rules());
let (body, report) = encoder.encode_with_report("𝟙 and ⅓").unwrap();
assert_eq!(body, r"\ensuremath{\mathds{1}} and \nicefrac{1}{3}");

let mut preamble = String::new();
report.needs.write_preamble(&mut preamble).unwrap();
assert_eq!(preamble, "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n");
```

The `Encoder` can be customized with your own encoding rules (for instance, with
lookup tables or custom callback functions), your preferred policy when unknown
characters are encountered, along with more options.


## Documentation

Use `cargo doc` to generate the API documentation with all the fun details!


## Crate Dependencies and Features

The crate is compatible with `no_std` + `alloc` (uses strings and vectors).  Its
single dependency is `unicode-normalization`.

The feature `std` (on by default) adds support for streaming in `std::io::Write`
without creating temporary owned strings.  Disable the `std` feature for a
`no_std` build; this is achieved with the command `cargo build
--no-default-features`.


## Extending and Customizing the Encoder

Different steps of the pipeline can be customized and extended.

- A *rule* specifies how an input character, or an input substring, is
  mapped to a LaTeX encoded value.  A special rule type, [`RuleChain`],
  tries several rules in order until the first match; it should be used
  whenever the encoder should apply multiple rules.
  
  Each rule can be a lookup table, a callback function, or anything that
  implements the `Rule` rust trait.

- *Replacement protection* refers to additional syntax applied to the LaTeX
  encoded symbol to ensure the generated LaTeX code is valid.  For instance,
  the encoder might replace `'~'` by `'\textasciitilde'`; if no further
  processing happened, the text `'~user'` would be encoded incorrectly as
  `'\textasciitildeuser'`. Standard replacement protection strategies ensure
  that such symbols are represented for instance as `{\textasciitilde}`,
  which composes correctly with surrounding strings.

  The standard strategy is configurable at run time; provide your own strategy
  by implementing a trait.

- Input normalization: Preprocessing applied to the string before applying
  the rules; by default, a unicode normalize step.

- *Unknown char policy*: How to handle a non-ASCII character or non-printable
  character in the input for which the rule didn't apply (or for which none of
  the rules of a rule chain applied).  By default, the character is left in the
  output. Other possible behaviors include: Fail with an error, replace by a
  fixed string, provide a custom callback function.

- *Output and reports:* The encoded LaTeX code is written incrementally to an
  output sink specified by a generic trait, so the encoded value can be appended
  to a string or streamed to an I/O buffer. Similarly, information for the
  report (preamble definitions, encountered unknown chars) can be accumulated
  to any object with a custom trait implementation.


## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

The builtin table was adapted from
[pylatexenc](https://github.com/phfaist/pylatexenc)'s (MIT license), whose
character map was in turn adapted from
[latexcodec](https://pypi.python.org/pypi/latexcodec) (MIT license).  See
`src/builtin/default_table.rs` for more complete license information.
