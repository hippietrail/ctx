//! # Context Pattern Analyzer
//!
//! A tool for analyzing collocation and context patterns to help distinguish
//! between similar or confusable words. This is primarily useful for creating
//! grammar checker rules and linters.
//!
//! ## Purpose
//!
//! The tool analyzes word alternatives using Google Ngram Viewer data to identify
//! unique contextual usage patterns. For example:
//!
//! - **Spelling variants**: "tomato" vs "tomatoe"
//! - **Contractions vs full forms**: "can't" vs "cant"
//! - **Pluralization patterns**: "pastas" vs "spaghettis"
//! - **Part-of-speech ambiguity**: "dose" (verb/noun) vs "does" (verb only)
//!
//! By identifying unique contexts (words that appear with one alternative but
//! not others) and prohibited contexts (words that appear with all other
//! alternatives but not the current one), you can create rules to detect
//! when a word might be a mistake.
//!
//! ## Features
//!
//! - Fetches context words from Google Ngram Viewer (before/after each alternative)
//! - Uses part-of-speech tagging to categorize context words
//! - Identifies unique contexts (🟢) - words that appear with only one alternative
//! - Identifies prohibited contexts (🚫) - words that appear with all other alternatives
//! - Supports family grouping for comparing related alternatives
//! - Raw diagnostic output with POS tags for debugging
//!
//! ## Family Grouping
//!
//! Families allow comparing groups of related alternatives:
//! - Case variants: lowercase vs titlecase of the same word
//! - Compound forms: spaced compounds vs hyphenated compounds
//! - Any other grouping where you want to compare within the group but not across groups

// ============================================================================
// MODULES AND IMPORTS
// ============================================================================

// Local modules
mod google_ngram_viewer;
// Part-of-speech
mod pos;

// Standard library
use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Display,
    vec::Vec,
};

// External crates
use harper_core::spell::{Dictionary, FstDictionary};
use itertools::Itertools;
use owo_colors::{FgDynColorDisplay, OwoColorize, Rgb};

// Local modules
use google_ngram_viewer::{
    Row, Side,
    Side::{After, Before},
    build_url, fetch_json, parse_items,
};
use pos::Pos;

// ============================================================================
// COLOR UTILITIES
// ============================================================================

/// Shorthand trait for truecolor formatting
///
/// Provides a shorter `.tc()` method as an alias for `.truecolor()` from owo_colors.
/// This reduces verbosity when specifying RGB colors for terminal output.
///
/// Example:
/// ```rust
/// "text".tc(255, 0, 0)  // Red text
/// "text".tc(0, 255, 0)  // Green text
/// ```
pub trait ShortColor {
    fn tc(&self, r: u8, g: u8, b: u8) -> FgDynColorDisplay<'_, Rgb, Self>;
}

// Implement it for all types that already implement OwoColorize
impl<T: OwoColorize> ShortColor for T {
    #[inline(always)]
    fn tc(&self, r: u8, g: u8, b: u8) -> FgDynColorDisplay<'_, Rgb, Self> {
        self.truecolor(r, g, b)
    }
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Represents a word alternative being analyzed
/// - `raw`: The original input string from the user
/// - `jfmt`: Formatted how the Ngram Viewer JSON uses (with hyphens replaced by " - ")
/// - `fam`: Optional family grouping for related alternatives
#[derive(Debug, Clone)]
pub struct Alternative {
    pub raw: String,
    pub jfmt: String,
    pub fam: Option<String>,
}

/// Configuration parsed from command-line arguments
/// - `raw`: If true, print raw diagnostic output
/// - `alternatives`: List of word alternatives to analyze
/// - `has_families`: Whether family groupings are enabled
pub struct Cfg {
    pub debug: bool,
    pub raw: bool,
    pub alternatives: Vec<Alternative>,
    pub has_families: bool,
    pub since_year: Option<[char; 4]>,
}

/// How well a context word matches across alternatives
/// - `Exact`: Identical string match (shared context, not unique)
/// - `Normalized`: Case-insensitive match (unique only with case sensitivity)
/// - `NoMatch`: No match across alternatives (truly unique context)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ContextMatch {
    Exact,
    Normalized,
    NoMatch,
}
use ContextMatch::*;

/// A collocation: a specific alternative with its side and optional family
///
/// Represents the combination of an alternative word, which side (Before/After),
/// and optionally a family grouping. Used as a key in the results HashMap to
/// organize unique context words by their collocation context.
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
struct Collocation<'a> {
    fam: Option<&'a str>,
    alt: &'a str,
    side: Side,
}

impl Display for Collocation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.fam {
            Some(fam) => write!(f, "{}({:?})[{}]", self.alt, self.side, fam),
            None => write!(f, "{}({:?})", self.alt, self.side),
        }
    }
}

/// A context word with its case-sensitivity flag
///
/// - `word`: The context word that appears near an alternative
/// - `case_sensitive`: If true, this word is unique only when considering case
///   sensitivity (e.g., "The" vs "the"). The DAGGER (†) symbol marks these in output.
struct ContextWord<'a> {
    word: &'a str,
    case_sensitive: bool,
}

// ============================================================================
// COMMAND-LINE INTERFACE
// ============================================================================

/// Parses command-line arguments into a configuration struct
///
/// Supported arguments:
/// - `--raw`: Enable raw diagnostic output
/// - `--family=<name>`, `--fam=<name>`, `-f=<name>`: Set family for subsequent alternatives
///
/// TODO: Add --help flag to show usage
/// TODO: Validate that at least one alternative is provided
pub fn cli() -> Result<Cfg, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut cfg = Cfg {
        debug: false,
        raw: false,
        alternatives: Vec::new(),
        has_families: false,
        since_year: None,
    };

    let mut family = None;

    for arg in args {
        if arg == "--raw" {
            cfg.raw = true;
        } else if ["--debug", "-d"].contains(&arg.as_str()) {
            cfg.debug = true;
        } else if ["--family=", "--fam=", "-f="]
            .iter()
            .any(|p| arg.starts_with(p))
        {
            family = match arg.split('=').nth(1) {
                None | Some("") => None,
                Some(s) => Some(s.to_string()),
            };
            cfg.has_families = true;
        } else if let Some(prefix) = ["--since=", "--since-year="]
            .into_iter()
            .find(|p| arg.starts_with(p))
        {
            cfg.since_year = match &arg[prefix.len()..] {
                "" => None,
                y if y.len() == 4 && y.chars().all(|c| c.is_ascii_digit()) => {
                    let y = y.as_bytes();
                    Some([y[0] as char, y[1] as char, y[2] as char, y[3] as char])
                }
                _ => return Err(format!("Year must be exactly 4 digits").into()),
            };
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: {}", arg).into());
        } else if arg.contains('-') {
            // Hyphenated phrases: replace hyphens with " - " for Ngram JSON API compatibility
            // NOTE: See also the special handling for apostrophes in google_ngram_viewer.rs/build_url()
            cfg.alternatives.push(Alternative {
                raw: arg.clone(),
                // jfmt: arg.split('-').intersperse(" - ").collect(),
                jfmt: arg.split('-').join(" - "),
                fam: family.clone(),
            });
        } else if arg.contains('\'') {
            // Special handling when any word starts or ends with an apostrophe, such as 'tis or 'nother
            // NOTE: See also the special handling for apostrophes in google_ngram_viewer.rs/build_url()
            let jfmt = arg
                .split(' ')
                .map(|part| {
                    if let Some(w) = part.strip_prefix('\'') {
                        format!("' {w}")
                    } else if let Some(w) = part.strip_suffix('\'') {
                        format!("{w} '")
                    } else {
                        part.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");
            cfg.alternatives.push(Alternative {
                raw: jfmt.clone(),
                jfmt,
                fam: family.clone(),
            });
        } else {
            // Simple words: use as-is
            cfg.alternatives.push(Alternative {
                raw: arg.clone(),
                jfmt: arg.clone(),
                fam: family.clone(),
            });
        }
    }

    Ok(cfg)
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Checks if a row belongs to the same family as the target alternative
/// Used for filtering when family grouping is enabled
fn same_family(cfg: &Cfg, row_alt: &str, target_fam: Option<&str>) -> bool {
    !cfg.has_families
        || cfg
            .alternatives
            .iter()
            .find(|a| a.jfmt == row_alt)
            .map(|a| a.fam.as_deref() == target_fam)
            .unwrap_or(false)
}

/// Gets the formatted (jfmt) version of an alternative by its raw name
fn get_jfmt<'a>(cfg: &'a Cfg, alt: &str) -> &'a str {
    cfg.alternatives
        .iter()
        .find(|a| a.raw == alt)
        .map(|a| a.jfmt.as_str())
        .unwrap()
}

/// Formats context words with alternating colors and case-sensitive marker
fn format_context_words(words: &[ContextWord]) -> String {
    words
        .iter()
        .sorted_by_key(|cw| cw.word.to_lowercase())
        .enumerate()
        .map(|(i, cw)| {
            format!(
                "\x1b[{}m{}\x1b[0m{}",
                31 + (i % 2),
                cw.word,
                if cw.case_sensitive { "†" } else { "" }
            )
        })
        .join(" ")
}

// ============================================================================
// PART-OF-SPEECH LOOKUP
// ============================================================================

/// Gets the part-of-speech tags for a given word
///
/// Uses the harper_core dictionary to look up word metadata
/// and returns matching POS tags based on predefined predicates.
///
/// TODO: Consider caching results for repeated lookups
/// TODO: Add support for unknown words (OOV - out of vocabulary)
fn get_poses(dict: &FstDictionary, word: &str) -> Vec<&'static Pos> {
    dict.get_word_metadata_str(word)
        .map_or_else(Vec::new, |md| {
            pos::POS_DEFINITIONS
                .iter()
                .filter(|&(_, pred)| pred(&md))
                .map(|(enum_variant, _)| enum_variant)
                .collect()
        })
}

// ============================================================================
// CONTEXT EVALUATION
// ============================================================================

/// Evaluates context words across alternatives to find unique contexts
///
/// For each alternative, compares its context words with other alternatives
/// to determine uniqueness:
/// - NoMatch: Context appears only with this alternative (truly unique)
/// - Normalized: Context appears with different case (unique with case sensitivity)
/// - Exact: Context appears identically across alternatives (not unique, discarded)
///
/// When families are enabled, comparisons are limited to alternatives within the same family.
///
/// TODO: This is O(n²) complexity - could be optimized with better data structures
/// TODO: Consider adding a threshold for minimum frequency to filter noise
fn evaluate_contexts<'a>(
    cfg: &'a Cfg,
    table: &[Row<'a>],
) -> HashMap<Collocation<'a>, Vec<ContextWord<'a>>> {
    let mut results: HashMap<Collocation<'_>, Vec<ContextWord<'_>>> = HashMap::new();

    for alternative in &cfg.alternatives {
        for row in table.iter().filter(|r| r.alt == alternative.jfmt) {
            // Check matches across other alternatives
            let matching_rows: Vec<&Row> = table
                .iter()
                .filter(|r| {
                    r.side == row.side
                        && r.alt != alternative.jfmt
                        && same_family(cfg, r.alt, alternative.fam.as_deref())
                })
                .collect();

            let mut match_kind = NoMatch;
            for other in matching_rows {
                if other.ctx == row.ctx {
                    match_kind = Exact;
                    break;
                } else if other.ctx.to_lowercase() == row.ctx.to_lowercase() {
                    match_kind = Normalized;
                }
            }

            let coll = Collocation {
                alt: &alternative.raw,
                side: row.side,
                fam: alternative.fam.as_deref(),
            };

            match match_kind {
                NoMatch => results.entry(coll).or_default().push(ContextWord {
                    word: row.ctx,
                    case_sensitive: false,
                }),
                Normalized => results.entry(coll).or_default().push(ContextWord {
                    word: row.ctx,
                    case_sensitive: true,
                }),
                Exact => {} // Discarded from standard uniqueness lists
            }
        }
    }

    results
}

// ============================================================================
// OUTPUT FORMATTING
// ============================================================================

/// Prints raw diagnostic output for each alternative
///
/// Shows all context words (before and after) with their POS tags.
/// Useful for debugging and understanding the raw data from the API.
///
/// TODO: Consider making this output more structured (e.g., JSON)
/// TODO: Add frequency information from the timeseries data
fn print_raw_diagnostics(cfg: &Cfg, table: &[Row], dict: &FstDictionary) {
    for (i, var) in cfg
        .alternatives
        .iter()
        .sorted_by_key(|a| a.fam.as_ref().map(|s| s.to_ascii_lowercase()))
        .enumerate()
    {
        println!(
            "{}•{}{}",
            if i == 0 { "" } else { "\n" },
            if var.fam.is_some() {
                format!("{}:", var.fam.as_ref().unwrap())
            } else {
                String::new()
            },
            var.raw,
        );

        let before_words: Vec<_> = table
            .iter()
            .filter(|r| r.alt == var.jfmt && r.side == Before)
            .sorted_by_key(|r| r.ctx.to_ascii_lowercase())
            .map(|r| r.ctx)
            .unique()
            .collect();
        let after_words: Vec<_> = table
            .iter()
            .filter(|r| r.alt == var.jfmt && r.side == After)
            .sorted_by_key(|r| r.ctx.to_ascii_lowercase())
            .map(|r| r.ctx)
            .unique()
            .collect();

        let before_poses: Vec<&'static Pos> = before_words
            .iter()
            .flat_map(|w| get_poses(dict, w))
            .sorted_by_key(|pos| pos::pos_info(pos).ord)
            .unique()
            .collect();
        println!(
            "«p {}",
            before_poses
                .iter()
                .enumerate()
                .map(|(i, pos)| pos::pos_info(pos)
                    ._emoji
                    .tc(100, 200 + ((i as u8) & 1) * 50, 100)
                    .to_string())
                .join("")
        );
        println!(
            "«w {}",
            before_words
                .iter()
                .enumerate()
                .map(|(i, w)| format!("{}", w.tc(100, 100, 200 + ((i as u8) & 1) * 50)))
                .join(" ")
        );
        println!(
            "»w {}",
            after_words
                .iter()
                .enumerate()
                .map(|(i, w)| format!("{}", w.tc(100, 100, 200 + ((i as u8) & 1) * 50)))
                .join(" ")
        );

        let after_poses: Vec<&'static Pos> = after_words
            .iter()
            .flat_map(|w| get_poses(dict, w))
            .sorted_by_key(|pos| pos::pos_info(pos).ord)
            .unique()
            .collect();
        println!(
            "»p {}",
            after_poses
                .iter()
                .enumerate()
                .map(|(i, pos)| pos::pos_info(pos)
                    ._emoji
                    .tc(100, 150 + ((i as u8) & 1) * 100, 100)
                    .to_string())
                .join("")
        );
    }
}

fn print_uniq_to(
    fam: Option<&str>,
    formatted: &[(Side, String)],
    dict: &FstDictionary,
    alt: &str,
    cfg: &Cfg,
    table: &[Row],
) {
    let fam_display = fam.map(|f| format!("{}::", f)).unwrap_or_default();

    let target_jfmt = get_jfmt(cfg, alt);

    // Get all POS from ALL context words of current alternative for each side
    let current_poses_by_side = |side: Side| -> HashSet<&'static Pos> {
        table
            .iter()
            .filter(|r| r.alt == target_jfmt && r.side == side)
            .flat_map(|r| get_poses(dict, r.ctx))
            .collect()
    };

    // Get all POS from ALL context words of other alternatives for each side
    let other_poses_by_side = |side: Side| -> HashSet<&'static Pos> {
        table
            .iter()
            .filter(|r| r.alt != target_jfmt && r.side == side && same_family(cfg, r.alt, fam))
            .flat_map(|r| get_poses(dict, r.ctx))
            .collect()
    };

    let current_before_poses = current_poses_by_side(Before);
    let current_after_poses = current_poses_by_side(After);
    let other_before_poses = other_poses_by_side(Before);
    let other_after_poses = other_poses_by_side(After);

    let side_pos = |side: Side| {
        let (current_poses, other_poses) = if side == Before {
            (&current_before_poses, &other_before_poses)
        } else {
            (&current_after_poses, &other_after_poses)
        };

        current_poses
            .iter()
            .filter(|pos| !other_poses.contains(*pos))
            .sorted_by_key(|pos| pos::pos_info(pos).ord)
            .enumerate()
            .map(|(i, pos)| format!("\x1b[{}m{}\x1b[0m", 33 + i % 2, pos::pos_info(pos).letter))
            .join("")
    };

    let side_w = |side: Side| {
        formatted
            .iter()
            .find(|(s, _)| *s == side)
            .map(|(_, s)| s.as_str())
            .unwrap_or("")
    };

    let (pre_str, post_str) = (side_w(Before), side_w(After));

    println!(
        "🟢 \x1b[35m{}\x1b[0m ¦ \x1b[36m{pre_str}\x1b[0m \
        {fam_display}«\x1b[1m{alt}\x1b[0m» \
        \x1b[34m{post_str}\x1b[0m ¦ \x1b[32m{}\x1b[0m",
        side_pos(Before),
        side_pos(After)
    );
}

/// Prints "prohibited" context words - words that appear with ALL other alternatives
/// but NOT with the current alternative.
///
/// This helps identify contexts that strongly discriminate against a particular
/// alternative. For example, if "house" appears with "steak" but never with "stake",
/// then "house" is a prohibited context for "stake".
///
/// TODO: Consider adding a confidence score based on frequency
fn print_prohibited(cfg: &Cfg, table: &[Row], dict: &FstDictionary, alt: &str, fam: Option<&str>) {
    if cfg.alternatives.len() <= 2 {
        return;
    }
    let mut prohib_pre_words = Vec::new();
    let mut prohib_post_words = Vec::new();

    let target_jfmt = get_jfmt(cfg, alt);
    let other_count = if cfg.has_families {
        cfg.alternatives
            .iter()
            .filter(|a| a.fam.as_deref() == fam && a.jfmt != target_jfmt)
            .count()
    } else {
        cfg.alternatives.len() - 1
    };

    if other_count == 0 {
        return;
    }

    let context_counts = table
        .iter()
        .filter(|r| r.alt != target_jfmt && same_family(cfg, r.alt, fam))
        .fold(HashMap::new(), |mut acc, r| {
            *acc.entry((r.side, r.ctx)).or_insert(0) += 1;
            acc
        });

    for ((side, ctx), count) in context_counts {
        if count == other_count
            && !table
                .iter()
                .any(|r| r.alt == target_jfmt && r.side == side && r.ctx == ctx)
        {
            match side {
                Before => prohib_pre_words.push(ctx),
                After => prohib_post_words.push(ctx),
            }
        }
    }

    if !prohib_pre_words.is_empty() || !prohib_post_words.is_empty() {
        prohib_pre_words.sort_by_key(|w| w.to_lowercase());
        prohib_post_words.sort_by_key(|w| w.to_lowercase());

        // Calculate prohibited POS: POS that appear in ALL other alternatives but NOT in current
        let pos_counts = table
            .iter()
            .filter(|r| r.alt != target_jfmt && same_family(cfg, r.alt, fam))
            .fold(HashMap::new(), |mut acc, r| {
                for pos in get_poses(dict, r.ctx) {
                    *acc.entry((r.side, pos)).or_insert(0) += 1;
                }
                acc
            });

        let n_pos = |side: Side| {
            pos_counts
                .iter()
                .filter(|((s, _), count)| *s == side && **count == other_count)
                .map(|((_, pos), _)| *pos)
                .filter(|pos| {
                    // Filter out POS that appear in current alternative
                    !table
                        .iter()
                        .filter(|r| r.alt == target_jfmt && r.side == side)
                        .any(|r| get_poses(dict, r.ctx).contains(pos))
                })
                .unique()
                .sorted_by_key(|p| pos::pos_info(p).ord)
                .enumerate()
                .map(|(i, p)| format!("\x1b[{}m{}\x1b[0m", 33 + i % 2, pos::pos_info(p).letter))
                .join("")
        };

        let n_pre_pos = n_pos(Before);
        let n_post_pos = n_pos(After);

        println!(
            "🚫 \x1b[31m{}\x1b[0m | {} ¦ «{}» ¦ {} | \x1b[31m{}\x1b[0m",
            n_pre_pos,
            prohib_pre_words
                .iter()
                .enumerate()
                .map(|(i, w)| format!("\x1b[3{}m{}\x1b[0m", i % 2 + 4, w))
                .join(" "),
            alt,
            prohib_post_words
                .iter()
                .enumerate()
                .map(|(i, w)| format!("\x1b[3{}m{}\x1b[0m", i % 2 + 4, w))
                .join(" "),
            n_post_pos
        );
    }
}

/// Prints family-level uniqueness view
///
/// Shows context words and POS tags that are unique to each family when comparing
/// across all families. This helps identify what makes each family distinctive.
///
/// The DAGGER (†) symbol indicates words that are unique only when
/// considering case sensitivity.
///
/// TODO: This function has deeply nested loops that could be refactored
/// TODO: The uniqueness check logic is complex and could be extracted
/// TODO: Consider adding a summary view showing the most distinctive words
fn print_family_uniqueness(
    results: &HashMap<Collocation, Vec<ContextWord>>,
    dict: &FstDictionary,
    table: &[Row],
) {
    let mut family_contexts: HashMap<Option<&str>, HashMap<Side, HashMap<&str, ContextMatch>>> =
        HashMap::new();

    for (collocation, context_words) in results {
        let side_map = family_contexts
            .entry(collocation.fam)
            .or_default()
            .entry(collocation.side)
            .or_default();

        for cw in context_words {
            let match_kind = if cw.case_sensitive {
                Normalized
            } else {
                NoMatch
            };
            side_map
                .entry(cw.word)
                .and_modify(|existing| {
                    if *existing == Normalized && match_kind == NoMatch {
                        *existing = NoMatch;
                    }
                })
                .or_insert(match_kind);
        }
    }

    // Calculate POS from raw table data (not filtered results)
    let mut family_poses: HashMap<Option<&str>, HashMap<Side, HashSet<&'static Pos>>> =
        HashMap::new();

    for row in table {
        let fam = results
            .keys()
            .find(|k| k.alt == row.alt && k.side == row.side)
            .and_then(|k| k.fam);

        let pos_map = family_poses
            .entry(fam)
            .or_default()
            .entry(row.side)
            .or_default();

        for pos in get_poses(dict, row.ctx) {
            pos_map.insert(pos);
        }
    }

    let families: Vec<_> = family_contexts.keys().cloned().collect();
    for fam in &families {
        let mut before_unique = HashMap::new();
        let mut after_unique = HashMap::new();
        let mut before_pos_unique: HashSet<&'static Pos> = HashSet::new();
        let mut after_pos_unique: HashSet<&'static Pos> = HashSet::new();

        if let Some(fam_ctx) = family_contexts.get(fam) {
            if let Some(this_before) = fam_ctx.get(&Before) {
                for (ctx, match_kind) in this_before {
                    let is_unique =
                        families
                            .iter()
                            .filter(|other_fam| *other_fam != fam)
                            .all(|other_fam| {
                                family_contexts
                                    .get(other_fam)
                                    .and_then(|ctxs| ctxs.get(&Before))
                                    .map(|ob| !ob.contains_key(ctx))
                                    .unwrap_or(true)
                            });
                    if is_unique {
                        before_unique.insert(ctx, *match_kind);
                    }
                }
            }
            if let Some(this_after) = fam_ctx.get(&After) {
                for (ctx, match_kind) in this_after {
                    let is_unique =
                        families
                            .iter()
                            .filter(|other_fam| *other_fam != fam)
                            .all(|other_fam| {
                                family_contexts
                                    .get(other_fam)
                                    .and_then(|ctxs| ctxs.get(&After))
                                    .map(|oa| !oa.contains_key(ctx))
                                    .unwrap_or(true)
                            });
                    if is_unique {
                        after_unique.insert(ctx, *match_kind);
                    }
                }
            }
        }

        // Calculate unique POS for this family
        if let Some(fam_pos) = family_poses.get(fam) {
            if let Some(this_before_pos) = fam_pos.get(&Before) {
                for pos in this_before_pos {
                    let is_unique = families
                        .iter()
                        .filter(|other_fam| *other_fam != fam)
                        .all(|other_fam| {
                            family_poses
                                .get(other_fam)
                                .and_then(|ps| ps.get(&Before))
                                .map(|ob| !ob.contains(pos))
                                .unwrap_or(true)
                        });
                    if is_unique {
                        before_pos_unique.insert(*pos);
                    }
                }
            }
            if let Some(this_after_pos) = fam_pos.get(&After) {
                for pos in this_after_pos {
                    let is_unique = families
                        .iter()
                        .filter(|other_fam| *other_fam != fam)
                        .all(|other_fam| {
                            family_poses
                                .get(other_fam)
                                .and_then(|ps| ps.get(&After))
                                .map(|oa| !oa.contains(pos))
                                .unwrap_or(true)
                        });
                    if is_unique {
                        after_pos_unique.insert(*pos);
                    }
                }
            }
        }

        fn format_contexts(contexts: &HashMap<&&str, ContextMatch>) -> Vec<String> {
            contexts
                .iter()
                .sorted_by_key(|(s, _)| s.to_lowercase())
                .enumerate()
                .map(|(i, (s, mk))| {
                    format!(
                        "\x1b[{}m{}\x1b[0m{}",
                        32 - (i & 1),
                        s,
                        if *mk == Normalized { "†" } else { "" }
                    )
                })
                .collect()
        }

        fn format_poses(poses: &HashSet<&'static Pos>) -> String {
            poses
                .iter()
                .sorted_by_key(|pos| pos::pos_info(pos).ord)
                .enumerate()
                .map(|(i, pos)| {
                    format!("\x1b[{}m{}\x1b[0m", 33 + i % 2, pos::pos_info(pos).letter)
                })
                .join("")
        }

        print!(
            "{} \x1b[0;1m{}\x1b[0m {}\x1b[0m\x1b[K\n",
            format_contexts(&before_unique).join(" "),
            format!("«{}»", fam.unwrap_or("None")),
            format_contexts(&after_unique).join(" ")
        );

        // Print POS discriminators if any
        if !before_pos_unique.is_empty() || !after_pos_unique.is_empty() {
            print!(
                "{} \x1b[0;1m{}\x1b[0m {}\x1b[0m\x1b[K\n",
                format_poses(&before_pos_unique),
                format!("«{}»", fam.unwrap_or("None")),
                format_poses(&after_pos_unique)
            );
        }
    }
}

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

/// Main function - orchestrates the entire analysis pipeline
///
/// 1. Parse CLI arguments
/// 2. Build API URL and fetch JSON data
/// 3. Parse ngram data into structured rows
/// 4. Load dictionary for POS lookup
/// 5. Print raw diagnostics if requested
/// 6. Evaluate contexts across alternatives
/// 7. Print formatted results with POS tags
/// 8. Print prohibited contexts (discriminators)
/// 9. Print family-level uniqueness if families enabled
///
/// TODO: Add error handling for empty results
/// TODO: Consider adding a summary statistics section
/// TODO: Add support for output to file (JSON, CSV, etc.)
/// TODO: The main function is getting long - consider extracting phases
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = cli()?;
    let url = build_url(&cfg);
    let mut graph_url = url.clone();
    graph_url.path_segments_mut().unwrap().pop().push("graph");
    println!("ℹ️ URL: {}", graph_url);

    let json_val = fetch_json(url)?;
    let items: Vec<google_ngram_viewer::NgramItem> = serde_json::from_value(json_val)?;
    let table = parse_items(&items)?;
    let dict = FstDictionary::curated();

    if cfg.raw {
        print_raw_diagnostics(&cfg, &table, &dict);
    }

    let results = evaluate_contexts(&cfg, &table);
    let keys = results
        .keys() //.collect::<Vec<_>>();
        .sorted_by(|a, b| {
            a.fam
                .cmp(&b.fam)
                .then(a.alt.cmp(&b.alt))
                .then(a.side.cmp(&b.side))
        });

    for (fam_alt, gr) in &keys.into_iter().chunk_by(|k| (k.fam, k.alt)) {
        let mut side_cwords_pair = Vec::new();

        for colloc in gr {
            let cwords = results.get(colloc).unwrap();
            let cwords_str = format_context_words(cwords);
            side_cwords_pair.push((colloc.side, cwords_str));
        }

        let (fam, alt) = fam_alt;

        print_uniq_to(
            fam,
            &side_cwords_pair,
            &dict,
            alt,
            &cfg,
            &table,
        );

        print_prohibited(&cfg, &table, &dict, alt, fam);
    }

    if cfg.has_families {
        println!();
        print_family_uniqueness(&results, &dict, &table);
    }

    Ok(())
}
