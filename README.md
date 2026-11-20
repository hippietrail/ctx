## **ctx**

(formerly "cngd", for "contextual n-gram differencer")

## Purpose

This tool analyzes commonly confused English words or phrases (e.g., `"affect"` / `"effect"`, `"point out some"` / `"point some"`) to identify discriminative patterns that help developers write grammar checker rules.

The tool provides empirical corpus data to developers rather than automated rule generation. For example, it might reveal: *"If 'point' is followed by 'some' and a noun like 'problems' or 'errors', it's almost certainly missing 'out'."*

The analysis works by:
1. **Data Fetching**: Queries Google Ngrams' JSON API for wildcard expansions before and after each alternative.
2. **Hierarchical Organization**: Organizes data into a `FamilyTree` structure (`Family → Alternative → Side → Context → (POS tags, frequency)`).
3. **Set Operations & Frequency Weighting**: Compares two (or three) families using set operations (`union`, `intersection`, `difference`) and aggregates `timeseries` relative frequencies to highlight:
   - **Unique contexts** (color-coded by family) — words and POS tags that appear with only one family.
   - **Shared contexts** (blue bracketed comparisons) — words and POS tags that appear with both families, ranked and weighted by relative frequency.

Developers can use this information to write [Harper](https://github.com/hippietrail/harper) linter rules that flag potential misuse of confusable terms based on their surrounding context.

## Features

- Fetches pre- and post-context wildcard expansions from Google Ngrams' JSON endpoint in a single combined request
- Computes mean relative frequency weights from Google Ngrams' `timeseries` data across the queried year range
- Categorizes context words by part of speech using `harper-core`'s lexical dictionary
- Compares 2 or 3 families of alternatives using set operations
- Color-coded terminal output with relative frequency superscripts (e.g., `to¹⁰⁰`, `and³⁰˙¹²`)
- Supports family grouping (`-f`) and automatic case-variant expansion (`--expand-families`)
- Optional start-year filtering (`--since`) to focus on modern or period-specific usage

## How It Works

1. **API Query**: Builds a single Google Ngrams JSON API URL combining `* {alt}` and `{alt} *` wildcard patterns for every alternative.
2. **Data Fetching**: Uses `reqwest` (blocking) to fetch the JSON payload directly from Google Ngrams.
3. **JSON Parsing**: Filters `EXPANSION` entries, isolates the preceding or following context word, and averages the `timeseries` array into a relative frequency score.
4. **POS Lookup**: Looks up each context word in `harper-core`'s `FstDictionary` to attach matching POS tags (and flags out-of-vocabulary tokens).
5. **Hierarchical Organization**: Inserts rows into `FamilyTree`: `Family → Alternative → Side (Before/After) → Context → ContextEntry { poses, freq }`.
6. **Comparison & Output**: Computes set differences and intersections across families, normalizes relative frequencies on a `0–100` scale within each comparison group, and prints color-coded results.

## Language Choice

**Rust** was chosen for direct integration with `harper-core`, ensuring that `ctx` uses the exact same dictionary and lexical POS metadata available to Harper's linters.

## Getting Data from Google Ngrams

The tool queries Google Ngrams' JSON endpoint (`https://google.com`) with a comma-separated `content` parameter containing both `* {alt}` (pre-context) and `{alt} *` (post-context) for each alternative. Because `*` matches a single token in Google Ngrams, `ctx` analyzes one context word immediately before and one immediately after each target phrase.

Example `content` query constructed internally:
`* they ' re,they ' re *,* their,their *,* there,there *`

### Special Input Handling

The tool automatically formats special characters for Google Ngrams API compatibility:

- **Hyphens**: Hyphenated phrases are spaced around the hyphen (e.g., `"wide-ranging"` becomes `"wide - ranging"`).
- **Leading/trailing apostrophes**: Clitics like `"'tis"` or `"'nother"` are formatted with space-separated apostrophes (e.g., `"' tis"`, `"nother '"`).
- **Internal apostrophes**: Words containing apostrophes (e.g., `"don't"`) are wrapped in square brackets (e.g., `"[* don't]"`).

## Usage

```bash
# Basic 2-way comparison
cargo run --release -- affect effect

# 3-way comparison
cargo run --release -- there their "they ' re"

# Filter data from a specific year onwards
cargo run --release -- --since=1968 "point out some" "point some"

# Group related forms into families (bare -f creates a family and includes that term)
cargo run --release -- -f foo foos -f bar bars

# Group multi-word phrases into 2 families
cargo run --release -- -f "shopping center" "shopping centre" -f "shopping mall" mall

# Symbolic family names (groups alternatives without querying the family label itself)
cargo run --release -- -f=US color theater -f=UK colour theatre

# Expand case variants automatically and show shared context words
cargo run --release -- --since=1962 --expand-families --common-words --no-common-pos -f "point out some" -f "point some"
```

## Command Line Arguments

- `[terms...]` — Confusable terms or phrases to analyze (use quotes for multi-word phrases).
- `--family <term>`, `--fam <term>`, `-f <term>` — Start a family named `<term>` and include `<term>` as its first alternative.
- `--family=<name>`, `--fam=<name>`, `-f=<name>` — Start a family with symbolic label `<name>` without adding `<name>` as a search term.
- `--expand-families` — Automatically generate case variants (lowercase, Title Case, ALL CAPS) for each alternative.
- `--common-words`, `--no-common-words` — Show or hide shared context words across families (default: hide).
- `--common-pos`, `--no-common-pos` — Show or hide shared POS categories across families (default: show).
- `--since=<year>`, `--since-year=<year>` — Filter data from a specific 4-digit start year onwards.
- `--debug`, `-d` — Enable debug output.

> **Note:** Analysis requires **2 or 3 families**. If no `-f` flags are provided, each term is placed in its own individual family.

## Output

In 2-way comparison mode, `ctx` prints `Before` and `After` sections for both `WORDS` and `POS`:

```text
“family A” vs. “family B”
=== Before WORDS ===
 Before family A: [words unique to A in orange]
 Before family B: [words unique to B in red]
 Before common : [words with freq¹⁰⁰ : family A] vs [words with freq¹²˙⁵ : family B]
=== Before POS ===
 🌈J : [top words unique to A in orange] : family A
 📥P : [words : family A] vs [words : family B]   (shared POS in blue, when --common-pos is enabled)
 👉D : [top words unique to B in red] : family B
=== After WORDS ===
 After family A: [words unique to A in orange]
 After family B: [words unique to B in red]
 After common : [family A : words with freq¹⁰⁰] vs [family B : words with freq⁷˙³⁵]
=== After POS ===
 ...
```

- **Colors**: Family A is **orange**, Family B is **red**, and shared comparisons are **blue**.
- **Out-of-vocabulary (OOV) tokens**: Words not found in Harper's dictionary are dimmed and wrapped in guillemets (e.g., `«ap-»`).
- **POS tags**: Displayed as emoji + single-letter codes (e.g., `📦N` Noun, `🏃V` Verb, `🌈J` Adjective, `🤷R` Adverb, `📥P` Preposition, `👉D` Determiner, `🔗C` Conjunction, `📛O` Pronoun).
- **Frequency superscripts**: Relative frequencies derived from Google Ngrams `timeseries` data, normalized on a `0–100` scale relative to the strongest collocate in each comparison group (with trailing zeros trimmed).

----

## History

I initially coded this by hand, with help from the AI assistant built into Devin (formerly Windsurf) and from Google Search's AI.

Once I had it working as I wanted, I got Devin to refactor it to be more idiomatic Rust and add a few small features. Since then I've modified it through a mix of hand-coding and AI-assisted suggestions. Like many vibe-coded tools, the code eventually grew harder to understand and modify, though the core pipeline became clearer to me. When I discovered Google Ngrams' JSON endpoint, I rewrote it from scratch by hand, integrating selective AI-assisted refinements since then.
