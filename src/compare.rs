// 1. Standard Library Imports
use std::collections::{HashMap, HashSet};

// 2. External Crate Imports
use harper_core::spell::{Dictionary, FstDictionary};
use itertools::Itertools;

// 3. Local Crate Imports
use crate::colour::{BLUE, CYAN, Colour, GREEN, MAGENTA, ORANGE, RED, YELLOW};
use crate::pos::{Pos, pos_info}; // Fixed path: assuming Pos is inside pos
use crate::types::FamilyTree; // Fixed path: assuming FamilyTree is inside types

pub fn compare_two(dict: &FstDictionary, tree: FamilyTree) {
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
        // println!(" 🗂️ {}", [a_not_b, both, b_not_a].join(", "));
        println!(" {label} {}: {}", fam_name_a.c(ORANGE), a_not_b.join(", "));
        println!(" {label} {}: {}", fam_name_b.c(RED), b_not_a.join(", "));

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
        let pos_intersection: HashSet<&'static Pos> = pos_set_a.intersection(&pos_set_b).copied().collect();

        println!("=== {} {} ===", label.c(side_color), "POS".c(pos_colour));

        let print_top_words =

            |pos, words_map: Option<&HashMap<String, usize>>, fam_name: &str, color| {
                let words = match words_map {
                    Some(m) if !m.is_empty() => m,
                    _ => return, // Skip if None or empty
                };

                let words_str = words
                    .iter()
                    .sorted_by(|a, b| b.1.cmp(a.1)) // Sort by count descending
                    .take(3)
                    .map(|(word, count)| format_pos_w_count(word, count))
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
        for pos in pos_intersection.iter().sorted() {
            let info = pos_info(pos);
            let pos_label = format!("{}{}", info.emoji, info.letter);

            // Format helper for the word subsets
            let get_words_str = |words_map: Option<&HashMap<String, usize>>| -> Option<String> {
                let map = words_map?;
                if map.is_empty() { return None; }
                
                Some(map
                    .iter()
                    .sorted_by(|a, b| b.1.cmp(a.1))
                    .take(3)
                    .map(|(word, count)| format_pos_w_count(word, count))
                    .collect::<Vec<_>>()
                    .join(" "))
            };

            let words_a = get_words_str(pos_counts_a.get(pos)).unwrap_or_default();
            let words_b = get_words_str(pos_counts_b.get(pos)).unwrap_or_default();

            // Construct unified single-line outputs depending on side layout
            let output = match side_variant {
                crate::Side::Before => format!(
                    " {pos_label} : [{} : {}] vs [{} : {}]",
                    words_a, fam_name_a.c(ORANGE),
                    words_b, fam_name_b.c(RED)
                ),
                crate::Side::After => format!(
                    " {pos_label} : [{} : {}] vs [{} : {}]",
                    fam_name_a.c(ORANGE), words_a,
                    fam_name_b.c(RED), words_b
                ),
            };

            println!("{}", output.c(BLUE));
        }

        // 5. Print unique POS for Family B
        for pos in pos_diff_b.iter().sorted() {
            print_top_words(pos, pos_counts_b.get(pos), fam_name_b, RED);
        }
    };

    analyze_side(crate::Side::Before, "Before");
    analyze_side(crate::Side::After, "After");
}

pub fn compare_three(dict: &FstDictionary, tree: FamilyTree) {
    // 1. Extract family names and their corresponding AlternativeMaps in sorted order
    let (family_keys, fam_a, fam_b, fam_c) = tree.into_three_families_sorted();
    let fam_name_a = family_keys[0].as_deref().unwrap_or("None");
    let fam_name_b = family_keys[1].as_deref().unwrap_or("None");
    let fam_name_c = family_keys[2].as_deref().unwrap_or("None");

    println!(
        "“{}” vs. “{}” vs. “{}”",
        fam_name_a.c(ORANGE),
        fam_name_b.c(RED),
        fam_name_c.c(BLUE)
    );

    // 2. The unified reusable analysis pipeline closure
    let analyze_side = |label: &str, side_variant: crate::Side| {
        // Keep the exact case variants as they came out of the text data
        let original_set_a = fam_a.contexts_for_side(side_variant);
        let original_set_b = fam_b.contexts_for_side(side_variant);
        let original_set_c = fam_c.contexts_for_side(side_variant);

        // Construct case-insensitive tracking sets so case variants merge for set arithmetic
        let word_set_a: HashSet<String> = original_set_a.iter().map(|s| s.to_lowercase()).collect();
        let word_set_b: HashSet<String> = original_set_b.iter().map(|s| s.to_lowercase()).collect();
        let word_set_c: HashSet<String> = original_set_c.iter().map(|s| s.to_lowercase()).collect();

        let (word_color, pos_colour) = (YELLOW, MAGENTA);
        let side_color = [GREEN, CYAN][side_variant as usize];
        println!("=== {} {} ===", label.c(side_color), "WORDS".c(word_color));

        // Calculate unions for comparison
        let union_bc: HashSet<String> = word_set_b.union(&word_set_c).cloned().collect();
        let union_ac: HashSet<String> = word_set_a.union(&word_set_c).cloned().collect();
        let union_ab: HashSet<String> = word_set_a.union(&word_set_b).cloned().collect();

        // Calculate unique sets: words in one family but not in the union of the other two
        let unique_a: HashSet<String> = word_set_a.difference(&union_bc).cloned().collect();
        let unique_b: HashSet<String> = word_set_b.difference(&union_ac).cloned().collect();
        let unique_c: HashSet<String> = word_set_c.difference(&union_ab).cloned().collect();

        // Calculate intersection of all three
        let intersection_all: HashSet<String> = word_set_a
            .intersection(&word_set_b)
            .cloned()
            .collect::<HashSet<String>>()
            .intersection(&word_set_c)
            .cloned()
            .collect();

        // Helper to pull the true original case variant out of the source text data
        let get_raw_input_word = |lowercase_word: &str| -> String {
            if let Some(orig) = original_set_a.get(lowercase_word) {
                return orig.clone();
            }
            if let Some(orig) = original_set_b.get(lowercase_word) {
                return orig.clone();
            }
            if let Some(orig) = original_set_c.get(lowercase_word) {
                return orig.clone();
            }
            lowercase_word.to_string()
        };

        let mut combined_pos = Vec::new();

        // Group 1: Unique to A (Words only in A, not in B or C)
        for low_word in unique_a.iter().sorted() {
            let display_word = get_raw_input_word(low_word);
            let is_oov = dict.get_word_metadata_str(&display_word).is_none();
            let canonical_word = get_correct_capitalization_of_string(dict, &display_word);

            let formatted = if is_oov {
                format!("{}", canonical_word.d().c(ORANGE))
            } else {
                format!("{}", canonical_word.c(ORANGE))
            };
            combined_pos.push(formatted);
        }

        // Group 2: Unique to B (Words only in B, not in A or C)
        for low_word in unique_b.iter().sorted() {
            let display_word = get_raw_input_word(low_word);
            let is_oov = dict.get_word_metadata_str(&display_word).is_none();
            let canonical_word = get_correct_capitalization_of_string(dict, &display_word);

            let formatted = if is_oov {
                format!("{}", canonical_word.d().c(RED))
            } else {
                format!("{}", canonical_word.c(RED))
            };
            combined_pos.push(formatted);
        }

        // Group 3: Unique to C (Words only in C, not in A or B)
        for low_word in unique_c.iter().sorted() {
            let display_word = get_raw_input_word(low_word);
            let is_oov = dict.get_word_metadata_str(&display_word).is_none();
            let canonical_word = get_correct_capitalization_of_string(dict, &display_word);

            let formatted = if is_oov {
                format!("{}", canonical_word.d().c(BLUE))
            } else {
                format!("{}", canonical_word.c(BLUE))
            };
            combined_pos.push(formatted);
        }

        // Group 4: Shared by all three
        for low_word in intersection_all.iter().sorted() {
            let display_word = get_raw_input_word(low_word);
            let is_oov = dict.get_word_metadata_str(&display_word).is_none();
            let canonical_word = get_correct_capitalization_of_string(dict, &display_word);

            let formatted = if is_oov {
                format!("{}", canonical_word.d().b())
            } else {
                format!("{}", canonical_word.b())
            };
            combined_pos.push(formatted);
        }
        println!(" 🗂️ {}", combined_pos.join(", "));

        // --- Parts of Speech (Pos) Set Processing ---
        let pos_set_a = fam_a.poses_for_side(side_variant);
        let pos_set_b = fam_b.poses_for_side(side_variant);
        let pos_set_c = fam_c.poses_for_side(side_variant);

        println!("=== {} {} ===", label.c(side_color), "POS".c(pos_colour));

        let pos_union: HashSet<&'static Pos> = pos_set_a
            .union(&pos_set_b)
            .copied()
            .collect::<HashSet<&'static Pos>>()
            .union(&pos_set_c)
            .copied()
            .collect();

        // POS combined with color coding
        let mut combined = Vec::new();
        for pos in pos_union {
            let in_a = pos_set_a.contains(&pos);
            let in_b = pos_set_b.contains(&pos);
            let in_c = pos_set_c.contains(&pos);

            if in_a && in_b && in_c {
                combined.push(format!("\x1b[1m{:?}\x1b[0m", pos)); // Bold for pos in all three
            } else if in_a && in_b {
                combined.push(format!("\x1b[33m{:?}\x1b[0m", pos)); // Yellow for pos in A and B only
            } else if in_a && in_c {
                combined.push(format!("\x1b[36m{:?}\x1b[0m", pos)); // Cyan for pos in A and C only
            } else if in_b && in_c {
                combined.push(format!("\x1b[35m{:?}\x1b[0m", pos)); // Magenta for pos in B and C only
            } else if in_a {
                combined.push(format!("\x1b[33m{:?}\x1b[0m", pos)); // Orange for pos only in A
            } else if in_b {
                combined.push(format!("\x1b[31m{:?}\x1b[0m", pos)); // Red for pos only in B
            } else {
                combined.push(format!("\x1b[34m{:?}\x1b[0m", pos)); // Blue for pos only in C
            }
        }
        println!("  {}", combined.join(", "));
    };

    // 3. Execute everything seamlessly for the LEFT side
    analyze_side("LEFT", crate::Side::Before);

    // 4. Execute everything seamlessly for the RIGHT side
    analyze_side("RIGHT", crate::Side::After);
}

fn format_pos_w_count(word: &str, count: &usize) -> String {
    const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

    let superscript = count
        .to_string()
        .chars()
        .map(|c| SUPERSCRIPTS[(c as usize) - ('0' as usize)])
        .collect::<String>();

    format!("{word}{superscript}")
}

// Wrapper that converts string to &[char] for Harper's API then back to String
fn get_correct_capitalization_of_string(dict: &FstDictionary, s: &str) -> String {
    let s_chars: Vec<char> = s.chars().collect();
    dict.get_correct_capitalization_of(&s_chars)
        // If a match is found, return it cleanly as a String
        .map(|v| v.iter().collect::<String>())
        // If no match is found, wrap the original string in quotes
        .unwrap_or_else(|| format!("«{}»", s))
}
