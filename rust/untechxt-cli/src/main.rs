//! The `untechxt` command-line encoder.
//!
//! Reads Unicode text and writes it out as LaTeX source, using the
//! [`untechxt`] library. This is the std-linking program around a `no_std`
//! library, and what the library cannot do is here: the file and stream I/O,
//! the report on standard error, and the exit code.
//!
//! The program is one pass. It reads every input, encodes all of the inputs
//! into one string and one shared report, writes the LaTeX, writes the
//! preamble file, and prints the report. Nothing is written before every
//! input has been encoded, so a failure leaves no half-written output behind.

mod cli;

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use untechxt::preamble::{Engine, PreambleNeeds};
use untechxt::report::EncodeReport;
use untechxt::{EncodeError, Encoder};

use cli::Cli;

/// The name that every message on standard error starts with.
const PROGRAM: &str = "untechxt";

/// The name that standard input is reported under, in the place of the path
/// of a file.
const STDIN: &str = "<stdin>";

/// Exit code 0: the input was encoded. A warning about the characters that
/// have no known LaTeX representation does not change the exit code, because
/// the LaTeX has still been written.
const EXIT_OK: u8 = 0;

/// Exit code 1: the input could not be encoded. Today that means a character
/// that has no known LaTeX representation under `--unknown-char-policy fail`.
const EXIT_UNENCODABLE: u8 = 1;

/// Exit code 2: the program could not do its job. A file that could not be
/// read or written, an input that is not UTF-8, or a command line that clap
/// rejected, which clap itself ends with this code.
const EXIT_FAILED: u8 = 2;

/// Parses the command line, encodes, and reports.
///
/// The work is in [`run`]. This turns the answer of [`run`] into an exit code,
/// and it is the one frame without a `?` in it, where a failure of the report
/// itself can still be printed.
fn main() -> ExitCode {
    match run(&Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(failure) => {
            // The failure message is the only explanation that the exit code
            // comes with, so `-q`, which suppresses the report about the
            // input, does not suppress the message. A standard error that
            // cannot be written to does.
            let _ = writeln!(std::io::stderr(), "{PROGRAM}: {}", failure.message);
            ExitCode::from(failure.code)
        }
    }
}

/// Encodes what the arguments ask for. `Ok` carries the exit code, and `Err`
/// carries the message to print and the exit code that goes with it.
fn run(cli: &Cli) -> Result<u8, Failure> {
    let inputs = read_inputs(&cli.files)?;

    let encoder = Encoder::new(cli.rules())
        .with_protection(cli.protection())
        .with_unknown_chars(cli.unknown_chars());
    // One output string and one report for all the inputs together: the
    // method `encode_into` appends to both, so the inputs are written one
    // after the other and the document they make up has a single preamble.
    let mut latex = String::new();
    let mut report = EncodeReport::new();
    for input in &inputs {
        encoder
            .encode_into(&input.text, &mut latex, &mut report)
            .map_err(|error| Failure::encoding(&input.name, &error))?;
    }

    write_output(cli.output.as_deref(), &latex)?;
    let preamble = preamble_text(&report.needs, cli.engine());
    if let Some(path) = cli.preamble.as_deref() {
        write_preamble(path, &preamble)?;
    }
    if !cli.quiet {
        // Printed after the LaTeX has been written: when the output is a pipe
        // that has gone away, the caller learns about that failure rather
        // than about the characters of the input.
        print_report(&report, &preamble, cli.preamble.is_some())?;
    }
    Ok(EXIT_OK)
}

/// One input that was read, together with the name it is reported under.
struct Input {
    /// The name of the input in a message: the path of the file, or `<stdin>`
    /// for standard input.
    name: String,
    /// The text that was read.
    text: String,
}

/// Reads every input, in the order in which the arguments name them. When the
/// arguments name no file at all, this reads standard input.
fn read_inputs(files: &[PathBuf]) -> Result<Vec<Input>, Failure> {
    if files.is_empty() {
        return Ok(vec![read_stdin()?]);
    }
    files.iter().map(|path| read_input(path)).collect()
}

/// Reads one input: standard input when `path` is `-`, and the file it names
/// otherwise.
///
/// The input must be UTF-8. Input that is not UTF-8 is reported as the I/O
/// failure that the standard library returns for it, and the message names
/// the file.
fn read_input(path: &Path) -> Result<Input, Failure> {
    if path.as_os_str() == "-" {
        return read_stdin();
    }
    let text = std::fs::read_to_string(path)
        .map_err(|error| Failure::failed(format!("{}: {error}", path.display())))?;
    Ok(Input { name: path.display().to_string(), text })
}

/// Reads all of standard input.
fn read_stdin() -> Result<Input, Failure> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|error| Failure::failed(format!("{STDIN}: {error}")))?;
    Ok(Input { name: STDIN.into(), text })
}

/// Writes the LaTeX to the file that `--output` names, or to standard output
/// when it names none.
///
/// Standard output is flushed here, because a flush that fails while the
/// process exits would report success for text that nobody received.
fn write_output(file: Option<&Path>, latex: &str) -> Result<(), Failure> {
    match file {
        Some(path) => std::fs::write(path, latex)
            .map_err(|error| Failure::failed(format!("{}: {error}", path.display()))),
        None => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            stdout
                .write_all(latex.as_bytes())
                .and_then(|()| stdout.flush())
                .map_err(|error| Failure::failed(format!("<stdout>: {error}")))
        }
    }
}

/// Returns what the output needs in the document preamble, as the lines to
/// put there: for `engine` alone, or for every engine when `--engine` named
/// none. The text is empty when the output needs nothing.
fn preamble_text(needs: &PreambleNeeds, engine: Option<Engine>) -> String {
    let mut preamble = String::new();
    let written = match engine {
        Some(engine) => needs.write_preamble_for(engine, &mut preamble),
        None => needs.write_preamble(&mut preamble),
    };
    // The library returns the error of the output it writes to, and a
    // `String` never returns one.
    written.expect("writing to a string does not fail");
    preamble
}

/// Writes `preamble`, which is what the output needs in the document
/// preamble, to the file that `--preamble` names.
///
/// The file is written whenever the encoding succeeded, even when the output
/// needs nothing and the file is therefore empty, so that a document can
/// `\input` the file unconditionally.
fn write_preamble(path: &Path, preamble: &str) -> Result<(), Failure> {
    std::fs::write(path, preamble)
        .map_err(|error| Failure::failed(format!("{}: {error}", path.display())))
}

/// Prints the report about the encoding to standard error.
///
/// The report has two parts, and each part is printed only when there is
/// something to say. The first part is a warning that lists the characters
/// that have no known LaTeX representation. The second part is `preamble`,
/// which is what the output needs in the document preamble. It is printed only
/// when `--preamble` named no file for it; `preamble_written` says whether it
/// did. The output can need nothing under the engine that `--engine` names
/// even when it needs something under another engine, and `preamble` is then
/// empty.
fn print_report(
    report: &EncodeReport,
    preamble: &str,
    preamble_written: bool,
) -> Result<(), Failure> {
    let mut message = String::new();
    if !report.unknown_chars.is_empty() {
        let characters: Vec<String> =
            report.unknown_chars.iter().copied().map(describe_char).collect();
        message.push_str(PROGRAM);
        message.push_str(": warning: no known LaTeX representation for: ");
        message.push_str(&characters.join(", "));
        message.push('\n');
    }
    if !preamble_written && !preamble.is_empty() {
        message.push_str(PROGRAM);
        message.push_str(": the output needs the following in the document preamble:\n");
        message.push_str(preamble);
    }
    if message.is_empty() {
        return Ok(());
    }

    let stderr = std::io::stderr();
    let mut stderr = stderr.lock();
    stderr
        .write_all(message.as_bytes())
        .and_then(|()| stderr.flush())
        .map_err(|error| Failure::failed(format!("<stderr>: {error}")))
}

/// Spells one character out for the warning: its code point, and the
/// character itself. A control character is shown as its code point alone,
/// because a terminal would not print the character.
fn describe_char(ch: char) -> String {
    if ch.is_control() {
        format!("U+{:04X}", ch as u32)
    } else {
        format!("U+{:04X} '{ch}'", ch as u32)
    }
}

/// A message to print on standard error, together with the exit code that
/// goes with it.
struct Failure {
    /// What went wrong. It is printed after `untechxt: `.
    message: String,
    /// The exit code that the program ends with.
    code: u8,
}

impl Failure {
    /// The program could not do its job: a file could not be read or written,
    /// or an input was not UTF-8. This is [`EXIT_FAILED`].
    fn failed(message: String) -> Self {
        Failure { message, code: EXIT_FAILED }
    }

    /// The encoding of the input named `name` stopped with `error`.
    ///
    /// A character that has no known LaTeX representation, under
    /// `--unknown-char-policy fail`, is the input that cannot be encoded, and
    /// so it is [`EXIT_UNENCODABLE`]. Every other error is the program
    /// failing at its job, and so it is [`EXIT_FAILED`]: a rule that failed,
    /// or an output that refused what was written to it.
    fn encoding(name: &str, error: &EncodeError) -> Self {
        let code = match error {
            EncodeError::UnknownChar { .. } => EXIT_UNENCODABLE,
            _ => EXIT_FAILED,
        };
        Failure { message: format!("{name}: {error}"), code }
    }
}
