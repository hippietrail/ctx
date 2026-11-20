// 1. Standard Library Imports
use std::collections::{HashMap, HashSet};

// 2. External Crate Imports
use harper_core::spell::{Dictionary, FstDictionary};
use itertools::Itertools;

// 3. Local Crate Imports
use crate::{
    colour::{BLUE, CYAN, Colour, GREEN, MAGENTA, ORANGE, RED, YELLOW},
    compare::{format_pos_w_count, get_correct_capitalization_of_string},
    config::Cfg,
    pos::{Pos, pos_info},
    types::FamilyTree,
};

pub fn compare_two(dict: &FstDictionary, tree: FamilyTree, cfg: &Cfg) {
    // 1. Extract family names and their corresponding AlternativeMaps in sorted order
    let (family_keys, fam_a, fam_b) = tree.into_two_families_sorted();
    let fam_name_a = family_keys[0].as_deref().unwrap_or("None");
    let fam_name_b = family_keys[1].as_deref().unwrap_or("None");

    println!("“{}” vs. “{}”", fam_name_a.c(ORANGE), fam_name_b.c(RED));

    // 2. The unified reusable analysis pipeline closure
    let analyze_side = |side_variant: crate::Side, label: &str| {
        // Keep the exact case variants as they came out of the text data
        let original_set_a = fam_a.contexts_for_side(side_variant);
        let original_set_b = fam_b.contexts_for_side(side_variant);

        // Construct case-insensitive tracking sets so case variants merge for set arithmetic
        let word_set_a: HashSet<String> = original_set_a.iter().map(|s| s.to_lowercase()).collect();
        let word_set_b: HashSet<String> = original_set_b.iter().map(|s| s.to_lowercase()).collect();

        let (word_color, pos_colour) = (YELLOW, MAGENTA);
        let side_color = [GREEN, CYAN][side_variant as usize];
        println!("=== {} {} ===", label.c(side_color), "WORDS".c(word_color));

        let _word_union: HashSet<String> = word_set_a.union(&word_set_b).cloned().collect();
        let word_intersection: HashSet<String> =
            word_set_a.intersection(&word_set_b).cloned().collect();
        let word_diff_a: HashSet<String> = word_set_a.difference(&word_set_b).cloned().collect();
        let word_diff_b: HashSet<String> = word_set_b.difference(&word_set_a).cloned().collect();

        // Helper to pull the true original case variant out of the source text data
        let get_raw_input_word = |lowercase_word: &str| -> String {
            if let Some(orig) = original_set_a.get(lowercase_word) {
                return orig.clone();
            }
            if let Some(orig) = original_set_b.get(lowercase_word) {
                return orig.clone();
            }
            lowercase_word.to_string()
        };

        // let mut combined_pos = Vec::new();
        let mut a_not_b = Vec::new();
        let mut both = Vec::new();
        let mut b_not_a = Vec::new();

        // Group 1: Diff A (Words unique to A)
        for low_word in word_diff_a.iter().sorted() {
            let display_word = get_raw_input_word(low_word);
            let is_oov = dict.get_word_metadata_str(&display_word).is_none();
            let canonical_word = get_correct_capitalization_of_string(dict, &display_word);

            let formatted = if is_oov {
                format!("{}", canonical_word.d().c(ORANGE))
            } else {
                format!("{}", canonical_word.c(ORANGE))
            };
            a_not_b.push(formatted);
        }

        // Group 2: Intersection (Words shared by both)
        for w in word_intersection.iter().sorted() {
            let display_word = get_raw_input_word(w);
            let is_oov = dict.get_word_metadata_str(&display_word).is_none();
            let canon = get_correct_capitalization_of_string(dict, &display_word);

            let formatted = if is_oov {
                format!("{}", canon.d().b())
            } else {
                format!("{}", canon.b())
            };
            both.push(formatted);
        }

        // Group 3: Diff B (Words unique to B)
        for w in word_diff_b.iter().sorted() {
            let display_word = get_raw_input_word(w);
            let is_oov = dict.get_word_metadata_str(&display_word).is_none();
            let canonical_word = get_correct_capitalization_of_string(dict, &display_word);

            let formatted = if is_oov {
                format!("{}", canonical_word.d().c(RED))
            } else {
                format!("{}", canonical_word.c(RED))
            };
            b_not_a.push(formatted);
        }
        println!(" {label} {}: {}", fam_name_a.c(ORANGE), a_not_b.join(", "));
        println!(" {label} {}: {}", fam_name_b.c(RED), b_not_a.join(", "));

        // Common words (Format Option A: bracketed comparison in BLUE)
        if cfg.common_words && !word_intersection.is_empty() {
            let word_counts_a = fam_a.word_counts_for_side(side_variant, dict);
            let word_counts_b = fam_b.word_counts_for_side(side_variant, dict);

            let collect_sorted = |counts: &HashMap<String, f64>| -> Vec<(String, f64)> {
                word_intersection
                    .iter()
                    .map(|low_word| {
                        let display_word = get_raw_input_word(low_word);
                        let canon = get_correct_capitalization_of_string(dict, &display_word);
                        let freq = counts
                            .get(&canon)
                            .or_else(|| counts.get(&display_word))
                            .or_else(|| counts.get(low_word))
                            .copied()
                            .unwrap_or(0.0);
                        (canon, freq)
                    })
                    .sorted_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)))
                    .collect()
            };

            let list_a = collect_sorted(&word_counts_a);
            let list_b = collect_sorted(&word_counts_b);

            let max_freq = list_a
                .first()
                .map(|x| x.1)
                .unwrap_or(0.0)
                .max(list_b.first().map(|x| x.1).unwrap_or(0.0));

            let words_a = list_a
                .into_iter()
                .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                .collect::<Vec<_>>()
                .join(" ");

            let words_b = list_b
                .into_iter()
                .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                .collect::<Vec<_>>()
                .join(" ");

            let output = match side_variant {
                crate::Side::Before => format!(
                    " {label} common : [{} : {}] vs [{} : {}]",
                    words_a,
                    fam_name_a.c(ORANGE),
                    words_b,
                    fam_name_b.c(RED)
                ),
                crate::Side::After => format!(
                    " {label} common : [{} : {}] vs [{} : {}]",
                    fam_name_a.c(ORANGE),
                    words_a,
                    fam_name_b.c(RED),
                    words_b
                ),
            };

            println!("{}", output.c(BLUE));
        }

        // --- Parts of Speech (Pos) Set Processing ---
        // Build the counting structures: POS → (canonical word → count)
        let pos_counts_a = fam_a.pos_to_word_counts_for_side(side_variant, dict);
        let pos_counts_b = fam_b.pos_to_word_counts_for_side(side_variant, dict);

        // 1. Extract the unique sets of POS for each family
        let pos_set_a: HashSet<&'static Pos> = pos_counts_a.keys().copied().collect();
        let pos_set_b: HashSet<&'static Pos> = pos_counts_b.keys().copied().collect();

        // 2. Compute differences and intersections
        let pos_diff_a: HashSet<&'static Pos> = pos_set_a.difference(&pos_set_b).copied().collect();
        let pos_diff_b: HashSet<&'static Pos> = pos_set_b.difference(&pos_set_a).copied().collect();
        let pos_intersection: HashSet<&'static Pos> =
            pos_set_a.intersection(&pos_set_b).copied().collect();

        println!("=== {} {} ===", label.c(side_color), "POS".c(pos_colour));

        let top_words_for = |words_map: Option<&HashMap<String, f64>>| -> Vec<(String, f64)> {
            let Some(map) = words_map else {
                return Vec::new();
            };
            map.iter()
                .sorted_by(|a, b| b.1.total_cmp(a.1).then_with(|| a.0.cmp(b.0)))
                .take(3)
                .map(|(word, &freq)| (word.clone(), freq))
                .collect()
        };

        let print_top_words =
            |pos, words_map: Option<&HashMap<String, f64>>, fam_name: &str, color| {
                let top = top_words_for(words_map);
                if top.is_empty() {
                    return;
                }
                let max_freq = top[0].1;
                let words_str = top
                    .into_iter()
                    .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                    .collect::<Vec<_>>()
                    .join(" ");

                let info = pos_info(pos);
                let pos_label = format!("{}{}", info.emoji, info.letter);

                let output = match side_variant {
                    crate::Side::Before => format!(" {pos_label} : {words_str} : {fam_name}"),
                    crate::Side::After => format!(" {pos_label} : {fam_name} : {words_str}"),
                };

                println!("{}", output.c(color));
            };

        // 3. Print unique POS for Family A
        for pos in pos_diff_a.iter().sorted() {
            print_top_words(pos, pos_counts_a.get(pos), fam_name_a, ORANGE);
        }

        // 4. Print shared POS (Intersection) on a single combined line
        if cfg.common_pos {
            for pos in pos_intersection.iter().sorted() {
                let info = pos_info(pos);
                let pos_label = format!("{}{}", info.emoji, info.letter);

                let top_a = top_words_for(pos_counts_a.get(pos));
                let top_b = top_words_for(pos_counts_b.get(pos));
                let max_freq = top_a
                    .first()
                    .map(|x| x.1)
                    .unwrap_or(0.0)
                    .max(top_b.first().map(|x| x.1).unwrap_or(0.0));

                let words_a = top_a
                    .into_iter()
                    .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                    .collect::<Vec<_>>()
                    .join(" ");

                let words_b = top_b
                    .into_iter()
                    .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                    .collect::<Vec<_>>()
                    .join(" ");

                // Construct unified single-line outputs depending on side layout
                let output = match side_variant {
                    crate::Side::Before => format!(
                        " {pos_label} : [{} : {}] vs [{} : {}]",
                        words_a,
                        fam_name_a.c(ORANGE),
                        words_b,
                        fam_name_b.c(RED)
                    ),
                    crate::Side::After => format!(
                        " {pos_label} : [{} : {}] vs [{} : {}]",
                        fam_name_a.c(ORANGE),
                        words_a,
                        fam_name_b.c(RED),
                        words_b
                    ),
                };

                println!("{}", output.c(BLUE));
            }
        }

        // 5. Print unique POS for Family B
        for pos in pos_diff_b.iter().sorted() {
            print_top_words(pos, pos_counts_b.get(pos), fam_name_b, RED);
        }
    };

    analyze_side(crate::Side::Before, "Before");
    analyze_side(crate::Side::After, "After");
}
