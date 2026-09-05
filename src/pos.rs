use harper_core::DictWordMetadata;

pub type PosPredicate = fn(&DictWordMetadata) -> bool;

/// Result of a part-of-speech lookup operation
///
/// Explicitly distinguishes between words not in the dictionary (true OOV)
/// and words that are in the dictionary but match no POS predicates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PosLookupResult {
    /// Word not found in the dictionary (true out-of-vocabulary)
    NotFound,
    /// Word found in dictionary but matches no POS predicates
    FoundWithNoMatches,
    /// Word found in dictionary with matching POS tags
    FoundWithMatches(Vec<&'static Pos>),
}

impl PosLookupResult {
    /// Returns the matched POS tags if any, regardless of lookup status
    ///
    /// - `NotFound` → empty Vec
    /// - `FoundWithNoMatches` → empty Vec  
    /// - `FoundWithMatches` → the Vec of matches
    pub fn _into_poses(self) -> Vec<&'static Pos> {
        match self {
            PosLookupResult::NotFound => Vec::new(),
            PosLookupResult::FoundWithNoMatches => Vec::new(),
            PosLookupResult::FoundWithMatches(poses) => poses,
        }
    }

    /// Returns true if the word was found in the dictionary (regardless of matches)
    pub fn _is_found(&self) -> bool {
        !matches!(self, PosLookupResult::NotFound)
    }

    /// Returns true if the word was not found in the dictionary (true OOV)
    pub fn _is_not_found(&self) -> bool {
        matches!(self, PosLookupResult::NotFound)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub enum Pos {
    Adjective,
    Adverb,
    Conjunction,
    Determiner,
    Noun,
    Preposition,
    Pronoun,
    ProperNoun,
    Verb,
}

#[allow(dead_code)]
pub struct POSInfo {
    pub _name: &'static str,
    pub ord: usize,
    pub _ptb: &'static str, // Penn Treebank
    pub letter: &'static str,
    pub emoji: &'static str,
    pub _gng: &'static str, // Google Ngram Viewer
}

pub const POS_DEFINITIONS: &[(Pos, PosPredicate)] = &[
    /* #1 */ (Pos::Determiner, DictWordMetadata::is_determiner),
    /* #2 */ (Pos::Noun, |m| m.is_noun() && !m.is_proper_noun()),
    /* #2 */ (Pos::ProperNoun, DictWordMetadata::is_proper_noun),
    /* #3 */ (Pos::Pronoun, DictWordMetadata::is_pronoun),
    /* #4 */ (Pos::Verb, DictWordMetadata::is_verb),
    /* #5 */ (Pos::Adjective, DictWordMetadata::is_adjective),
    /* #6 */ (Pos::Adverb, DictWordMetadata::is_adverb),
    /* #7 */ (Pos::Preposition, |m| m.preposition),
    /* #8 */ (Pos::Conjunction, DictWordMetadata::is_conjunction),
];

pub fn pos_info(pos: &Pos) -> POSInfo {
    match pos {
        Pos::Noun => POSInfo {
            letter: "N",
            ord: 2,
            _ptb: "NN",
            emoji: "📦",
            _name: "noun",
            _gng: "_NOUN_",
        },
        Pos::ProperNoun => POSInfo {
            letter: "O",
            ord: 2,
            _ptb: "NNP",
            emoji: "📛",
            _name: "proper noun",
            _gng: "_PROPN_",
        },
        Pos::Verb => POSInfo {
            letter: "V",
            ord: 4,
            _ptb: "VB",
            emoji: "🏃",
            _name: "verb",
            _gng: "_VERB_",
        },
        Pos::Adjective => POSInfo {
            letter: "J",
            ord: 5,
            _ptb: "JJ",
            emoji: "🌈",
            _name: "adjective",
            _gng: "_ADJ_",
        },
        Pos::Adverb => POSInfo {
            letter: "R",
            ord: 6,
            _ptb: "RB",
            emoji: "🤷",
            _name: "adverb",
            _gng: "_ADV_",
        },
        Pos::Conjunction => POSInfo {
            letter: "C",
            ord: 8,
            _ptb: "CC",
            emoji: "🔗",
            _name: "conjunction",
            _gng: "_CONJ_",
        },
        Pos::Determiner => POSInfo {
            letter: "D",
            ord: 1,
            _ptb: "DT",
            emoji: "👉",
            _name: "determiner",
            _gng: "_DET_",
        },
        Pos::Preposition => POSInfo {
            letter: "P",
            ord: 7,
            _ptb: "IN",
            emoji: "📥",
            _name: "preposition",
            _gng: "_ADP_",
        },
        Pos::Pronoun => POSInfo {
            letter: "I",
            ord: 3,
            _ptb: "PRP",
            emoji: "👤",
            _name: "pronoun",
            _gng: "_PRON_",
        },
    }
}
