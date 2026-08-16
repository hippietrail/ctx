## **ctx**

(formerly "cngd", for "contextual n-gram differencer")

## Purpose

This tool analyzes commonly confused English words or phrases (e.g., "affect/effect", "their/there/they're") to identify discriminative patterns that help developers write grammar checker rules.

The tool provides information to developers, not automated rule generation. For example, it might reveal: "If 'rightly' comes after a word with POS X or before specific word Y, it's probably a mistake for 'rightfully'."

The analysis works by:
1. **Data Fetching**: Fetches context words from Google Ngram Viewer for each alternative
2. **Hierarchical Organization**: Organizes data into a FamilyTree structure (Family → Alternative → Side → Context → POS tags)
3. **Set Operations**: Compares two families using set operations (union, intersection, difference) to identify:
   - Shared contexts (bold) - words that appear with both families
   - Unique contexts (color-coded) - words that appear with only one family
   - POS patterns that distinguish between families

Developers can use this information to write Harper linter rules that flag potential misuse of confusable terms based on their linguistic context.

## Features

- Fetches context words from Google Ngram Viewer (before/after each alternative)
- Uses part-of-speech tagging to categorize context words via Harper's dictionary
- Compares two families of alternatives using set operations
- Color-coded output showing shared vs unique contexts and POS patterns
- Supports family grouping for comparing related alternatives (e.g., case variants, compound forms)
- Optional year filtering to analyze language usage from specific time periods

## How It Works

1. **API Query**: Constructs a Google NGrams JSON API query using the `*` wildcard before and after each confusable term to capture context words
2. **Data Fetching**: Uses `reqwest` to fetch JSON data directly from Google NGrams
3. **JSON Parsing**: Parses the NGrams response, extracting EXPANSION-type entries to identify context words
4. **Context Extraction**: For each confusable term, extracts pre-contexts (words before) and post-contexts (words after) from the ngram data
5. **Hierarchical Organization**: Builds a FamilyTree structure: Family → Alternative → Side (Before/After) → Context → POS tags
6. **Set Operations**: Compares two families using union, intersection, and difference operations to identify shared vs unique contexts
7. **POS Analysis**: Categorizes context words by part-of-speech using Harper's dictionary
8. **Output**: Color-coded results showing which context words and POS tags are shared (bold) or unique (colored) to each family

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
- `--debug`, `-d` - Enable debug mode (shows query content and family assignments)
- `--family=<name>`, `--fam=<name>`, `-f=<name>` - Set family for subsequent alternatives (for comparing groups of related terms)
- `--since=<year>`, `--since-year=<year>` - Filter data from a specific year onwards (must be exactly 4 digits)

Note: The current implementation requires exactly 2 families for comparison. If no families are specified, each alternative becomes its own family.

## Output

The tool outputs color-coded results comparing two families of alternatives:

```
=== LEFT WORDS ===
  [shared words in bold], [family A unique words in yellow], [family B unique words in red]
=== LEFT POS ===
  [shared POS in bold], [family A unique POS in yellow], [family B unique POS in red]
=== RIGHT WORDS ===
  [shared words in bold], [family A unique words in yellow], [family B unique words in red]
=== RIGHT POS ===
  [shared POS in bold], [family A unique POS in yellow], [family B unique POS in red]
```

- **Bold text**: Items shared by both families
- **Yellow text**: Items unique to the first family
- **Red text**: Items unique to the second family
- **POS tags**: Full enum names (e.g., Noun, Verb, Adjective) from Harper's dictionary
- **Family grouping**: Enables comparison of groups of related alternatives

This information can be used to create grammar checker rules that help determine when a confusable word is used correctly or mistakenly.

----

I initially coded this by hand, but with help from the AI assistant built into Devin, the code editor formerly known as Windsurf, and from Google Search's AI.

Once I had it working as I wanted, I got Devin to refactor it to be more idiomatic Rust and then add some trivial features.  
Since then I modified it both with hand-coding and using a couple of free coding AIs, mostly for suggestions, but sometimes to directly modify the code.  
Like many vibe-coded tools, the code got harder to understand and modify, but the basic steps became clearer to me.  
Then I discovered that Google Ngrams JSON endpoint and rewrote it from scratch by hand. Once more some AI-generated suggestions have since been integrated.  
