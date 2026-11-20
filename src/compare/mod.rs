mod three_way;
mod two_way;

// 1. Standard Library Imports

// 2. External Crate Imports
use harper_core::spell::{Dictionary, FstDictionary};

// 3. Local Crate Imports

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
