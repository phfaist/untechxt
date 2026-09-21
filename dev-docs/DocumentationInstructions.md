# Instructions for writing documentation

Applies to rustdoc comments, `README.md`, and any other user-facing prose. The
reader is a Rust developer who has never seen this crate and knows LaTeX only
casually. Write plain, conventional technical English in full sentences. The
goal is to be clear, not to be compact or clever. (These rules come from the
maintainer's rewrite of the crate-level docs in `src/lib.rs`; its first
sections are the model to imitate.)

## Wording

- Use the standard vocabulary of API docs: a function "takes" and "returns", a
  table "contains" entries, a rule "matches" input. Do not invent figurative
  verbs. Bad: "`encode` answers a new string", "a character no rule knows",
  "what the preamble must hold", "a rule hands out strings".
- Write out the full clause instead of an elliptical or mannered one.
  - Bad: "immutable — and so shareable between threads whenever its rules are".
    Good: "is immutable, and can be shared across threads provided the rules
    are thread-safe".
  - Bad: "Everything in between is open: the rules that match the input, the
    sink the LaTeX is written to, ...". Good: "The library is highly
    extensible: You may define custom rules for encodings, ... The output can
    be assembled as a string or directly written to an I/O buffer."
- Repeat the noun rather than leaning on a pronoun when several things are in
  play: "overrides information from the lookup table", not "overrides it".
- Name the kind of item next to its link: "the [`encode`] function", "the
  method [`Encoder::encode_into`]", "the builtin lookup table [`DEFAULTS`]".
  Point to related items with "(see [`OutBuffer`])".
- Prefer "custom" and direct instructions ("Create a rule from your custom
  [`LookupTable`] with [`TableRule`]") over roundabout phrasing ("a
  [`LookupTable`] of your own becomes one through [`TableRule`]").
- Go easy on em-dash asides, semicolon chains, telegraphic parentheses, and
  bold. A fragment such as "(the 13 ASCII entries alone)" should instead be a
  sentence saying what the item contains and what it is for.

## Structure

- Open with a short, plain statement of what the item is ("Unicode-to-LaTeX
  encoder."), followed by one concrete example. Keep secondary features (the
  report, the preamble) out of the opening sentence.
- Signpost where to start: "Quick start: Head to the [`encode`] function,
  which runs the encoder with some reasonable default settings ...".
- Define each term where it is first used, in a sentence or two. Do not list a
  type's components by name and move on: give each component a bullet saying
  what it is, what the default behavior is, and which alternatives exist. Never
  describe something (e.g. the quick-start function) using concepts that are
  only introduced further down.
- Motivate a non-obvious concept with a concrete example of what goes wrong
  without it. E.g. without replacement protection, `~user` would be encoded
  as the broken `\textasciitildeuser`; `{\textasciitilde}` composes correctly.
- Introduce a concept (e.g. the report) and its purpose in a paragraph of its
  own before the methods that use it.
- Present parallel alternatives (the ways to invoke the encoder, the kinds of
  rules) as a bullet list with an introducing sentence. Each bullet says what
  the item takes, what it returns, and when to pick it.
- Describe API from the caller's side (inputs, outputs, side effects, purpose),
  not through implementation relationships ("the primitive the other two are
  written with").
- When stating a limitation, say what the user can do about it: "first match
  wins" is followed by how to order or write the rules if, say, the longest
  match is wanted.
- Mention design goals that benefit the user (modular design, small compiled
  artifacts, `no_std`).
- Delete sentences that only restate a relationship in compressed form and
  teach nothing ("`encode` is that encoder's `encode` with the report
  dropped").

## Accuracy

- No hard-coded counts or other details that go stale when the data changes
  ("the builtin table of 1549 characters", "the 13 ASCII entries"). Say "a
  built-in symbol encoding table", "the few printable ASCII characters that
  have special meaning for LaTeX".
- Do not overclaim: "the compiler may unroll and inline", not "the compiler
  unrolls and inlines".
- After changing code, reread every doc passage that mentions it, including
  the crate-level overview, and fix what is no longer true. (The overview still
  called `ASCII_SPECIALS` a "view" of `DEFAULTS` after it had become a table of
  its own.)

## Layout

- Wrap prose and doc-example code at 80 columns, counting the `//! ` or `/// `
  prefix. Break a long example statement by hand inside the call parentheses
  rather than the way rustfmt would:

  ```rust
  //! let math = Encoder::new(&DEFAULTS).with_protection(
  //!     StandardProtection::math_mode()
  //! );
  ```
- Leave a blank line between the crate-level `//!` block and the first
  attribute or item.
