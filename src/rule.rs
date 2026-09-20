
use super::{OutRuleInterface, CallbackError, NeedsProfile, ReplacementProtectionHints};


#[derive(Error)]
pub enum RuleError {
    CallbackError(CallbackError),
}



/// When a rule matches and applies, it reports (1) the number of input bytes
/// that were consumed; (2) the encoded variant of the consumed input; and (3)
/// any additional "needs" associated with this encoded string, for instance,
/// additional preamble package includes/definitions.  See [`NeedsProfile`].
pub struct RuleResultData<OutType> {
    consumed : usize,
    encoded : OutType,
    needs : NeedsProfile,
}

/// What a rule call reports.  The rule may report (i) that it was successfully
/// applied, returning associated success data (`Ok(Some(...))`, see
/// [`RuleResultData`]); (ii) that it did not match (`Ok(None)`); or (iii) that
/// it would normally apply but encountered a critical error that should be
/// reported (`Err(RuleError)`).
pub type RuleResult<OutType> = Result<Option<RuleResultData<OutType>>, RuleError>;


pub trait Rule : Debug + Send + Sync {

    type OutType : AsRef<str>; // &'static str, String, Cow<'_, str>, Box<str>, ...

    pub fn apply(&'a self, s : &str, pos : usize, ch : char) -> RuleResult<OutType>;

    pub fn protection_hints(&self, value : &OutType) -> Option<ReplacementProtectionHints>;
}


type RuleCallbackFn = dyn Fn(&str, usize, char) -> RuleResult<Cow::<'_, str>>;

pub struct RuleCallback {
    callback : RuleCallbackFn;
}

impl Rule for RuleCallback {

    type OutType = Cow::<'_, str>;

    pub fn new(callback : RuleCallbackFn) -> Self {
        RuleCallback { callback }
    }

    pub fn apply(&self, out : s : &str, pos : usize, ch : char) -> RuleResult {
        callback(s, pos, ch)
    }
}

impl Debug for RuleCallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("RuleCallback");
        d.field("callback", "…".into()).finish()
    }
}




pub struct DynRuleChain<OutRuleInterface> {
    rules : Vec<dyn Rule>;
}

impl Rule for DynRuleChain {
    type OutType = Cow::<'_, str>;

    pub fn new(rules : Vec<dyn Rule>) {
        
    }
    
    pub fn apply<'a, OutBuffer>(
        &'a self, out : &mut OutRuleInterface, s : &str, pos : usize, ch : char,
    ) -> RuleResult {
    }
}
