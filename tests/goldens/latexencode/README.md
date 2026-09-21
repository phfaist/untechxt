# Conformance golden of `untechxt`

`uni_chars_test_previous.txt` holds one line per code point — the code point,
the character's Unicode name in brackets, and the character itself, built with
the format string `"0x%04X %-50s    |%s|\n"` — the whole line encoded with NFC
normalization, pylatexenc's `braces-almost-all` protection and the `fail`
unknown-character policy. It records what the builtin Unicode-to-LaTeX table of
`src/builtin/default_table.rs` spells each character as, and it is that table's
acceptance test: `tests/latexencode.rs` rebuilds every line from this file,
encodes it, and compares the result with this file.

`braces-almost-all` is pylatexenc's version-1 protection and is not a standard
option of this crate; the test defines it as a small `ReplacementProtection`
of its own, which also makes it the worked example of a custom strategy.
21 of these lines only encode because NFC maps the character onto one that has
a table entry.

**Provenance.** The file began as pylatexenc's `test/uni_chars_test_previous.txt`
(<https://github.com/phfaist/pylatexenc>, commit `e4ddf2ba`; MIT License,
Copyright (c) 2015-2023 Philippe Faist), which pylatexenc's
`test/test_latexencode_all.py` generates by encoding that line for every code
point that has a Unicode name and keeping the lines that encode. The table is
maintained in this repository, so this file now records this table's output;
the lines that differ from pylatexenc's are the departures listed in the
table's own documentation.

It reached this crate by way of FLM's `flm-latexencode`, kept here as
`initial-rust-port/`. The only change made on the way was the rename of the
`\flm` command prefix to `\UnxT`, which touched 68 lines and which
`tools/migrate_tables.py` applied; every other byte is the earlier file's.

**Refreshing it.** Run the suite with `UPDATE_GOLDEN=1` and read the diff:

```sh
UPDATE_GOLDEN=1 cargo test --test latexencode
```

**Never regenerate the golden to make a test pass.** Every changed line must be
a change to the table that was meant, and the diff is reviewed line by line.
The code points and the Unicode names come from this file itself — the crate
carries no database of Unicode names — so no line is ever added; a line drops
out when the table stops spelling its character, which is what the `fail`
policy does to it.
