# Build plan - Notes



## Naming conventions

- A rule's output string is called `encoded` (not `latex`).
- What a matching rule returns is an `EncodedReplacement` (not `Replacement`).
- The protection strategy trait is `ReplacementProtection` (not `Protection`).
- Standard strategies: `BraceProtection::{BracesAround, BracesAfter,
  NoBraceProtection}`. Names must say what they do; a name that disables a
  safety net must say so.

## Design decisions

Settled so far (design session, September 2026):

- One dyn-compatible `Rule: Debug` trait (no `Send + Sync` supertraits).
  `apply(&self, input: RuleInput<'_>) -> Result<Option<EncodedReplacement<'a>>, RuleError>`;
  first match wins; all positions and lengths are byte offsets.
- `EncodedReplacement<'a>` holds `consumed`, `encoded: Cow<'a, str>`, a
  protection hint, and preamble needs. Its constructor takes
  `impl Into<Cow<'a, str>>`.
- `RuleInput<'s>`: small `Copy` struct with private fields and accessors.
- Chains: one generic `RuleChain<L>` wrapper (room for chain-level options);
  sealed `RuleList` helper implemented for tuples, arrays, `Vec`, slices.
  Aliases `DynRuleChain` (`+ Send + Sync`) and `LocalDynRuleChain`.
  Closures become rules through `rule_fn(..)`.
- Protection is stateless: each encoded string is protected as it comes (no
  lookahead). Hints travel on the `EncodedReplacement`; no per-rule mode
  overrides. pylatexenc's v1-compatibility modes are dropped.
- Protection hints are authoritative and required: the rule states
  `ValueIsSelfTerminating`, `ValueEndsWithNamedMacro`, or `DoNotProtect`
  (no "inspect for me" default). A public `const fn` inspection helper exists
  for rules that do not know; the table macro runs it at compile time.
- `EncodedReplacement` has private fields and is built only through
  `RuleInput` (`replace_char`, `replace_prefix`, `try_replace_prefix`), so an
  invalid `consumed` cannot be constructed. No `InvalidConsumption` error.
- Unknown chars (no rule matched, and not printable ASCII / `\n\r\t`):
  `UnknownCharPolicy::{Keep, Ignore, Fail, ReplaceWith, Callback}`. The
  callback is a plain `Fn(char) -> String + Send + Sync`; its output gets no
  protection, hints, or profiles. Anything richer is a rule at the end of the
  chain. `unknown_unihex` ships as a plain function.
- Preamble needs: a chunk is one package load or one snippet; a profile is
  the set of chunks one encoded string needs. Static tables store a `u8`
  profile index, but a rule hands out a plain reference to the profile (no
  runtime id spaces or registries). `Chunk`/`Profile` have no lifetime
  parameter (`Cow<'static, ..>` inside).
- `OutBuffer`: own trait, `push_str` required, fixed `BoxError` error type;
  impl for `String`, adapters for `fmt::Write` and (feature `std`)
  `io::Write`.
- `EncodeError::{UnknownChar, Rule, Output}`, `#[non_exhaustive]`.
- Reporting: accumulating primitive `encode_into(text, &mut out, &mut report)`
  plus pure conveniences `encode` / `encode_with_report`. The report owns its
  data (no lifetime): needs + distinct unknown chars (no counts, no
  positions).
