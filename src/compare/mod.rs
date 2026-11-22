mod three_way;
mod two_way;

// 1. Standard Library Imports
use std::collections::HashSet;

// 2. External Crate Imports
use harper_core::spell::{Dictionary, FstDictionary};
use itertools::Itertools;

// 3. Local Crate Imports
use crate::colour::Colour;

// 4. Re-exports
pub use three_way::compare_three;
pub use two_way::compare_two;

fn format_pos_w_count(word: &str, freq: f64, max_freq: f64) -> String {
    const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

    let score = if max_freq > 0.0 {
        (freq / max_freq) * 100.0
    } else {
        0.0
    };

    let mut formatted = format!("{score:.2}");
    if formatted.contains('.') {
        formatted = formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
    }

    let superscript = formatted
        .chars()
        .map(|c| match c {
            '0'..='9' => SUPERSCRIPTS[(c as usize) - ('0' as usize)],
            '.' => '˙',
            other => other,
        })
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

pub fn is_meaningful_word(s: &str) -> bool {
    let alpha_count = s.chars().filter(|c| c.is_alphabetic()).count();
    // Keep words with 2+ letters, or 1-letter words not ending in '.' (skips "1.", "•", "a.")
    alpha_count > 1 || (alpha_count == 1 && !s.ends_with('.'))
}

pub fn format_word_diff(
    diff: &HashSet<String>,
    orig_a: &HashSet<String>,
    orig_b: &HashSet<String>,
    dict: &FstDictionary,
    color: (u8, u8, u8),
) -> String {
    diff.iter()
        .sorted()
        .map(|low_word| {
            let display_word = orig_a
                .get(low_word)
                .or_else(|| orig_b.get(low_word))
                .map(String::as_str)
                .unwrap_or(low_word);
            let is_oov = dict.get_word_metadata_str(display_word).is_none();
            let canon = get_correct_capitalization_of_string(dict, display_word);
            if is_oov {
                format!("{}", canon.d().c(color))
            } else {
                format!("{}", canon.c(color))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub struct SideCol<'a> {
    pub words: &'a str,
    pub fam: &'a str,
    pub color: (u8, u8, u8),
}

pub fn format_side_comparison(
    side: crate::Side,
    prefix: &str,
    a: SideCol<'_>,
    b: SideCol<'_>,
) -> String {
    match side {
        crate::Side::Before => format!(
            " {prefix} : [{} : {}] vs [{} : {}]",
            a.words,
            a.fam.c(a.color),
            b.words,
            b.fam.c(b.color)
        ),
        crate::Side::After => format!(
            " {prefix} : [{} : {}] vs [{} : {}]",
            a.fam.c(a.color),
            a.words,
            b.fam.c(b.color),
            b.words
        ),
    }
}
