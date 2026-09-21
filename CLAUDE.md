# untechxt

Unicode-to-LaTeX encoder in Rust: fast, lightweight (`no_std` + `alloc`), and
very extensible. A redesign based on pylatexenc's `latexencode`. The earlier
quick-and-dirty port lives in `dev-docs/initial-rust-port/` (reference only; its
data tables, tests, and golden file are the parts worth carrying over).

The cargo workspace is `rust/` (a sibling of possible future bindings); run
every cargo command from there. The library crate is `rust/untechxt/`.

## Working conventions

- **Questions:** do not use the interactive question box (`AskUserQuestion`).
  Explain the question clearly in prose and ask for an answer in prose.
- **Language:** US English everywhere — code, comments, docs, and chat
  ("behavior", not "behaviour").
- **Agents:** run all subagents (exploration, research, ...) with the Opus
  model.

- Never run `cargo fmt`.

## Writing documentation

- Apply instructions in `dev-docs/DocumentationInstructions.md` whenever writing
  code documentation.
