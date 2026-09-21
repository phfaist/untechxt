//! Benchmarks for the encoder and for the three static table layouts.
//!
//! Five corpora stand for the kinds of text the encoder sees: ASCII-heavy
//! source with a few LaTeX specials, accented Latin prose, Greek with
//! mathematics, Cyrillic prose, and CJK prose, which the builtin table does
//! not cover at all and which therefore runs the unknown-character path under
//! [`UnknownCharPolicy::Keep`].
//!
//! Each corpus is encoded with the same 1549 builtin entries compiled in each
//! of the three static layouts — `binary_search`, `two_level_linear` and
//! `two_level_direct_index` — and with `DEFAULTS` itself, which adds the
//! `BuiltinTable` newtype around whichever layout the crate ships. Every run
//! is text mode with `NoReport`, the encoder's defaults. Three more groups
//! measure what the report costs (on the accented corpus, whose entries need
//! nothing in the preamble, and on the Cyrillic one, whose entries all name a
//! profile) and what the input normalizer costs.
//!
//! The encoder is built once per benchmark and reused, which is how an
//! encoder is meant to be used; the timed part is one `encode` call, output
//! string included.
//!
//! Run them with
//!
//! ```text
//! cargo bench                                   # everything
//! cargo bench -- 'layout/accented'              # one corpus
//! cargo bench -- 'layout/.*/binary_search'      # one layout everywhere
//! cargo bench -- --measurement-time 2           # shorter, noisier
//! ```
//!
//! Criterion writes its results under `target/criterion/`. This build has no
//! HTML reports: they would pull in `plotters` and `rayon` for nothing.

use std::hint::black_box;

use criterion::measurement::WallTime;
use criterion::{criterion_main, BenchmarkGroup, Criterion, Throughput};

use untechxt::builtin::default_table::ENTRIES;
use untechxt::builtin::needs_profiles::PROFILES;
use untechxt::{
    compile_static_table, Encoder, NoNormalization, Rule, StaticTableBinarySearch,
    StaticTableTwoLevelDirect, StaticTableTwoLevelLinear, UnknownCharPolicy, DEFAULTS,
};

// The same data as `DEFAULTS`, compiled in each layout. `ENTRIES` is
// `#[doc(hidden)] pub` for exactly this.

static BINARY_SEARCH: StaticTableBinarySearch =
    compile_static_table!(ENTRIES, &PROFILES, binary_search);

static TWO_LEVEL_LINEAR: StaticTableTwoLevelLinear =
    compile_static_table!(ENTRIES, &PROFILES, two_level_linear);

static TWO_LEVEL_DIRECT: StaticTableTwoLevelDirect =
    compile_static_table!(ENTRIES, &PROFILES, two_level_direct_index);

// ------------------------------------------------------------------ corpora

/// ASCII-heavy source-like text: a build recipe and a bit of prose, with the
/// LaTeX specials `# $ % & \ { } _ ^ < > ~ "` scattered through it. Almost
/// every byte of it is copied in bulk by the ASCII fast path.
const ASCII_SOURCE: &str = r##"
# Makefile fragment for the paper. `make paper` builds paper.pdf out of the
# sources under src/ and the figures under fig/; run it with VERBOSE=1 to see
# every command as it goes by.

LATEX  ?= lualatex
FLAGS  := -interaction=nonstopmode -halt-on-error
FIGS   := $(wildcard fig/*.pdf)
SHARE  := 65%

paper.pdf: paper.tex $(FIGS) refs.bib
	$(LATEX) $(FLAGS) paper.tex && biber paper && $(LATEX) $(FLAGS) paper.tex

clean:
	rm -f *.aux *.log *.out *.bbl *.bcf paper.pdf

# The encoder turns text into LaTeX source, so a benchmark corpus that is
# mostly ASCII is the common case: an abstract, a table caption, a bibliography
# entry. The few characters that LaTeX reads specially -- the comment sign, the
# math shift, the two alignment characters, the braces, the subscript and
# superscript marks, the tilde and the backslash -- all have to be escaped, and
# they appear in file names and in code listings all the time: see src/a_b.rs,
# the format string "%s <%d>", the shell glob fig/*~, and the C++ template
# vector<pair<int, int>> that the listing on page 7 prints verbatim.

if [ "$STATUS" != "ok" ]; then echo "build failed: $STATUS" >&2; exit 1; fi
printf '%s\t%s\n' "$name" "$value" | awk '{ print $1 "_" $2 }' > out.tsv
"##;

/// Accented Latin prose: French, Spanish, Portuguese and German, in NFC, so
/// that `NormalizeNfc` borrows the input instead of rebuilding it. Roughly one
/// character in ten is a table lookup.
const ACCENTED: &str = "\
Le café où Émile prenait son thé à cinq heures était déjà fermé quand nous \
sommes arrivés, et la pluie n'avait pas cessé depuis le matin. « Tant pis », \
dit-il en relevant son col, « nous irons à la Brasserie de l'Hôtel de Ville, \
c'est à deux pas. » La façade, décorée de céramiques bleues, portait encore \
l'enseigne d'un marchand de journaux disparu en mil neuf cent quarante.\n\
El señor Muñoz, que había pasado la mañana revisando los álbumes de su \
bisabuelo, encontró una fotografía tomada en Cádiz: la niña del vestido \
almidonado miraba al fotógrafo con más curiosidad que paciencia, y detrás de \
ella un carromato cargado de melocotones ocupaba media calle.\n\
Em Lisboa, à beira do rio, o avô contava que os navios traziam açúcar, canela \
e notícias do outro lado do oceano; hoje trazem contentores, e ninguém sai de \
casa para os ver chegar.\n\
Fräulein Jäger hatte die Übersetzung schon dreimal überarbeitet, doch der \
Satz über die Größe des Weltalls wollte auf Deutsch nicht klingen: zu lang, zu \
umständlich, zu höflich. Sie schloß das Heft, öffnete das Fenster und ließ die \
kühle Luft herein.\n";

/// Greek prose with the mathematics that goes with it: most of these
/// characters are table entries whose spelling is math-only, so every one of
/// them is wrapped in `\\ensuremath{…}` on the way out.
const GREEK_MATH: &str = "\
Ἡ ἀπόδειξη ξεκινᾶ ἀπὸ τὴν ἀνισότητα ‖x‖ ≤ ‖y‖ + ε, ὅπου ε > 0 εἶναι αὐθαίρετα \
μικρό. Ἔστω α, β ∈ ℝ μὲ α < β· τότε γιὰ κάθε συνεχῆ συνάρτηση f : [α, β] → ℝ \
ὑπάρχει ξ ∈ (α, β) ὥστε ∫ f = f(ξ)·(β − α).\n\
Θεώρημα. Ἂν Σ ⊆ ℕ καὶ Σ ≠ ∅, τότε ὑπάρχει ἐλάχιστο στοιχεῖο· ἡ ἀπόδειξη \
γίνεται μὲ ἐπαγωγή. Παρατηρῆστε ὅτι λ · μ ≠ μ · λ ὅταν οἱ πίνακες δὲν \
μετατίθενται, ἐνῶ ∀ διανύσματα u, v ἰσχύει ⟨u, v⟩ = ⟨v, u⟩ στὸν ℝⁿ.\n\
Ἀπὸ τὸ ἀνάπτυγμα Taylor: f(x) = Σ aₙ xⁿ, μὲ ἀκτίνα σύγκλισης ρ = ∞ ὅταν ἡ \
συνάρτηση εἶναι ἀκέραια· διαφορετικά ρ < ∞ καὶ τὸ ὅριο lim |aₙ|^(1/n) → 1/ρ. \
Ὁ τελεστής ∇ × (∇ φ) = 0 γιὰ κάθε βαθμωτὸ πεδίο φ, καὶ ∂²u/∂t² = c² ∇²u \
περιγράφει τὸ κῦμα. Ἐπίσης: π ≈ 3.14159, γ ≈ 0.5772, ℵ₀ < 2^ℵ₀, A ⊗ B ≅ B ⊗ A, \
καὶ √2 ∉ ℚ.\n";

/// Cyrillic prose. Every letter is a table entry with a `T2A` font-encoding
/// spelling and a preamble profile, so this corpus exercises the needs path
/// as well (it does nothing under `NoReport`, which is the point).
const CYRILLIC: &str = "\
Весь день шёл снег, и к вечеру двор за окном стал неузнаваем: забор, поленница, \
старая яблоня — всё оказалось под одним белым покрывалом, и только синица, \
прыгавшая по подоконнику, напоминала, что зима ещё не всё укрыла.\n\
— Ты опять не закрыл форточку, — сказала бабушка, не оборачиваясь от плиты. — \
В комнате холодно, как в сенях.\n\
Он молча притворил окно и сел за стол. На клеёнке лежала раскрытая тетрадь, в \
которой вчера он начал переписывать стихотворение, да так и бросил на середине \
строки. Буквы выходили кривыми, перо цеплялось за бумагу, а за спиной шумел \
чайник, и было слышно, как в печи потрескивают дрова.\n\
Через полчаса пришёл сосед, принёс газету и долго рассказывал о том, что \
дорогу за мостом опять замело, что автобус не ходит и что придётся идти пешком \
до самой станции. Бабушка налила ему чаю, и разговор перешёл на цены, на \
погоду и на то, каким было это же самое место тридцать лет назад.\n";

/// Chinese and Japanese prose. The builtin table has no ideograph and no
/// kana, so under `UnknownCharPolicy::Keep` every character of this corpus
/// goes through the unknown-character path and is copied as it is.
const CJK: &str = "\
春天的早晨，城南的老巷子里飘着豆浆和油条的香味。卖早点的摊子摆在墙根底下，\
蒸笼冒着白气，排队的人一边跺脚一边说着昨天的球赛。巷口那棵老槐树又发芽了，\
树底下坐着几位下棋的老人，棋盘是水泥台面上画的，棋子敲下去咔咔作响。\n\
他骑着自行车穿过巷子，车筐里放着刚买的报纸和半斤水果。拐过第二个路口就是学校，\
铁门还没开，几个学生靠在墙上背单词。\n\
その日の午後、図書館の窓際の席はいつもより静かだった。雨が降り出したせいで、\
中庭を歩く人の姿もない。彼女は借りたばかりの本を開き、最初の一行を読んでから、\
しばらく窓の外を眺めていた。\n\
帰り道、駅前の商店街ではもう明かりがともり、揚げ物のにおいが漂っていた。\
傘を差した人たちが足早に通り過ぎていく。明日も同じ時間にここを通るのだろう、\
と思いながら、彼女は改札を抜けた。\n";

/// The five corpora, with the names the benchmark ids carry.
const CORPORA: [(&str, &str); 5] = [
    ("ascii_source", ASCII_SOURCE),
    ("accented", ACCENTED),
    ("greek_math", GREEK_MATH),
    ("cyrillic", CYRILLIC),
    ("cjk", CJK),
];

// --------------------------------------------------------------- benchmarks

/// One encoder over `rule`, built once, then one timed `encode` per
/// iteration.
///
/// The rule is taken by reference, as all four of the tables here are used:
/// a table is a static, and `&R` forwards `Rule`.
fn bench_rule<R: Rule>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &str,
    rule: &'static R,
    text: &'static str,
) {
    let encoder = Encoder::new(rule).with_unknown_chars(UnknownCharPolicy::Keep);
    // Fail here rather than inside the measurement loop.
    assert!(!encoder.encode(text).unwrap().is_empty());
    group.bench_function(name, |b| {
        b.iter(|| black_box(encoder.encode(black_box(text)).unwrap()));
    });
}

/// The layout comparison: every corpus against every layout.
fn layouts(c: &mut Criterion) {
    for (corpus, text) in CORPORA {
        let mut group = c.benchmark_group(format!("layout/{corpus}"));
        group.throughput(Throughput::Bytes(text.len() as u64));
        bench_rule(&mut group, "binary_search", &BINARY_SEARCH, text);
        bench_rule(&mut group, "two_level_linear", &TWO_LEVEL_LINEAR, text);
        bench_rule(&mut group, "two_level_direct_index", &TWO_LEVEL_DIRECT, text);
        bench_rule(&mut group, "defaults", &DEFAULTS, text);
        group.finish();
    }
}

/// What the report costs: `NoReport`, which is what `encode` uses, against
/// `EncodeReport`.
///
/// On the accented corpus every entry needs nothing in the preamble, so this
/// is the cost of carrying the reporter at all; the Cyrillic corpus, whose
/// entries all name a `fontenc` profile, is the one that actually records
/// something.
fn reporting(c: &mut Criterion) {
    let encoder = Encoder::new(&DEFAULTS);
    for (corpus, text) in [("accented", ACCENTED), ("cyrillic", CYRILLIC)] {
        let mut group = c.benchmark_group(format!("report/{corpus}"));
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_function("no_report", |b| {
            b.iter(|| black_box(encoder.encode(black_box(text)).unwrap()));
        });
        group.bench_function("encode_report", |b| {
            b.iter(|| black_box(encoder.encode_with_report(black_box(text)).unwrap()));
        });
        group.finish();
    }
}

/// What the input normalizer costs: `NormalizeNfc` on text that already is
/// NFC (the quick check alone, no rebuilding) against `NoNormalization`.
fn normalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("normalization/accented");
    group.throughput(Throughput::Bytes(ACCENTED.len() as u64));
    let nfc = Encoder::new(&DEFAULTS);
    let raw = Encoder::new(&DEFAULTS).with_normalizer(NoNormalization);
    group.bench_function("normalize_nfc", |b| {
        b.iter(|| black_box(nfc.encode(black_box(ACCENTED)).unwrap()));
    });
    group.bench_function("no_normalization", |b| {
        b.iter(|| black_box(raw.encode(black_box(ACCENTED)).unwrap()));
    });
    group.finish();
}

// `criterion_group!` declares a public function of its own, which the
// crate's `missing_docs = "deny"` would reject.
#[allow(missing_docs)]
mod harness {
    criterion::criterion_group!(
        benches,
        super::layouts,
        super::reporting,
        super::normalization
    );
}

criterion_main!(harness::benches);
