// ============================================================================
// ALTERNATIVES TRANSFORMATION PIPELINE
// ============================================================================
//! Level of indirection between raw command-line parsing and query execution.
//!
//! This module provides a programmatic transformation stage for `cfg.alternatives`
//! before n-grams are queried from Google Ngrams or analyzed.
//!
//! ### Pipeline Stages:
//! 1. **Casing & Orthographic Expansion (`--expand-families`)**:
//!    - Generates Sentence case, Title Case, and ALL CAPS variants for each phrase.
//!
//! 2. **Affixing (Prepending / Appending words)** *(Planned)*:
//!    - Prepending or appending context words, particles, or grammatical frames
//!      (e.g., prepending modals "will", "can", or "to", or appending prepositions).
//!
//! 3. **Verb Lemma Inflection Expansion** *(Planned)*:
//!    - Detecting verb lemmas within alternatives and expanding them automatically
//!      into their full inflection paradigm (e.g. "look forward to" -> "looks forward to",
//!      "looked forward to", "looking forward to").

use std::collections::HashSet;

use crate::Alternative;
use crate::config::Cfg;

/// Programmatically modifies the set of alternatives in the configuration.
///
/// This function is called immediately after CLI argument parsing and before
/// any remote queries or analysis pipelines are executed.
pub fn transform_alternatives(cfg: &mut Cfg) -> Result<(), Box<dyn std::error::Error>> {
    // ------------------------------------------------------------------------
    // Transformation pipeline passes:
    // ------------------------------------------------------------------------

    // 1. Orthographic / casing expansion if requested
    if cfg.expand_families {
        expand_family_casings(cfg);
    }

    // 2. Future: Verb lemma expansion
    //   expand_verb_lemmas(cfg)?;

    // 3. Future: Affixing (prepending / appending words)
    //   apply_affixes(cfg, &prefix, &suffix)?;

    Ok(())
}

/// Generates sentence-capitalized, titlecase, and ALL CAPS variants for each alternative.
fn expand_family_casings(cfg: &mut Cfg) {
    let mut xpanded = Vec::<Alternative>::new();
    let mut seen = HashSet::new();

    // Pre-populate with existing raw alternatives to avoid redundant duplicates
    for alt in &cfg.alternatives {
        seen.insert(alt.raw.clone());
    }

    for alt in cfg.alternatives.iter() {
        let (raw, fam): (&String, &Option<String>) = (&alt.raw, &alt.fam);

        // Split into owned strings so we can manipulate and reuse them
        let parts: Vec<String> = raw
            .split(&[' ', '-'][..])
            .map(|s| s.to_string())
            .collect();

        if parts.is_empty() || parts[0].is_empty() {
            continue;
        }

        // --- 1. Sentence-capitalized version ---
        let mut sentence_chars = parts[0].chars();
        let first_part_titlecase = match sentence_chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + sentence_chars.as_str(),
        };

        // Recombine the first titlecase word with the rest of the original words
        let mut sentence_version = first_part_titlecase;
        if parts.len() > 1 {
            sentence_version.push(' ');
            sentence_version.push_str(&parts[1..].join(" "));
        }

        if seen.insert(sentence_version.clone()) {
            if cfg.debug {
                println!("🧩 SC {}", sentence_version);
            }
            xpanded.push(Alternative {
                raw: sentence_version.clone(),
                jfmt: sentence_version,
                fam: fam.clone(),
            });
        }

        // --- 2. Each word in Titlecase version ---
        let titlecase_version = parts
            .iter()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        if seen.insert(titlecase_version.clone()) {
            if cfg.debug {
                println!("🧩 TC {}", titlecase_version);
            }
            xpanded.push(Alternative {
                raw: titlecase_version.clone(),
                jfmt: titlecase_version,
                fam: fam.clone(),
            });
        }

        // --- 3. Each word in ALL CAPS version ---
        let all_caps_version = parts
            .iter()
            .map(|word| word.to_uppercase())
            .collect::<Vec<String>>()
            .join(" ");

        if seen.insert(all_caps_version.clone()) {
            if cfg.debug {
                println!("🧩 AC {}", all_caps_version);
            }
            xpanded.push(Alternative {
                raw: all_caps_version.clone(),
                jfmt: all_caps_version,
                fam: fam.clone(),
            });
        }
    }

    cfg.alternatives.extend(xpanded);
}
