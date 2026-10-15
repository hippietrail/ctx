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
mod config;
mod google_ngrams;
mod pos;
mod types;

// ============================================================================
// IMPORTS
// ============================================================================
// 1. Standard Library Imports
use std::vec::Vec;

// 2. External Crate Imports
use harper_core::spell::{Dictionary, FstDictionary};

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
    let cfg = config::cli()?;

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
