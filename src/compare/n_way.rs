// 1. Standard Library Imports
use std::collections::{BTreeMap, HashMap, HashSet};

// 2. External Crate Imports
use harper_core::spell::FstDictionary;
use itertools::Itertools;

// 3. Local Crate Imports
use crate::{
    colour::{BLUE, CYAN, Colour, GREEN, MAGENTA, ORANGE, RED, YELLOW},
    compare::{
        SideCol, format_n_side_comparison, format_pos_w_count, format_word_diff,
        get_correct_capitalization_of_string, is_meaningful_word,
    },
    config::Cfg,
    pos::{Pos, pos_info},
    types::FamilyTree,
};

const PALETTE: &[(u8, u8, u8)] = &[ORANGE, RED, BLUE, GREEN, MAGENTA, CYAN];

pub fn compare_n(dict: &FstDictionary, tree: FamilyTree, cfg: &Cfg) {
    let raw_families = tree.into_families_sorted();
    let n = raw_families.len();

    let families: Vec<(String, _, (u8, u8, u8))> = raw_families
        .into_iter()
        .enumerate()
        .map(|(i, (name, map))| (name, map, PALETTE[i % PALETTE.len()]))
        .collect();

    let header = families
        .iter()
        .map(|(name, _, color)| format!("“{}”", name.c(*color)))
        .join(" vs. ");
    println!("{header}");

    let analyze_side = |side_variant: crate::Side, label: &str| {
        let side_color = [GREEN, CYAN][side_variant as usize];
        println!("=== {} {} ===", label.c(side_color), "WORDS".c(YELLOW));

        // 1. Collect original-case and lowercased sets per family
        let mut orig_sets: Vec<HashSet<String>> = Vec::with_capacity(n);
        let mut lower_sets: Vec<HashSet<String>> = Vec::with_capacity(n);

        for (_, fam_map, _) in &families {
            let mut orig = fam_map.contexts_for_side(side_variant);
            if !cfg.raw_tokens {
                orig.retain(|w| is_meaningful_word(w));
            }
            let lower = orig.iter().map(|s| s.to_lowercase()).collect();
            orig_sets.push(orig);
            lower_sets.push(lower);
        }

        // Combined original-case lookup across all families
        let all_orig: HashSet<String> = orig_sets.iter().flatten().cloned().collect();
        let empty_set = HashSet::new();

        // 2. Map every unique lowercase word to the bitmask (or Vec<usize>) of families containing it
        let mut word_owners: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (idx, set) in lower_sets.iter().enumerate() {
            for word in set {
                word_owners.entry(word.clone()).or_default().push(idx);
            }
        }

        // Partition words into:
        // - exclusive[i]: only in family i
        // - partial[owners]: in 2..n families
        // - common_all: in all n families
        let mut exclusive: Vec<HashSet<String>> = vec![HashSet::new(); n];
        let mut partial: BTreeMap<Vec<usize>, HashSet<String>> = BTreeMap::new();
        let mut common_all: HashSet<String> = HashSet::new();

        for (word, owners) in word_owners {
            if owners.len() == 1 {
                exclusive[owners[0]].insert(word);
            } else if owners.len() == n {
                common_all.insert(word);
            } else {
                partial.entry(owners).or_default().insert(word);
            }
        }

        // Print exclusive words per family (matching compare_two style)
        for (i, (name, _, color)) in families.iter().enumerate() {
            if exclusive[i].is_empty() {
                continue;
            }
            let diff_str = format_word_diff(&exclusive[i], &orig_sets[i], &all_orig, dict, *color);
            println!(" {label} {}: {}", name.c(*color), diff_str);
        }

        // Print partial overlaps (2..n families) so pairwise battleground words aren't lost!
        for (owners, words) in &partial {
            let subset_label = owners
                .iter()
                .map(|&idx| format!("{}", families[idx].0.c(families[idx].2)))
                .join(" + ");
            let diff_str = format_word_diff(words, &all_orig, &empty_set, dict, YELLOW);
            println!(" {label} [{subset_label}]: {diff_str}");
        }

        // Print words common to all N families with superscript relative frequencies
        if cfg.common_words && !common_all.is_empty() {
            let counts_per_fam: Vec<HashMap<String, f64>> = families
                .iter()
                .map(|(_, fam_map, _)| fam_map.word_counts_for_side(side_variant, dict))
                .collect();

            let lists: Vec<Vec<(String, f64)>> = counts_per_fam
                .iter()
                .map(|counts| {
                    common_all
                        .iter()
                        .map(|low_word| {
                            let display_word = all_orig
                                .get(low_word)
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
                })
                .collect();

            let max_freq = lists
                .iter()
                .filter_map(|l| l.first().map(|x| x.1))
                .fold(0.0_f64, f64::max);

            let rendered_words: Vec<String> = lists
                .into_iter()
                .map(|list| {
                    list.into_iter()
                        .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                        .join(" ")
                })
                .collect();

            let cols: Vec<SideCol<'_>> = families
                .iter()
                .zip(rendered_words.iter())
                .map(|((name, _, color), words)| SideCol {
                    words,
                    fam: name,
                    color: *color,
                })
                .collect();

            let output = format_n_side_comparison(side_variant, &format!("{label} common"), &cols);
            println!("{}", output.c(BLUE));
        }

        // --- Parts of Speech (POS) Processing ---
        println!("=== {} {} ===", label.c(side_color), "POS".c(MAGENTA));

        let pos_counts: Vec<HashMap<&'static Pos, HashMap<String, f64>>> = families
            .iter()
            .map(|(_, fam_map, _)| fam_map.pos_to_word_counts_for_side(side_variant, dict))
            .collect();

        let mut pos_owners: BTreeMap<&'static Pos, Vec<usize>> = BTreeMap::new();
        for (idx, map) in pos_counts.iter().enumerate() {
            for &pos in map.keys() {
                pos_owners.entry(pos).or_default().push(idx);
            }
        }

        let top_words_for = |words_map: Option<&HashMap<String, f64>>| -> Vec<(String, f64)> {
            let Some(map) = words_map else {
                return Vec::new();
            };
            map.iter()
                .sorted_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)))
                .take(3)
                .map(|(word, &freq)| (word.clone(), freq))
                .collect()
        };

        for (pos, owners) in pos_owners {
            let info = pos_info(pos);
            let pos_label = format!("{}{}", info.emoji, info.letter);

            if owners.len() == 1 {
                let idx = owners[0];
                let (name, _, color) = &families[idx];
                let top = top_words_for(pos_counts[idx].get(pos));
                if top.is_empty() {
                    continue;
                }
                let max_freq = top[0].1;
                let words_str = top
                    .into_iter()
                    .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                    .join(" ");

                let output = match side_variant {
                    crate::Side::Before => format!(" {pos_label} : {words_str} : {name}"),
                    crate::Side::After => format!(" {pos_label} : {name} : {words_str}"),
                };
                println!("{}", output.c(*color));
            } else if owners.len() < n || cfg.common_pos {
                let tops: Vec<Vec<(String, f64)>> = owners
                    .iter()
                    .map(|&idx| top_words_for(pos_counts[idx].get(pos)))
                    .collect();

                let max_freq = tops
                    .iter()
                    .filter_map(|t| t.first().map(|x| x.1))
                    .fold(0.0_f64, f64::max);

                let rendered: Vec<String> = tops
                    .into_iter()
                    .map(|top| {
                        top.into_iter()
                            .map(|(word, freq)| format_pos_w_count(&word, freq, max_freq))
                            .join(" ")
                    })
                    .collect();

                let cols: Vec<SideCol<'_>> = owners
                    .iter()
                    .zip(rendered.iter())
                    .map(|(&idx, words)| SideCol {
                        words,
                        fam: &families[idx].0,
                        color: families[idx].2,
                    })
                    .collect();

                let output = format_n_side_comparison(side_variant, &pos_label, &cols);
                println!("{}", output.c(BLUE));
            }
        }
    };

    analyze_side(crate::Side::Before, "Before");
    analyze_side(crate::Side::After, "After");
}
