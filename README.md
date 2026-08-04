## **ctx**

(formerly "cngd", for "contextual n-gram differencer")

## Purpose

This tool analyzes commonly confused English words or phrases (e.g., "affect/effect", "their/there/they're") to identify discriminative patterns that help developers write grammar checker rules.

The tool provides information to developers, not automated rule generation. For example, it might reveal: "If 'rightly' comes after a word with POS X or before specific word Y, it's probably a mistake for 'rightfully'."

The analysis works in multiple stages:
1. **Word-level discrimination**: Find context words that uniquely identify one confusable term but not others
2. **POS-level discrimination**: Find part-of-speech patterns that uniquely identify one confusable term but not others
3. **Negative context detection**: For >2 alternatives, identify contexts that appear with all other alternatives but not the current one
4. **Family-level analysis**: When families are enabled, compare within families and provide family-level uniqueness

Developers can use this information to write Harper linter rules that flag potential misuse of confusable terms based on their linguistic context.

## Features

- Fetches context words from Google Ngram Viewer (before/after each alternative)
- Uses part-of-speech tagging to categorize context words
- Identifies unique contexts (🟢) - words that appear with only one alternative
- Identifies prohibited contexts (🚫) - words that appear with all other alternatives
- Supports family grouping for comparing related alternatives (e.g., case variants, compound forms)
- Case sensitivity analysis to distinguish context words that are unique only with case consideration
- Raw diagnostic output with POS tags for debugging
- Optional year filtering to analyze language usage from specific time periods

## How It Works

1. **API Query**: Constructs a Google NGrams JSON API query using the `*` wildcard before and after each confusable term to capture context words
2. **Data Fetching**: Uses `reqwest` to fetch JSON data directly from Google NGrams
3. **JSON Parsing**: Parses the NGrams response, extracting EXPANSION-type entries to identify context words
4. **Context Extraction**: For each confusable term, extracts pre-contexts (words before) and post-contexts (words after) from the ngram data
5. **Context Mapping**: Builds maps from context words to the variants they appear with, and from POS tags to variants
6. **Discrimination Analysis**: Finds context words and POS tags that appear with exactly ONE confusable term (these are the discriminators)
7. **Negative Context Detection**: For >2 alternatives, identifies contexts that appear with all-but-one variant (useful for exclusion rules)
8. **Family Grouping**: When families are enabled, compares alternatives within families and also provides family-level uniqueness analysis
9. **Output**: Color-coded results showing which context words and POS tags can distinguish between confusable terms

## Language Choice

**Rust** chosen because:
- Harper integration (Harper is written in Rust)
- Uses Harper's lexical POS tagging (the same POS information available to linters)

## Getting Data from Google Ngrams

The tool now fetches data directly from Google NGrams' JSON endpoint. The query is automatically constructed using the `*` wildcard before and after each confusable term to capture context words.

Example query constructed internally: `* they ' re,they ' re *,* their,their *,* there,there *`

Note: Google NGrams has limitations on the number of alternatives per query and only allows one `*` per query, which is why the tool makes separate queries for pre-context and post-context. `*` can only match one word, hence the tool only considers one word of context per side.

### Special Input Handling

The tool automatically handles special characters in input:

- **Hyphens**: Hyphenated phrases are converted to use spaces (e.g., "wide-ranging" becomes "wide - ranging") for API compatibility
- **Apostrophes**: Words starting or ending with apostrophes (e.g., "'tis", "'nother") are formatted with space-separated apostrophes (e.g., "' tis", "nother '") for proper API query construction
- **Apostrophes in general**: Words containing apostrophes (e.g., "don't") are wrapped in brackets (e.g., "[* don't]") for the API query

## Usage

```bash
# Basic usage - specify confusable terms as arguments
cargo run --release there their "they ' re"

# Multi-word phrases (use quotes)
cargo run --release "shopping center" "shopping centre" "shopping mall" mall

# Raw diagnostic output with POS tags
cargo run --release --raw foo bar

# Filter data from a specific year onwards
cargo run --release --since=1968 foo bar

# Family grouping for comparing related alternatives
cargo run --release -f=foo foos foo bar

# Build and run
cargo build --release
cargo run --release -- term1 term2 term3
```

## Command Line Arguments

- `[terms...]` - The confusable terms or phrases to analyze (space-separated, use quotes for multi-word phrases)
- `--raw` - Print raw diagnostic output with all context words and POS tags
- `--debug`, `-d` - Enable debug mode (shows query content)
- `--family=<name>`, `--fam=<name>`, `-f=<name>` - Set family for subsequent alternatives (for comparing groups of related terms)
- `--since=<year>`, `--since-year=<year>` - Filter data from a specific year onwards (must be exactly 4 digits)

## Output

The tool outputs color-coded results for each confusable term:

```
🟢 [pre-POS-tags] ¦ [pre-context-words] «« term »» [post-context-words] ¦ [post-POS-tags]
🚫 [negative-pre-POS-tags] | [negative-pre-contexts] | term | [negative-post-contexts] | [negative-post-POS-tags]
```

- **POS tags** (single letters): D=determiner, N=noun, I=pronoun, O=proper noun, V=verb, J=adjective, R=adverb, P=preposition, C=conjunction
- **Context words**: Words that uniquely identify this term (appear with this term but not others)
- **Negative contexts** (🚫): When analyzing >2 alternatives, contexts that appear with all-but-this-one variant
- **Case-sensitive markers**: The DAGGER (†) symbol indicates words that are unique only when considering case sensitivity (e.g., "The" vs "the")
- **Family grouping**: When families are enabled, results show family-level uniqueness after individual term results

This information can be used to create grammar checker rules that help determine when a confusable word is used correctly or mistakenly.

----

I initially coded this by hand, but with help from the AI assistant built into Devin, the code editor formerly known as Windsurf, and from Google Search's AI.

Once I had it working as I wanted, I got Devin to refactor it to be more idiomatic Rust and then add some trivial features.  
Since then I modified it both with hand-coding and using a couple of free coding AIs, mostly for suggestions, but sometimes to directly modify the code.  
Like many vibe-coded tools, the code got harder to understand and modify, but the basic steps became clearer to me.  
Then I discovered that Google Ngrams JSON endpoint and rewrote it from scratch by hand. Once more some AI-generated suggestions have since been integrated.  
