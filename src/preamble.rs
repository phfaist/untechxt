


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkPreamble {
    /// A latex package that should be loaded (without any package options),
    /// e.g. `Package("amsmath")` for `\usepackage{amsmath}`.
    Package(&'static str, &'static str),

    /// A LaTeX package that needs to be loaded with a specific option string it
    /// should be loaded with.  Speicfy, e.g. `PackageWithOptions("fontenc",
    /// "T1")` for `\usepackage[T1]{fontenc}`.
    PackageWithOptions(&'static str, &'static str),

    /// Additional custom definitions that need to be included in the preamble
    /// which are more complex than a simple package inclusion.  For example,
    /// defining a math alphabet
    /// (`\DeclareMathAlphabet{\mybold}{U}{bbold}{m}{n}`), etc.
    Snippet(&'static str),
}
