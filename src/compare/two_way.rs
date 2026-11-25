// 1. Standard Library Imports
use std::collections::{HashMap, HashSet};

// 2. External Crate Imports
use harper_core::spell::FstDictionary;
use itertools::Itertools;

// 3. Local Crate Imports
use crate::{
    colour::{BLUE, CYAN, Colour, GREEN, MAGENTA, ORANGE, RED, YELLOW},
    compare::{
        SideCol, format_pos_w_count, format_side_comparison, format_word_diff,
        get_correct_capitalization_of_string, is_meaningful_word,
    },
    config::Cfg,
    pos::{Pos, pos_info},
    types::FamilyTree,
};

pub fn compare_two(dict: &FstDictionary, tree: FamilyTree, cfg: &Cfg) {
    let mut families = tree.into_families_sorted();
    let (fam_name_a, fam_a) = families.remove(0);
    let (fam_name_b, fam_b) = families.remove(0);

    let (fam_name_a, fam_name_b) = (fam_name_a.as_str(), fam_name_b.as_str());

    println!("“{}” vs. “{}”", fam_name_a.c(ORANGE), fam_name_b.c(RED));

    let analyze_side = |side_variant: crate::Side, label: &str| {
        let mut original_set_a = fam_a.contexts_for_side(side_variant);
        let mut original_set_b = fam_b.contexts_for_side(side_variant);

        if !cfg.raw_tokens {
            original_set_a.retain(|w| is_meaningful_word(w));
            original_set_b.retain(|w| is_meaningful_word(w));
        }

        let word_set_a: HashSet<String> = original_set_a.iter().map(|s| s.to_lowercase()).collect();
        let word_set_b: HashSet<String> = original_set_b.iter().map(|s| s.to_lowercase()).collect();

        let side_color = [GREEN, CYAN][side_variant as usize];
        println!("=== {} {} ===", label.c(side_color), "WORDS".c(YELLOW));

        let word_intersection: HashSet<String> =
            word_set_a.intersection(&word_set_b).cloned().collect();
        let word_diff_a: HashSet<String> = word_set_a.difference(&word_set_b).cloned().collect();
        let word_diff_b: HashSet<String> = word_set_b.difference(&word_set_a).cloned().collect();

        let a_not_b =
            format_word_diff(&word_diff_a, &original_set_a, &original_set_b, dict, ORANGE);
        let b_not_a = format_word_diff(&word_diff_b, &original_set_a, &original_set_b, dict, RED);

        println!(" {label} {}: {}", fam_name_a.c(ORANGE), a_not_b);
        println!(" {label} {}: {}", fam_name_b.c(RED), b_not_a);

        if cfg.common_words && !word_intersection.is_empty() {
            let word_counts_a = fam_a.word_counts_for_side(side_variant, dict);
            let word_counts_b = fam_b.word_counts_for_side(side_variant, dict);

            let collect_sorted = |counts: &HashMap<String, f64>| -> Vec<(String, f64)> {
                word_intersection
                    .iter()
                    .map(|low_word| {
                        let display_word = original_set_a
                            .get(low_word)
                            .or_else(|| original_set_b.get(low_word))
                            .map(String::as_str)
                            .unwrap_or(low_word);
                        let canon = get_correct_capitalization_of_string(dict, display_word);
                        let freq = counts
                            .get(&canon)
                            .or_else(|| counts.get(display_word))
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

            let format_list = |list: Vec<(String, f64)>| -> String {
                list.into_iter()
                    .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                    .collect::<Vec<_>>()
                    .join(" ")
            };

            let output = format_side_comparison(
                side_variant,
                &format!("{label} common"),
                SideCol {
                    words: &format_list(list_a),
                    fam: fam_name_a,
                    color: ORANGE,
                },
                SideCol {
                    words: &format_list(list_b),
                    fam: fam_name_b,
                    color: RED,
                },
            );
            println!("{}", output.c(BLUE));
        }

        // --- Parts of Speech (Pos) Set Processing ---
        let pos_counts_a = fam_a.pos_to_word_counts_for_side(side_variant, dict);
        let pos_counts_b = fam_b.pos_to_word_counts_for_side(side_variant, dict);

        let pos_set_a: HashSet<&'static Pos> = pos_counts_a.keys().copied().collect();
        let pos_set_b: HashSet<&'static Pos> = pos_counts_b.keys().copied().collect();

        let pos_diff_a: HashSet<&'static Pos> = pos_set_a.difference(&pos_set_b).copied().collect();
        let pos_diff_b: HashSet<&'static Pos> = pos_set_b.difference(&pos_set_a).copied().collect();
        let pos_intersection: HashSet<&'static Pos> =
            pos_set_a.intersection(&pos_set_b).copied().collect();

        println!("=== {} {} ===", label.c(side_color), "POS".c(MAGENTA));

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

        for pos in pos_diff_a.iter().sorted() {
            print_top_words(pos, pos_counts_a.get(pos), fam_name_a, ORANGE);
        }

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

                let format_top = |top: Vec<(String, f64)>| -> String {
                    top.into_iter()
                        .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                        .collect::<Vec<_>>()
                        .join(" ")
                };

                let output = format_side_comparison(
                    side_variant,
                    &pos_label,
                    SideCol {
                        words: &format_top(top_a),
                        fam: fam_name_a,
                        color: ORANGE,
                    },
                    SideCol {
                        words: &format_top(top_b),
                        fam: fam_name_b,
                        color: RED,
                    },
                );
                println!("{}", output.c(BLUE));
            }
        }

        for pos in pos_diff_b.iter().sorted() {
            print_top_words(pos, pos_counts_b.get(pos), fam_name_b, RED);
        }
    };

    analyze_side(crate::Side::Before, "Before");
    analyze_side(crate::Side::After, "After");
}
