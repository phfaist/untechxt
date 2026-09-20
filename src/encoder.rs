

/// Abstract output assembly.  Can be overridden, for instance, if you want to
/// output directly to some I/O stream instead of assembling a string first.
pub trait OutBuffer {
    pub fn push_str(&mut self, s : &str) -> Result<(), CallbackError>;
    pub fn push_char(&mut self, c : char) -> Result<(), CallbackError>;
}

impl OutBuffer for String {
    pub fn push_str(&mut self, s : &str) -> Result<(), CallbackError> {
        self.push_str(s);
        Ok(())
    }
    pub fn push_ch(&mut self, c : char) -> Result<(), CallbackError> {
        self.push(c);
        Ok(())
    }
}






/// Additional data accompanying the encoded result: Any additional definitions
/// the preamble should contain to ensure the replacement strings parse
/// correctly, along with the list of encountered unknown characters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EncodeReport {
    /// The unicode-normalized input that the encoder ran on.  If `None`, the
    /// input string is already normalized and didn't need any further
    /// normalization.
    pub normalized_input: Option<String>,
    /// Include these definitions in your document's preamble in order to ensure
    /// that all replacement texts used in the encoded string are appropriately
    /// defined.
    pub needs: PreambleNeeds,
    /// A list of characters that the encoder failed to encode and that
    ///  underwent the unknown char policy.
    pub unknown: Vec<UnknownChar>,
}




/// An error reported by the encoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// The error data reported when the unknown char policy is an error.
    UnknownCharacter {
        /// The unknown character.
        ch: char,
        /// Its byte position in the unicode-normalized input.
        position: usize,
    },
    /// An error happened in a callback.
    CallbackError {
        error: Box<dyn Error + Send + Sync>,
    },
    /// A callable rule reported a consumption that is not a valid advance:
    /// zero bytes, more than remain, or a count ending inside a character.
    InvalidRuleConsumption {
        /// The byte position the rule was tried at.
        position: usize,
        /// The number of bytes it claimed to consume.
        consumed: usize,
    },
}



/// The policy to adopt for unicode characters that the encoder found no
/// replacement for.
#[derive(Clone, Default)]
pub enum UnknownCharPolicy {
    /// Keep the character itself, as UTF-8.
    #[default]
    Keep,
    /// Write a fixed placeholder replacement text for any unknown character.
    Replace(String),
    /// Remove the unknown character without any corresponding output.
    Ignore,
    /// Stop with [`EncodeError::UnknownCharacter`].
    Fail,
    /// A custom handler, providing a custom replacement string for any given
    /// unicode char that the encoder didn't know how to handle.
    Custom(Arc<dyn (Fn(char) -> String) + Send + Sync>),
}


impl UnknownCharPolicy {
    /// Take action for the unknown character `ch` according to the stored
    /// policy.
    fn apply_into<OutBuffer>(&self, out: &mut OutBuffer, ch: char, position: usize)
                             -> Result<(), EncodeError> {
        match self {
            UnknownCharPolicy::Keep => out.push_char(ch),
            UnknownCharPolicy::Replace(placeholder) => out.push_str(placeholder)?,
            UnknownCharPolicy::Ignore => {}
            UnknownCharPolicy::Fail => {
                return Err(EncodeError::UnknownCharacter { ch, position });
            }
            UnknownCharPolicy::Custom(f) => out.push_str(&f(ch))?,
        }
        Ok(())
    }
}

impl fmt::Debug for UnknownCharPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            UnknownCharPolicy::Keep => "Keep",
            UnknownCharPolicy::Replace(_) => "Replace(..)",
            UnknownCharPolicy::Ignore => "Ignore",
            UnknownCharPolicy::Fail => "Fail",
            UnknownCharPolicy::Custom(_) => "Custom(..)",
        })
    }
}








pub struct Encoder<Rule> {
    

    rules

}
