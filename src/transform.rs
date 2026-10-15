// ============================================================================
// ALTERNATIVES TRANSFORMATION PIPELINE
// ============================================================================
//! Level of indirection between raw command-line parsing and query execution.
//!
//! This module provides a programmatic transformation stage for `cfg.alternatives`
//! before n-grams are queried from Google Ngrams or analyzed.
//!
//! ### Intended Transformations:
//! 1. **Affixing (Prepending / Appending words)**:
//!    - Prepending or appending context words, particles, or grammatical frames
//!      (e.g., prepending modals "will", "can", or "to", or appending prepositions).
//!
//! 2. **Verb Lemma Inflection Expansion**:
//!    - Detecting verb lemmas within alternatives and expanding them automatically
//!      into their full inflection paradigm (e.g. "look forward to" -> "looks forward to",
//!      "looked forward to", "looking forward to"), saving users from manually enumerating
//!      every tense and aspect on the CLI.
//!
//! 3. **Orthographic & Casing Expansions**:
//!    - Coordinating case patterns (lowercase, sentence-capitalized, titlecase, uppercase)
//!      and hyphen/punctuation variants.
//!
//! Currently, this stage acts as a pass-through / identity placeholder so the pipeline
//! structure is established before specific transform functions are introduced.

use crate::config::Cfg;

/// Programmatically modifies the set of alternatives in the configuration.
///
/// This function is called immediately after CLI argument parsing and before
/// any remote queries or analysis pipelines are executed.
pub fn transform_alternatives(cfg: &mut Cfg) -> Result<(), Box<dyn std::error::Error>> {
    // ------------------------------------------------------------------------
    // Future programmatic transformations will be chained here:
    //
    // Example 1: Lemma expansion
    //   expand_verb_lemmas(&mut cfg.alternatives)?;
    //
    // Example 2: Affixing (prepending / appending words)
    //   apply_affixes(&mut cfg.alternatives, &prefix, &suffix)?;
    //
    // Example 3: Deduplication / normalization
    //   deduplicate_alternatives(&mut cfg.alternatives);
    // ------------------------------------------------------------------------

    if cfg.debug {
        println!("🔧 Alternatives transformation stage (pass-through)");
    }

    Ok(())
}
