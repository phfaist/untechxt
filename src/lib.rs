//! Unicode-to-LaTeX encoding - main library module


#[no_std]


mod rule;
mod encoder;
mod replacement_protection;
mod preamble;


pub mod profile;
pub mod lookuptable;
pub mod statictable;

pub use preamble::ChunkPreamble;
pub use encoder::{
    OutBuffer, Encoder, EncodeReport, EncodeError, UnknownCharPolicy,
    OutRuleInterface,
};
pub use rule::{Rule, RuleResult, RuleError, RuleCallback, RuleCallbackFn}





pub type CallbackError = Box<dyn Error + Send + Sync>;




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReplacementProtectionHints {
    /// The generated latex encoded value, under a standard document parsing
    /// state/catcode setting, will always parse correctly, regardless of
    /// whatever string succeeds this replaced value.
    ValueIsSelfTerminating,
    /// The generated latex encoded value ends with a macro name (e.g.,
    /// `r"\hat\i"`).  If immediately follows by more content, the macro will
    /// fail to parse correctly (e.g., `r"\hat\i" + "more text" = r"\hat\imore
    /// text", incorrect).  The value needs to be protected using a suitable
    /// protection strategy, e.g., enclosing the value in braces (r"{\hat\i}").
    ValueEndsWithNamedMacro,

    // We might add further variants in the future...
    //
    // /// The value will cause any text that follows to be interpreted as part of
    // /// a comment.  Needs a newline to restore correct content parsing.
    // ValueEndsInAComment,
}




















    /// Try the rule at byte `pos` of `s`, where `ch` is the character there:
    /// the bytes consumed and the spelling, or `None` when it does not apply.
    fn apply<'a>(
        &'a self,
        s: &str,
        pos: usize,
        ch: char,
    ) -> Result<Option<(usize, Cow<'a, str>, Profile)>, EncodeError> {
        match &self.kind {
            RuleKind::Table(table) => Ok(table
                .binary_search_by_key(&ch, |&(entry, _, _)| entry)
                .ok()
                .map(|index| (ch.len_utf8(), Cow::Borrowed(table[index].1), table[index].2))),
            RuleKind::Dict(map) => Ok(map
                .get(&ch)
                .map(|latex| (ch.len_utf8(), Cow::Borrowed(latex.as_str()), Profile::BUILTINS))),
            RuleKind::Callable(f) => match f(s, pos) {
                None => Ok(None),
                Some((consumed, latex)) => {
                    let end = pos.checked_add(consumed);
                    let valid = consumed > 0
                        && end.is_some_and(|end| end <= s.len() && s.is_char_boundary(end));
                    if !valid {
                        return Err(EncodeError::InvalidConsumption { position: pos, consumed });
                    }
                    Ok(Some((consumed, Cow::Owned(latex), Profile::BUILTINS)))
                }
            },
        }
    }

































    /// Try the rule at byte `pos` of `s`, where `ch` is the character there:
    /// the bytes consumed and the spelling, or `None` when it does not apply.
    fn apply<'a>(
        &'a self,
        s: &str,
        pos: usize,
        ch: char,
    ) -> Result<Option<(usize, Cow<'a, str>, Profile)>, EncodeError> {
        match &self.kind {
            RuleKind::Table(table) => Ok(table
                .binary_search_by_key(&ch, |&(entry, _, _)| entry)
                .ok()
                .map(|index| (ch.len_utf8(), Cow::Borrowed(table[index].1), table[index].2))),
            RuleKind::Dict(map) => Ok(map
                .get(&ch)
                .map(|latex| (ch.len_utf8(), Cow::Borrowed(latex.as_str()), Profile::BUILTINS))),
            RuleKind::Callable(f) => match f(s, pos) {
                None => Ok(None),
                Some((consumed, latex)) => {
                    let end = pos.checked_add(consumed);
                    let valid = consumed > 0
                        && end.is_some_and(|end| end <= s.len() && s.is_char_boundary(end));
                    if !valid {
                        return Err(EncodeError::InvalidConsumption { position: pos, consumed });
                    }
                    Ok(Some((consumed, Cow::Owned(latex), Profile::BUILTINS)))
                }
            },
        }
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Rule");
        match &self.kind {
            RuleKind::Table(table) => d.field("table", &table.len()),
            RuleKind::Dict(map) => d.field("dict", &map.len()),
            RuleKind::Callable(_) => d.field("callable", &".."),
        };
        d.field("protection", &self.protection).finish()
    }
}
