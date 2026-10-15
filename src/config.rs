// Standard library imports
use std::env;

// External crate imports
use itertools::Itertools;

// Local module imports
use crate::Alternative;

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
    let mut expand_families = false;

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
        } else if arg == "--expand-families" {
            expand_families = true;
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

    
    if expand_families {
        let mut xpanded = Vec::<Alternative>::new();
        // A HashSet to keep track of strings we've already added
        let mut seen = std::collections::HashSet::new();

        for alt in cfg.alternatives.iter() {
            let (raw, fam): (&String, &Option<String>) = (&alt.raw, &alt.fam);
            
            // Split into owned strings so we can manipulate and reuse them
            let parts: Vec<String> = raw.split(&[' ', '-'][..])
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
            let titlecase_version = parts.iter()
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
            let all_caps_version = parts.iter()
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
        
        // dynamic assignment or update back to cfg goes here
        for x in xpanded {
            // stick the ones we made into cfg.alternatives
            cfg.alternatives.push(Alternative {
                raw: x.raw,
                jfmt: x.jfmt,
                fam: x.fam,
            });
        }
    }

    //     if expand_families {
    //     let mut xpanded = Vec::new();
    //     let mut seen = std::collections::HashSet::new();

    //     let mut push_expanded = |variant: String, fam: Option<String>, debug: bool, tag: &str| {
    //         if seen.insert(variant.clone()) {
    //             if debug {
    //                 println!("🧩 {} {}", tag, variant);
    //             }
    //             xpanded.push(Alternative {
    //                 raw: variant.clone(),
    //                 jfmt: variant,
    //                 fam,
    //             });
    //         }
    //     };

    //     for alt in &cfg.alternatives {
    //         if alt.raw.is_empty() {
    //             continue;
    //         }

    //         // Split into structural blocks of alphanumeric text vs non-alphanumeric separators
    //         let mut words_and_delims = Vec::new();
    //         let mut current = String::new();
    //         let mut is_alphanumeric = true;

    //         for c in alt.raw.chars() {
    //             if c.is_alphanumeric() == is_alphanumeric {
    //                 current.push(c);
    //             } else {
    //                 if !current.is_empty() {
    //                     words_and_delims.push((current.clone(), is_alphanumeric));
    //                 }
    //                 current.clear();
    //                 current.push(c);
    //                 is_alphanumeric = !is_alphanumeric;
    //             }
    //         }
    //         if !current.is_empty() {
    //             words_and_delims.push((current, is_alphanumeric));
    //         }

    //         // Helper to titlecase a single string chunk safely
    //         let to_titlecase = |s: &str| {
    //             let mut chars = s.chars();
    //             match chars.next() {
    //                 None => String::new(),
    //                 Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    //             }
    //         };

    //         // --- 1. Sentence-capitalized version ---
    //         let mut is_first_word = true;
    //         let sentence_version: String = words_and_delims.iter().map(|(chunk, is_word)| {
    //             if *is_word && is_first_word {
    //                 is_first_word = false;
    //                 to_titlecase(chunk)
    //             } else {
    //                 chunk.clone()
    //             }
    //         }).collect();
    //         push_expanded(sentence_version, alt.fam.clone(), cfg.debug, "SC");

    //         // --- 2. Each word in Titlecase version ---
    //         let titlecase_version: String = words_and_delims.iter().map(|(chunk, is_word)| {
    //             if *is_word { to_titlecase(chunk) } else { chunk.clone() }
    //         }).collect();
    //         push_expanded(titlecase_version, alt.fam.clone(), cfg.debug, "TC");

    //         // --- 3. Each word in ALL CAPS version ---
    //         let all_caps_version: String = words_and_delims.iter().map(|(chunk, is_word)| {
    //             if *is_word { chunk.to_uppercase() } else { chunk.clone() }
    //         }).collect();
    //         push_expanded(all_caps_version, alt.fam.clone(), cfg.debug, "AC");
    //     }
        
    //     cfg.alternatives.extend(xpanded);
    // }


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
