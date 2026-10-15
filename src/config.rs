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
    pub expand_families: bool,
}

impl Cfg {
    pub fn add_alternative(&mut self, text: &str, family: Option<String>) {
        if text.contains('-') {
            // Hyphenated phrases: replace hyphens with " - " for Ngram JSON API compatibility
            // NOTE: See also the special handling for apostrophes in google_ngrams.rs/build_url()
            self.alternatives.push(Alternative {
                raw: text.to_string(),
                jfmt: text.split('-').join(" - "),
                fam: family,
            });
        } else if text.contains('\'') {
            // Special handling when any word starts or ends with an apostrophe, such as 'tis or 'nother
            // NOTE: See also the special handling for apostrophes in google_ngrams.rs/build_url()
            let jfmt = text
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
            self.alternatives.push(Alternative {
                raw: jfmt.clone(),
                jfmt,
                fam: family,
            });
        } else {
            // Simple words: use as-is
            if self.debug {
                if let Some(ref fam) = family {
                    println!("👉Word: {}::{}", fam, text);
                } else {
                    println!("👉Word: {}", text);
                }
            }
            self.alternatives.push(Alternative {
                raw: text.to_string(),
                jfmt: text.to_string(),
                fam: family,
            });
        }
    }
}

// ============================================================================
// COMMAND-LINE INTERFACE
// ============================================================================

/// Parses command-line arguments into a configuration struct
///
/// Supported arguments:
/// - `--raw`: Enable raw diagnostic output
/// - `-f <first_alt>`, `--fam <first_alt>`, `--family <first_alt>`: Start family and include first alternative
/// - `-f=<name>`, `--fam=<name>`, `--family=<name>`: Set symbolic family label for subsequent alternatives
pub fn cli() -> Result<Cfg, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut cfg = Cfg {
        debug: false,
        raw: false,
        alternatives: Vec::new(),
        has_families: false,
        since_year: None,
        expand_families: false,
    };

    let mut family = None;
    let mut expect_family_head = false;

    for arg in args {
        if expect_family_head {
            if arg.starts_with('-') {
                return Err(format!(
                    "Expected alternative following -f / --family, found option '{}'",
                    arg
                )
                .into());
            }
            family = Some(arg.clone());
            if cfg.debug {
                println!("👉Family: {}", arg);
            }
            cfg.add_alternative(&arg, family.clone());
            expect_family_head = false;
            continue;
        }

        if arg == "--raw" {
            cfg.raw = true;
        } else if ["--debug", "-d"].contains(&arg.as_str()) {
            cfg.debug = true;
        } else if ["--family", "--fam", "-f"].contains(&arg.as_str()) {
            expect_family_head = true;
            cfg.has_families = true;
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
            cfg.expand_families = true;
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: {}", arg).into());
        } else {
            cfg.add_alternative(&arg, family.clone());
        }
    }

    if expect_family_head {
        return Err("Expected alternative following -f / --family flag at end of arguments".into());
    }


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
