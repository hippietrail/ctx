// //! # Context Pattern Analyzer
// //!
// //! A tool for analyzing collocation and context patterns to help distinguish
// //! between similar or confusable words. This is primarily useful for creating
// //! grammar checker rules and linters.
// //!
// //! ## Purpose
// //!
// //! The tool analyzes word alternatives using Google Ngram Viewer data to identify
// //! unique contextual usage patterns. For example:
// //!
// //! - **Spelling variants**: "tomato" vs "tomatoe"
// //! - **Contractions vs full forms**: "can't" vs "cant"
// //! - **Pluralization patterns**: "pastas" vs "spaghettis"
// //! - **Part-of-speech ambiguity**: "dose" (verb/noun) vs "does" (verb only)
// //!
// //! By identifying unique contexts (words that appear with one alternative but
// //! not others) and prohibited contexts (words that appear with all other
// //! alternatives but not the current one), you can create rules to detect
// //! when a word might be a mistake.
// //!
// //! ## Features
// //!
// //! - Fetches context words from Google Ngram Viewer (before/after each alternative)
// //! - Uses part-of-speech tagging to categorize context words
// //! - Identifies unique contexts (🟢) - words that appear with only one alternative
// //! - Identifies prohibited contexts (🚫) - words that appear with all other alternatives
// //! - Supports family grouping for comparing related alternatives
// //! - Raw diagnostic output with POS tags for debugging
// //!
// //! ## Family Grouping
// //!
// //! Families allow comparing groups of related alternatives:
// //! - Case variants: lowercase vs titlecase of the same word
// //! - Compound forms: spaced compounds vs hyphenated compounds
// //! - Any other grouping where you want to compare within the group but not across groups

// ============================================================================
// MODULES (Declare these first so Rust knows they exist)
// ============================================================================
mod colour;
mod compare;
mod google_ngrams;
mod pos;
mod types;

// ============================================================================
// IMPORTS
// ============================================================================
// 1. Standard Library Imports
// use std::collections::HashSet;
use std::env;
use std::vec::Vec;

// 2. External Crate Imports
use harper_core::spell::{Dictionary, FstDictionary};
use itertools::Itertools;

// 3. Local Module Imports (Now valid because the modules are declared above)
use compare::{compare_three, compare_two};
use pos::{Pos, PosLookupResult};
use types::FamilyTree;

// ============================================================================
// TYPES AND ITEMS
// ============================================================================

/// Represents which side of the target word a context appears on
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum Side {
    Before,
    After,
}
use Side::*;

#[derive(Debug)]
struct Row {
    pub fam: Option<String>, // For grouping alternatives together
    pub alt: String,         // An individual word or phrase being compared against others
    pub side: Side, // Does this row contain the context words before the 'alternative' or after?
    pub ctx: String, // A single context word that frequently appears right before or right after this 'alternative'.
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
/// - `raw`: Reserved for future raw diagnostic output
/// - `alternatives`: List of word alternatives to analyze
/// - `has_families`: Whether family groupings are enabled
pub struct Cfg {
    pub debug: bool,
    pub raw: bool,
    pub alternatives: Vec<Alternative>,
    pub has_families: bool,
    pub since_year: Option<[char; 4]>,
}

// ============================================================================
// COMMAND-LINE INTERFACE
// ============================================================================

/// Parses command-line arguments into a configuration struct
///
/// Supported arguments:
/// - `--raw`: Enable raw diagnostic output
/// - `--family=<name>`, `--fam=<name>`, `-f=<name>`: Set family for subsequent alternatives
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
            if cfg.debug {
                if let Some(f) = &family {
                    println!("👉Family: {}", f);
                } else {
                    unreachable!()
                }
            }
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
                _ => return Err("Year must be exactly 4 digits".into()),
            };
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: {}", arg).into());
        } else if arg.contains('-') {
            // Hyphenated phrases: replace hyphens with " - " for Ngram JSON API compatibility
            // NOTE: See also the special handling for apostrophes in google_ngrams.rs/build_url()
            cfg.alternatives.push(Alternative {
                raw: arg.clone(),
                jfmt: arg.split('-').join(" - "),
                fam: family.clone(),
            });
        } else if arg.contains('\'') {
            // Special handling when any word starts or ends with an apostrophe, such as 'tis or 'nother
            // NOTE: See also the special handling for apostrophes in google_ngrams.rs/build_url()
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
            if cfg.debug {
                if let Some(family) = &family {
                    println!("👉Word: {}::{}", family, arg);
                } else {
                    println!("👉Word: {}", arg);
                }
            }
            cfg.alternatives.push(Alternative {
                raw: arg.clone(),
                jfmt: arg.clone(),
                fam: family.clone(),
            });
        }
    }

    // --- POST-PARSING NORMALIZATION ---
    if !cfg.has_families {
        // If no families were specified anywhere on the CLI,
        // the family can just copy the raw alternative.
        for alt in cfg.alternatives.iter_mut() {
            alt.fam = Some(alt.raw.clone());
        }
    }

    Ok(cfg)
}

// ============================================================================
// PART-OF-SPEECH LOOKUP
// ============================================================================

/// Gets the part-of-speech tags for a given word
///
/// Uses the harper_core dictionary to look up word metadata
/// and returns matching POS tags based on predefined predicates.
///
/// Explicitly distinguishes between:
/// - Words not in the dictionary (true OOV)
/// - Words in dictionary but matching no POS predicates
/// - Words in dictionary with matching POS tags
fn get_poses(dict: &FstDictionary, word: &str) -> pos::PosLookupResult {
    match dict.get_word_metadata_str(word) {
        None => pos::PosLookupResult::NotFound,
        Some(md) => {
            let matches: Vec<&'static Pos> = pos::POS_DEFINITIONS
                .iter()
                .filter(|&(_, pred)| pred(&md))
                .map(|(enum_variant, _)| enum_variant)
                .collect();

            if matches.is_empty() {
                pos::PosLookupResult::FoundWithNoMatches
            } else {
                pos::PosLookupResult::FoundWithMatches(matches)
            }
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
/// 5. Build a hierarchical data structure (FamilyTree -> AlternativeMap -> SideMap -> ContextSet)
/// 6. Compare two families of alternatives using set operations (union, intersection, difference)
/// 7. Display results showing which context words and POS tags are unique to each family
///
/// The current implementation focuses on comparing exactly 2 families of alternatives,
/// with color-coded output showing shared vs unique contexts.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = cli()?;

    let table = google_ngrams::fetch_google_ngrams(&cfg)?;

    // --- EXECUTION CONTEXT ---

    let dict = FstDictionary::curated();
    let mut tree = FamilyTree::default();

    for row in table {
        let poses: Vec<&'static Pos> = match get_poses(&dict, &row.ctx) {
            PosLookupResult::NotFound => {
                // Word not in dictionary - true OOV, include with empty POS
                Vec::new()
            }
            PosLookupResult::FoundWithNoMatches => {
                // Word in dictionary but matches no POS predicates
                Vec::new()
            }
            PosLookupResult::FoundWithMatches(matches) => matches,
        };

        // 1. Normalize the grouping key:
        // If explicit family exists, use it.
        // If None, isolate this alternative into its own unique family group!
        let family_key = match &row.fam {
            Some(fam_name) => Some(fam_name.clone()),
            None => Some(row.alt.clone()), // Fall back to alternative name as the family name
        };

        // 2. Build the tree with the normalized key
        tree.entry(family_key)
            .or_default()
            .entry(row.alt.clone())
            .or_default()
            .entry(row.side)
            .or_default()
            .entry(row.ctx.clone())
            .or_default()
            .extend(poses);
    }

    println!(
        "Actual family keys in tree: {:?}",
        tree.keys().collect::<Vec<_>>()
    );

    match tree.len() {
        2 => compare_two(&dict, tree),
        3 => compare_three(&dict, tree),
        _ => {
            println!(
                "Tree size must be exactly 2 or 3 to run analysis, but got {}",
                tree.len()
            );
        }
    }

    Ok(())
}
