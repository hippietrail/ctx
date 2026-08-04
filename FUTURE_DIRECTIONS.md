# Future Directions for Confusable Word Analyzer

## Current Limitations

### Inflected Words
The current analyzer treats each distinct string as a separate candidate, which doesn't work well for morphological variants:

**Example: contend-vs-contest.txt**
- Contains 6 different forms: contended, contested, contends, contests, contending, contesting
- Current approach treats these as separate alternatives
- But all 6 forms are actually variants of the same 2 base words

**Needed improvements:**
- Lemmatization or stemming to group morphological variants
- Could use libraries like `lemmatizer` or integrate with NLP tools
- Consider part-of-speech tagging to handle different word classes

### Variable Set Sizes ~~PARTIALLY IMPLEMENTED~~
The current approach supports multiple confusable terms via family grouping:

**Examples:**
- `contend-vs-contest.txt`: 6 variants (contended, contested, contends, contests, contending, contesting)
- `wide-ranging-vs-wide-sweeping.txt`: 4 variants (wide sweeping, wide - sweeping, wide ranging, wide - ranging)

**Current implementation:**
- `--family=<name>` flag enables manual family grouping
- Families allow comparing groups of related alternatives (e.g., case variants, compound forms)
- User can manually specify families via command line arguments
- Provides family-level uniqueness analysis

**Previous clustering approach (deprecated):**
- Auto-clustering code exists in `srcold/clustering.rs` but is not currently used
- Frequency-based clustering using rolling baseline divergence threshold
- Automatically determined core cluster size from frequency distribution

**Future improvements:**
- Similarity-based clustering (Levenshtein, Jaccard, embedding-based)
- Hierarchical clustering to discover groups
- Configurable threshold for grouping
- Re-enable or improve auto-clustering

### Hyphenation and Punctuation
**Example: wide-ranging-vs-wide-sweeping.txt**
- "wide sweeping" vs "wide - sweeping" treated as separate
- Hyphenation patterns vary in corpora

**Needed improvements:**
- Normalization of punctuation/spacing
- Configurable tokenization rules
- Handle en-dashes, em-dashes, and other punctuation

### Case Sensitivity Analysis ~~IMPLEMENTED~~
**Example: trouble-vs-troubles.txt**
- 22 case-sensitive contexts (only appear in one case form)
- 1 case-insensitive context: "the" appears as both "the" and "The"
- This suggests most context words are consistently cased in the corpus

**Example: border-vs-boarder.txt**
- 23 case-sensitive contexts
- 0 case-insensitive contexts
- All context words appear in consistent case

**Current implementation:**
- Track case variations for each context word
- Classify contexts as:
  - **Case-sensitive**: Only one case form observed (e.g., "Mexican", "posterior")
  - **Case-insensitive**: Multiple case forms observed (e.g., "the"/"The")
- POS analysis uses case-insensitive contexts (since Harper's dictionary is case-folded)
- This data-driven approach avoids over-generalization while handling natural case variation
- DAGGER (†) symbol marks case-sensitive contexts in output

### Multi-word Phrases ~~IMPLEMENTED~~
**Example: wide-ranging vs wide-sweeping**
- Both are 2-word phrases
- Current implementation supports multi-word confusable terms (cores)
- Contexts are assumed to be single words for POS dictionary lookup

**Current status:**
- Multi-word cores are supported (e.g., "search for" vs "search")
- Context analysis limited to single-word contexts for dictionary compatibility
- Special handling for hyphens and apostrophes in multi-word phrases

## Proposed Architecture

### 1. Preprocessing Pipeline
```
Raw text → Normalization → Tokenization → Lemmatization → Feature extraction
```

**Current status**: Partially implemented
- Input normalization for hyphens and apostrophes
- Multi-word phrase support
- Basic tokenization via Google Ngrams API

### 2. Clustering Stage
- **Current**: Manual family grouping via CLI flags
- **Deprecated**: Frequency-based clustering with rolling baseline divergence (exists in srcold/)
- **Future**: Similarity metrics (Levenshtein, Jaccard, embedding-based)
- **Future**: Hierarchical clustering to discover groups
- **Future**: Configurable threshold for grouping

### 3. Context Analysis
- **Current**: Single-word context extraction with POS tagging via Harper
- **Current**: Case sensitivity analysis
- **Current**: Year filtering for temporal analysis
- **Future**: Expand context window beyond immediate neighbors
- **Future**: Weight contexts by frequency and discriminative power
- **Future**: Statistical significance testing

### 4. POS Analysis Enhancement
- **Current**: Basic POS categorization (Noun, Verb, Adjective, etc.)
- **Current**: POS discrimination analysis (unique POS per alternative)
- **Future**: Drill down into POS sub-properties:
  - Determiners: kinds (definite/indefinite, demonstrative, etc.)
  - Pronouns: person (1st/2nd/3rd), number (singular/plural), case (subject/object)
  - Verbs: tense, aspect, mood, voice inflection patterns
  - Adjectives: comparative/superlative forms
  - Adverbs: degree and manner classifications
- **Future**: Combined analysis views:
  - Group by all POSes of each context word
  - Group by all words for each POS category
  - Cross-tabulation of words × POS matrix

### 5. Output Generation
- **Current**: Color-coded confusable contexts with POS associations
- **Current**: Raw diagnostic output with POS tags
- **Current**: Prohibited context detection
- **Current**: Family-level uniqueness view
- **Future**: Grouped results with confidence scores
- **Future**: Contextual examples for each group
- **Future**: Statistical summaries
- **Future**: Export to structured formats (JSON, CSV)

## Data Quality Considerations

### Input Format ~~IMPLEMENTED~~
- **Current format**: Command-line arguments with confusable terms/phrases
- **Current features**:
  - Multi-word phrase support
  - Special character handling (hyphens, apostrophes)
  - Family grouping for related alternatives
- **Could support**:
  - File-based input (one phrase per line)
  - Sentence-level input with automatic extraction
  - Frequency counts alongside phrases
  - Metadata (source, date, domain)

### Validation
- Manual verification of confusable pairs
- Cross-validation with dictionaries
- False positive/negative analysis
