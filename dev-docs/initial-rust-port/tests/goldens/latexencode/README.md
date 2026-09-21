# Conformance golden of `flm-latexencode`

`uni_chars_test_previous.txt` holds one line per code point — the code point,
the character's Unicode name in brackets, and the character itself — the whole
line encoded under the `braces-almost-all` protection and the `fail`
unknown-character policy. It records what the Unicode-to-LaTeX table of
`flm-latexencode/src/tables.rs` spells each character as, and it is that
table's acceptance test: `flm-latexencode/tests/latexencode.rs` rebuilds every
line from this file, encodes it, and compares the result with this file.

**Provenance.** The file began as pylatexenc's `test/uni_chars_test_previous.txt`
(<https://github.com/phfaist/pylatexenc>, commit `e4ddf2ba`; MIT License,
Copyright (c) 2015-2023 Philippe Faist), which pylatexenc's
`test/test_latexencode_all.py` generates by encoding the line
`"0x%04X %-50s    |%s|\n"` for every code point that has a Unicode name and
keeping the lines that encode. The table is maintained in this repository, so
this file now records this table's output; the lines that differ from
pylatexenc's are the departures listed in the table's own documentation.

**Refreshing it.** Run the suite with `UPDATE_GOLDEN=1` and read the diff:

```sh
UPDATE_GOLDEN=1 cargo test -p flm-latexencode --test latexencode
```

Every changed line must be a change to the table that was meant. The code
points and the Unicode names come from this file itself — the crate carries no
database of Unicode names — so no line is ever added; a line drops out when the
table stops spelling its character, which is what the `fail` policy does to it.
