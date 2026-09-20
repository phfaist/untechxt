

pub trait ReplacementProtection {

    pub fn protected_value(& 'a self, value: &str) -> Result<String, CallbackError>;

    pub fn protected_write(& 'a self, out : & mut OutBuffer, value : & str)
                           -> Result<(), CallbackError> {
        out.push_str(self.protected_value(value)?)
    }

};




#[derive(Clone, Default)]
pub enum BracesPolicy {
    #[default]
    WhenNecessary,
    Always,
    AfterMacro,
};


#[derive(Clone, Default)]
pub struct ReplacementProtectionBraced {
    
    pub enum all : bool

    #[default]
    Braced,
    /// Wrap every spelling in braces, whatever its form — an empty spelling
    /// becomes `{}`. pylatexenc's `'braces-all'`.
    BracesAll,
    /// Wrap every spelling that starts with a backslash in braces, so `\'e`
    /// becomes `{\'e}` but `''` stays. pylatexenc's `'braces-almost-all'`,
    /// which reproduces the behavior of its version-1 encoder.
    BracesAlmostAll,
    /// Append `{}` to a spelling that ends in a named command, under the same
    /// test as [`Braces`](Protection::Braces): `\textemdash` becomes
    /// `\textemdash{}`. pylatexenc's `'braces-after-macro'`.
    BracesAfterMacro,
    /// Write every spelling as it is. Unsafe in general, since `\l` before a
    /// letter is then a different command. pylatexenc's `'none'`.
    None,
    /// A function of the caller's, given the spelling and returning what is
    /// written in its place; it is applied to every spelling, the empty one
    /// included.
    Custom(Arc<dyn Fn(&str) -> String + Send + Sync>),
}

impl Protection {
    /// Whether `latex` ends in a named command: its last backslash is followed
    /// by one or more letters and nothing else.
    fn ends_in_named_command(latex: &str) -> bool {
        match latex.rfind('\\') {
            Some(k) => {
                let name = &latex[k + 1..];
                !name.is_empty() && name.chars().all(char::is_alphabetic)
            }
            None => false,
        }
    }

    /// The spelling `latex` with this protection applied, as a new string.
    ///
    /// This is exactly what the encoder applies to every spelling a rule
    /// produces (under the rule's own protection if it has one, else the
    /// encoder's). A caller that writes spellings itself — from
    /// [`spelling_of`] or from a table of its own — protects them with it, so
    /// that they can no more merge with the letters after them than the
    /// encoder's output can. The encoder does not pass the output of an
    /// [`UnknownCharPolicy`] through it.
    ///
    /// # Panics
    ///
    /// Raises no panic of its own; a panic inside the function of a
    /// [`Protection::Custom`] is not caught.
    ///
    /// ```
    /// use flm_latexencode::Protection;
    ///
    /// assert_eq!(Protection::Braces.apply(r"\textemdash"), r"{\textemdash}");
    /// assert_eq!(Protection::Braces.apply(r"\'e"), r"\'e");
    /// assert_eq!(Protection::BracesAfterMacro.apply(r"\l"), r"\l{}");
    /// ```
    pub fn apply(&self, latex: &str) -> String {
        let mut out = String::with_capacity(latex.len() + 2);
        self.apply_into(&mut out, latex);
        out
    }

    /// Append the spelling `latex` to `out` with this protection applied: the
    /// same as [`apply`](Protection::apply), into a string the caller owns.
    ///
    /// # Panics
    ///
    /// As for [`apply`](Protection::apply).
    pub fn apply_into(&self, out: &mut String, latex: &str) {
        let braced = |out: &mut String| {
            out.push('{');
            out.push_str(latex);
            out.push('}');
        };
        match self {
            Protection::Braces if Protection::ends_in_named_command(latex) => braced(out),
            Protection::BracesAll => braced(out),
            Protection::BracesAlmostAll if latex.starts_with('\\') => braced(out),
            Protection::BracesAfterMacro => {
                out.push_str(latex);
                if Protection::ends_in_named_command(latex) {
                    out.push_str("{}");
                }
            }
            Protection::Custom(f) => out.push_str(&f(latex)),
            Protection::Braces | Protection::BracesAlmostAll | Protection::None => {
                out.push_str(latex);
            }
        }
    }
}

impl fmt::Debug for Protection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Protection::Braces => "Braces",
            Protection::BracesAll => "BracesAll",
            Protection::BracesAlmostAll => "BracesAlmostAll",
            Protection::BracesAfterMacro => "BracesAfterMacro",
            Protection::None => "None",
            Protection::Custom(_) => "Custom(..)",
        })
    }
}
