// ============================================================================
// MODULES AND IMPORTS
// ============================================================================

// Local modules

// Standard library

// External crates
use serde::Deserialize;

// Local modules
use crate::Cfg;

/// Represents a single item from the Google Ngrams JSON API response
/// - `ngram`: The full ngram string (e.g., "* stake")
/// - `parent`: The parent ngram this expands from
/// - `kind`: The type of ngram (Collection, Expansion, or individual Ngram)
/// - `_timeseries`: The frequency data over time (currently unused)
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct NgramItem {
    ngram: String,
    parent: String,
    #[serde(rename = "type")]
    kind: NgramType,
    #[serde(rename = "timeseries")]
    _timeseries: serde_json::Value,
}

/// Types of ngrams returned by the API
/// - `NgramCollection`: A collection of related ngrams
/// - `Expansion`: A wildcard expansion (e.g., "* stake" expanding to "the stake", etc.)
/// - `Ngram`: A specific ngram without wildcards
#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum NgramType {
    NgramCollection,
    Expansion,
    Ngram,
}

/// Represents which side of the target word a context appears on
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum Side {
    Before,
    After,
}
use Side::*;

/// A parsed row from the ngram data
/// - `side`: Whether this context appears before or after the target
/// - `alt`: The alternative word this context belongs to
/// - `ctx`: The context word itself
pub struct Row<'a> {
    pub side: Side,
    pub alt: &'a str,
    pub ctx: &'a str,
}

// ============================================================================
// URL BUILDING AND API FETCHING
// ============================================================================

/// Builds the Google Ngrams API URL for fetching JSON data
///
/// Constructs a query that requests wildcard expansions for each alternative.
/// For each alternative, it creates two patterns:
/// - "* {alt}" to find words before the alternative
/// - "{alt} *" to find words after the alternative
///
/// The L/R brackets handle apostrophes in words (e.g., "don't" becomes "[* don't]")
/// NOTE: See also the special handling for hyphens in main.rs/cli()
///
/// TODO: Consider adding smoothing parameter
/// NOTE: Case-insensitive mode cannot be used in combination with wildcards
pub fn build_url(cfg: &Cfg) -> url::Url {
    let mut url = url::Url::parse("https://books.google.com/ngrams/json").unwrap();

    const L: &str = "[";
    const R: &str = "]";

    let content = cfg
        .alternatives
        .iter()
        .map(|t| {
            let i = t.raw.contains(['\'', '’']) as usize;
            let (l, r) = (&L[..i], &R[..i]);
            format!("{l}* {}{r},{l}{} *{r}", t.raw, t.raw)
        })
        .collect::<Vec<_>>()
        .join(",");

    if cfg.debug {
        eprintln!("👉 ‘{content}’");
    }

    url.query_pairs_mut().append_pair("content", &content);

    if let Some(year) = cfg.since_year {
        url.query_pairs_mut()
            .append_pair("year_start", &year.iter().collect::<String>());
    }

    url
}

/// Fetches JSON data from the given URL
///
/// TODO: Add timeout configuration
/// TODO: Add retry logic for network failures
/// TODO: Consider using async reqwest for better performance
pub fn fetch_json(url: url::Url) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let json = response.json::<serde_json::Value>()?;
    Ok(json)
}

// ============================================================================
// DATA PARSING
// ============================================================================

/// Parses NgramItem data into a structured table of context rows
///
/// Filters for Expansion type ngrams and extracts the context words
/// that appear before or after each alternative.
///
/// Returns a vector of Row structs containing:
/// - The side (Before/After)
/// - The alternative word
/// - The context word
///
/// TODO: Add better error messages for parsing failures
/// TODO: Consider handling edge cases like empty ngrams
pub fn parse_items<'a>(items: &'a [NgramItem]) -> Result<Vec<Row<'a>>, Box<dyn std::error::Error>> {
    let mut table = Vec::<Row>::new();

    for item in items {
        if item.kind != NgramType::Expansion {
            continue;
        }
        if item.ngram.starts_with("* ") || item.ngram.ends_with(" *") {
            continue;
        }

        let (alternative, is_prefix) = match item.parent.strip_prefix("* ") {
            Some(alt) => (alt, true),
            None => (
                item.parent
                    .strip_suffix(" *")
                    .ok_or("No wildcard found in parent")?,
                false,
            ),
        };

        let context = match is_prefix {
            true => &item.ngram[..item.ngram.len() - (alternative.len() + 1)],
            false => &item.ngram[alternative.len() + 1..],
        };

        table.push(Row {
            side: match is_prefix {
                true => Before,
                false => After,
            },
            alt: alternative,
            ctx: context,
        });
    }

    Ok(table)
}
