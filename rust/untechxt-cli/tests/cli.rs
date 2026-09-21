//! Tests of the command line: every option, over fixture files, asserting
//! standard output, standard error and the exit code.
//!
//! These tests run the program itself rather than the library, because what is
//! worth testing here is what the library does not do: the parsing of the
//! arguments, the two output streams, the files, and the three exit codes.
//! `CARGO_BIN_EXE_untechxt` is the binary that cargo built for this test, so
//! no test dependency is needed to find it.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// What one run of the program produced.
struct Run {
    /// The exit code: 0 when the input was encoded, 1 when it could not be
    /// encoded, 2 when the program could not do its job.
    code: i32,
    /// Everything that was written to standard output, which is the LaTeX
    /// unless `-o` sent the LaTeX to a file.
    stdout: String,
    /// Everything that was written to standard error: the report about the
    /// encoding, or the one-line failure message.
    stderr: String,
}

/// Runs the program with `args`, and with `stdin` on its standard input.
///
/// Both output streams are captured whole. A write to the standard input of
/// the child is allowed to fail, because `--help` and a usage error make the
/// child exit before it reads anything, and the broken pipe that follows is
/// not what those tests are about.
fn run_with_stdin(args: &[&str], stdin: &str) -> Run {
    let mut child = Command::new(env!("CARGO_BIN_EXE_untechxt"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary that cargo built for this test runs");
    {
        let mut pipe = child.stdin.take().expect("standard input was piped");
        let _ = pipe.write_all(stdin.as_bytes());
        // Dropping the pipe here closes it. Without the end of the input the
        // child would wait for more forever.
    }
    let output = child.wait_with_output().expect("the child was waited for");
    Run {
        code: output
            .status
            .code()
            .expect("the child exited rather than being signalled"),
        stdout: String::from_utf8(output.stdout).expect("the LaTeX is UTF-8"),
        stderr: String::from_utf8(output.stderr).expect("the report is UTF-8"),
    }
}

/// Runs the program with `args` and nothing on standard input.
fn run(args: &[&str]) -> Run {
    run_with_stdin(args, "")
}

/// The path of one of the fixture files that are checked in beside this test.
fn fixture(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(name);
    path.into_os_string().into_string().expect("the fixture path is UTF-8")
}

/// A scratch path under the temporary directory of cargo, named after `name`.
///
/// Cargo sets `CARGO_TARGET_TMPDIR` for integration tests precisely so that
/// they need no temporary-file dependency. The directory outlives the run, so
/// each test uses a name of its own and overwrites its own file.
fn scratch(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    std::fs::create_dir_all(&path).expect("the scratch directory exists");
    path.push(name);
    let _ = std::fs::remove_file(&path);
    path
}

/// The scratch path as the string that it is passed as an argument as.
fn argument(path: &std::path::Path) -> &str {
    path.to_str().expect("the scratch path is UTF-8")
}

/// Runs the program with `args` and asserts all three of its results.
///
/// Every test that does not need a file goes through here, so that no test can
/// quietly ignore a stream: an option that started printing a warning would
/// fail the tests that expect none.
fn assert_run(args: &[&str], stdout: &str, stderr: &str, code: i32) {
    let result = run(args);
    assert_eq!(result.stdout, stdout, "standard output of {args:?}");
    assert_eq!(result.stderr, stderr, "standard error of {args:?}");
    assert_eq!(result.code, code, "exit code of {args:?}");
}

// --------------------------------------------------------- input and output

#[test]
fn standard_input_is_encoded_when_no_file_is_named() {
    let result = run_with_stdin(&[], "Café — 100%\n");
    assert_eq!(result.stdout, "Caf\\'e {\\textemdash} 100\\%\n");
    assert_eq!(result.stderr, "");
    assert_eq!(result.code, 0);
}

#[test]
fn a_file_argument_is_encoded_to_standard_output() {
    assert_run(&[&fixture("cafe.txt")], "Caf\\'e {\\textemdash} 100\\%\n", "", 0);
}

#[test]
fn several_files_are_encoded_one_after_the_other() {
    let alpha = fixture("alpha.txt");
    let beta = fixture("beta.txt");
    assert_run(
        &[&alpha, &beta],
        "\\ensuremath{\\alpha}\n\\ensuremath{\\beta}\n",
        "",
        0,
    );
    // A file named `-` is standard input, and it keeps its place among the
    // files rather than coming first or last.
    let result = run_with_stdin(&[&alpha, "-", &beta], "γ\n");
    assert_eq!(
        result.stdout,
        "\\ensuremath{\\alpha}\n\\ensuremath{\\gamma}\n\\ensuremath{\\beta}\n"
    );
    assert_eq!(result.stderr, "");
    assert_eq!(result.code, 0);
}

#[test]
fn the_output_flag_writes_a_file_and_leaves_standard_output_empty() {
    let output = scratch("output-flag.tex");
    assert_run(
        &[&fixture("cafe.txt"), "-o", argument(&output)],
        "",
        "",
        0,
    );
    assert_eq!(
        std::fs::read_to_string(&output).expect("the output file was written"),
        "Caf\\'e {\\textemdash} 100\\%\n"
    );
}

#[test]
fn a_file_that_cannot_be_read_is_a_failure_and_not_a_panic() {
    let missing = scratch("no-such-input.txt");
    let result = run(&[argument(&missing)]);
    assert_eq!(result.stdout, "");
    assert!(
        result.stderr.starts_with("untechxt: "),
        "the failure names the program: {:?}",
        result.stderr
    );
    assert!(
        result.stderr.contains("no-such-input.txt"),
        "the failure names the file: {:?}",
        result.stderr
    );
    assert_eq!(result.code, 2);
}

#[test]
fn input_that_is_not_utf8_is_a_failure_that_names_the_file() {
    let result = run(&[&fixture("not-utf8.bin")]);
    assert_eq!(result.stdout, "");
    assert!(
        result.stderr.contains("not-utf8.bin"),
        "the failure names the file: {:?}",
        result.stderr
    );
    assert!(
        result.stderr.contains("UTF-8"),
        "the failure says what is wrong with the file: {:?}",
        result.stderr
    );
    assert_eq!(result.code, 2);
}

#[test]
fn an_output_file_that_cannot_be_written_is_a_failure_and_not_a_panic() {
    let unwritable = scratch("no-such-directory/output.tex");
    let result = run(&[&fixture("cafe.txt"), "-o", argument(&unwritable)]);
    assert_eq!(result.stdout, "");
    assert!(result.stderr.starts_with("untechxt: "), "{:?}", result.stderr);
    assert_eq!(result.code, 2);
}

// ----------------------------------------------------------- which rule runs

#[test]
fn the_non_ascii_only_flag_keeps_the_latex_of_the_input() {
    let latex = fixture("latex.txt");
    // The input is LaTeX already: the backslash, the braces, the ampersand
    // and the percent sign stay as they are, and only the accented letter is
    // encoded.
    assert_run(
        &[&latex, "--non-ascii-only"],
        "\\emph{Caf\\'e} & 100%\n",
        "",
        0,
    );
    // Without the flag, the special characters are encoded as well.
    assert_run(
        &[&latex],
        "{\\textbackslash}emph\\{Caf\\'e\\} \\& 100\\%\n",
        "",
        0,
    );
    // The two flags override each other, so the last one on the command line
    // is the one that counts.
    assert_run(
        &[&latex, "--non-ascii-only", "--no-non-ascii-only"],
        "{\\textbackslash}emph\\{Caf\\'e\\} \\& 100\\%\n",
        "",
        0,
    );
    assert_run(
        &[&latex, "--no-non-ascii-only", "--non-ascii-only"],
        "\\emph{Caf\\'e} & 100%\n",
        "",
        0,
    );
}

// ------------------------------------------------------ protection and modes

#[test]
fn every_replacement_protection_strategy_writes_its_own_syntax() {
    let cafe = fixture("cafe.txt");
    let protection = |strategy: &str| {
        let result = run(&[&cafe, "--replacement-protection", strategy]);
        assert_eq!(result.stderr, "", "standard error of {strategy}");
        assert_eq!(result.code, 0, "exit code of {strategy}");
        result.stdout
    };
    // `\textemdash` ends with a macro name and is the value that each
    // strategy treats differently. `\'e` and `\%` end with a character that
    // no macro name can contain, so only `braces-all` adds anything to them.
    assert_eq!(protection("braces"), "Caf\\'e {\\textemdash} 100\\%\n");
    assert_eq!(protection("braces-after"), "Caf\\'e \\textemdash{} 100\\%\n");
    assert_eq!(protection("braces-all"), "Caf{\\'e} {\\textemdash} 100{\\%}\n");
    // The space of the strategy is written before the space of the input,
    // which is why the strategy is unsafe outside math mode: TeX would read
    // the two as one and the word spacing of the text would be lost.
    assert_eq!(protection("space"), "Caf\\'e \\textemdash  100\\%\n");
    assert_eq!(protection("none"), "Caf\\'e \\textemdash 100\\%\n");
    // Without the option, text mode writes braces.
    assert_run(&[&cafe], "Caf\\'e {\\textemdash} 100\\%\n", "", 0);
}

#[test]
fn the_math_mode_flag_encodes_for_mathematics() {
    let mixed = fixture("mathmix.txt");
    // In text mode every math value is wrapped once and the accented letter
    // is written as it is.
    assert_run(
        &[&mixed],
        "\\ensuremath{\\alpha}\\ensuremath{\\leq}\\ensuremath{\\beta}\\'e\n",
        "",
        0,
    );
    // In math mode the math values go in bare, followed by the space that the
    // mode's own strategy writes, and the accented letter is the value that
    // has to be wrapped.
    assert_run(
        &[&mixed, "--math-mode"],
        "\\alpha \\leq \\beta \\textnormal{\\'e}\n",
        "",
        0,
    );
    // An explicit strategy replaces the default of the mode. The mode itself
    // stays, so the accented letter is still wrapped.
    assert_run(
        &[&mixed, "--math-mode", "--replacement-protection", "braces"],
        "{\\alpha}{\\leq}{\\beta}\\textnormal{\\'e}\n",
        "",
        0,
    );
}

// ---------------------------------------------------------- unknown characters

/// The warning that the fixture `unknown.txt` produces.
const UNKNOWN_WARNING: &str =
    "untechxt: warning: no known LaTeX representation for: U+0E18 'ธ', U+4E2D '中'\n";

#[test]
fn every_unknown_char_policy_writes_its_own_output() {
    let unknown = fixture("unknown.txt");
    // Every policy but `fail` encodes the text and warns about the
    // characters, which is a warning and not a failure: the exit code stays
    // 0 because the LaTeX has been written.
    assert_run(&[&unknown], "ธ and 中\n", UNKNOWN_WARNING, 0);
    assert_run(
        &[&unknown, "--unknown-char-policy", "keep"],
        "ธ and 中\n",
        UNKNOWN_WARNING,
        0,
    );
    assert_run(
        &[&unknown, "--unknown-char-policy", "ignore"],
        " and \n",
        UNKNOWN_WARNING,
        0,
    );
    assert_run(
        &[&unknown, "--unknown-char-policy", "replace"],
        "{\\bfseries ?} and {\\bfseries ?}\n",
        UNKNOWN_WARNING,
        0,
    );
    assert_run(
        &[&unknown, "--unknown-char-policy", "unihex"],
        "\\ensuremath{\\langle}\\texttt{U+0E18}\\ensuremath{\\rangle} and \
         \\ensuremath{\\langle}\\texttt{U+4E2D}\\ensuremath{\\rangle}\n",
        UNKNOWN_WARNING,
        0,
    );
}

#[test]
fn a_control_character_is_warned_about_by_its_code_point_alone() {
    // A terminal would not print the character itself, so the warning names
    // the code point and nothing else.
    let result = run_with_stdin(&[], "a\u{1}b\n");
    assert_eq!(result.stdout, "a\u{1}b\n");
    assert_eq!(
        result.stderr,
        "untechxt: warning: no known LaTeX representation for: U+0001\n"
    );
    assert_eq!(result.code, 0);
}

#[test]
fn the_fail_policy_writes_nothing_at_all() {
    let output = scratch("fail-policy.tex");
    let preamble = scratch("fail-policy-preamble.tex");
    let result = run(&[
        &fixture("unknown.txt"),
        "--unknown-char-policy",
        "fail",
        "-o",
        argument(&output),
        "--preamble",
        argument(&preamble),
    ]);
    assert_eq!(result.stdout, "");
    assert!(
        result.stderr.contains("U+0E18"),
        "the failure names the character: {:?}",
        result.stderr
    );
    assert!(
        result.stderr.contains("unknown.txt"),
        "the failure names the file the character is in: {:?}",
        result.stderr
    );
    assert_eq!(result.code, 1, "the input could not be encoded");
    // Nothing is written before every input has been encoded, so neither file
    // was created.
    assert!(!output.exists(), "the output file was not created");
    assert!(!preamble.exists(), "the preamble file was not created");
}

// -------------------------------------------------------------- the preamble

#[test]
fn the_preamble_flag_writes_the_file_and_silences_the_note() {
    let preamble = scratch("preamble-flag.tex");
    assert_run(
        &[&fixture("symbols.txt"), "--preamble", argument(&preamble)],
        "\\ensuremath{\\mathds{1}} and \\nicefrac{1}{3}\n",
        "",
        0,
    );
    assert_eq!(
        std::fs::read_to_string(&preamble).expect("the preamble file was written"),
        "\\usepackage{dsfont}\n\\usepackage{nicefrac}\n"
    );
}

#[test]
fn the_preamble_file_is_written_even_when_nothing_is_needed() {
    let preamble = scratch("preamble-empty.tex");
    assert_run(
        &[&fixture("cafe.txt"), "--preamble", argument(&preamble)],
        "Caf\\'e {\\textemdash} 100\\%\n",
        "",
        0,
    );
    // A document can `\input` the file whatever the input was, which is why
    // the file is written even when it is empty.
    assert_eq!(
        std::fs::read_to_string(&preamble).expect("the preamble file was written"),
        ""
    );
}

/// What `ą я` is encoded as, whatever the engine.
const ENGINES_STDOUT: &str = "\\k{a} {\\fontencoding{T2A}\\selectfont\\cyrya}\n";

/// The line that the report starts what the preamble needs with.
const PREAMBLE_NOTE: &str =
    "untechxt: the output needs the following in the document preamble:\n";

#[test]
fn the_preamble_is_written_for_every_engine_by_default() {
    // The ogonek accent needs the font encoding `T1` under pdfLaTeX alone, and
    // the Cyrillic letter needs `T2A` beside the encoding of the document,
    // which depends on the engine. The preamble tests the engine for both.
    let result = run_with_stdin(&[], "ą я\n");
    assert_eq!(result.stdout, ENGINES_STDOUT);
    assert_eq!(
        result.stderr,
        format!(
            "{PREAMBLE_NOTE}\\usepackage{{iftex}}\n\\iftutex\n\
             \\usepackage[T2A,TU]{{fontenc}}\n\\else\n\\usepackage[T1]{{fontenc}}\n\
             \\usepackage[T2A,T1]{{fontenc}}\n\\fi\n"
        )
    );
    assert_eq!(result.code, 0);

    // `--engine any` is that default, spelled out.
    let spelled_out = run_with_stdin(&["--engine", "any"], "ą я\n");
    assert_eq!(spelled_out.stderr, result.stderr);
}

#[test]
fn the_engine_option_writes_the_preamble_of_one_engine() {
    let preamble_for = |engine: &str| {
        let result = run_with_stdin(&["--engine", engine], "ą я\n");
        // The LaTeX itself is the same for every engine.
        assert_eq!(result.stdout, ENGINES_STDOUT, "standard output under {engine}");
        assert_eq!(result.code, 0, "exit code under {engine}");
        result.stderr
    };
    assert_eq!(
        preamble_for("pdflatex"),
        format!(
            "{PREAMBLE_NOTE}\\usepackage[T1]{{fontenc}}\n\\usepackage[T2A,T1]{{fontenc}}\n"
        )
    );
    let unicode = format!("{PREAMBLE_NOTE}\\usepackage[T2A,TU]{{fontenc}}\n");
    assert_eq!(preamble_for("lualatex"), unicode);
    assert_eq!(preamble_for("xelatex"), unicode);
}

#[test]
fn the_engine_option_applies_to_the_preamble_file() {
    let preamble = scratch("preamble-engine.tex");
    let result = run_with_stdin(
        &["--engine", "lualatex", "--preamble", argument(&preamble)],
        "ą я\n",
    );
    assert_eq!(result.stdout, ENGINES_STDOUT);
    assert_eq!(result.stderr, "");
    assert_eq!(result.code, 0);
    assert_eq!(
        std::fs::read_to_string(&preamble).expect("the preamble file was written"),
        "\\usepackage[T2A,TU]{fontenc}\n"
    );
}

#[test]
fn nothing_is_reported_when_the_engine_needs_nothing() {
    // LuaLaTeX defines the ogonek accent itself, so the output needs nothing
    // under it, and the report has nothing to say.
    let result = run_with_stdin(&["--engine", "lualatex"], "ą\n");
    assert_eq!(result.stdout, "\\k{a}\n");
    assert_eq!(result.stderr, "");
    assert_eq!(result.code, 0);

    let result = run_with_stdin(&["--engine", "pdflatex"], "ą\n");
    assert_eq!(result.stderr, format!("{PREAMBLE_NOTE}\\usepackage[T1]{{fontenc}}\n"));
}

#[test]
fn an_unknown_engine_is_a_usage_error() {
    let result = run(&["--engine", "tex"]);
    assert_eq!(result.stdout, "");
    assert!(result.stderr.contains("--engine"), "{}", result.stderr);
    assert_eq!(result.code, 2);
}

// ----------------------------------------------------- the report and -q

#[test]
fn the_report_warns_and_states_what_the_preamble_needs() {
    // Two inputs share one report, so the warning lists the characters of
    // both and the preamble covers both.
    let result = run(&[&fixture("symbols.txt"), &fixture("unknown.txt")]);
    assert_eq!(
        result.stdout,
        "\\ensuremath{\\mathds{1}} and \\nicefrac{1}{3}\nธ and 中\n"
    );
    assert_eq!(
        result.stderr,
        format!(
            "{UNKNOWN_WARNING}untechxt: the output needs the following in the \
             document preamble:\n\\usepackage{{dsfont}}\n\
             \\usepackage{{nicefrac}}\n"
        )
    );
    assert_eq!(result.code, 0);
}

#[test]
fn the_quiet_flag_silences_the_report_but_not_a_failure() {
    let symbols = fixture("symbols.txt");
    let unknown = fixture("unknown.txt");
    assert_run(
        &[&symbols, &unknown, "-q"],
        "\\ensuremath{\\mathds{1}} and \\nicefrac{1}{3}\nธ and 中\n",
        "",
        0,
    );
    assert_run(
        &[&symbols, &unknown, "--quiet"],
        "\\ensuremath{\\mathds{1}} and \\nicefrac{1}{3}\nธ and 中\n",
        "",
        0,
    );
    // A failure is the only explanation that the exit code comes with, so it
    // is printed whatever `-q` says.
    let result = run(&[&unknown, "--unknown-char-policy", "fail", "-q"]);
    assert_eq!(result.stdout, "");
    assert!(result.stderr.starts_with("untechxt: "), "{:?}", result.stderr);
    assert_eq!(result.code, 1);
}

// ------------------------------------------------------- the command itself

#[test]
fn help_and_version_are_answered_without_reading_anything() {
    let help = run(&["--help"]);
    assert_eq!(help.code, 0);
    assert!(help.stderr.is_empty());
    for option in [
        "--output",
        "--non-ascii-only",
        "--no-non-ascii-only",
        "--replacement-protection",
        "--math-mode",
        "--unknown-char-policy",
        "--preamble",
        "--quiet",
    ] {
        assert!(help.stdout.contains(option), "{option} is in the help");
    }

    let version = run(&["--version"]);
    assert_eq!(version.code, 0);
    assert!(version.stdout.starts_with("untechxt "), "{}", version.stdout);
}

#[test]
fn an_unknown_flag_is_a_usage_error() {
    let result = run(&["--encode-everything"]);
    assert_eq!(result.stdout, "");
    assert!(
        result.stderr.contains("--encode-everything"),
        "{:?}",
        result.stderr
    );
    assert_eq!(result.code, 2);
}

#[test]
fn an_unknown_value_of_an_option_is_a_usage_error() {
    let result = run(&["--unknown-char-policy", "shrug"]);
    assert_eq!(result.stdout, "");
    assert!(result.stderr.contains("shrug"), "{:?}", result.stderr);
    assert_eq!(result.code, 2);
}
