use harper_core::DictWordMetadata;

pub type PosPredicate = fn(&DictWordMetadata) -> bool;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

pub struct POSInfo {
    pub _name: &'static str,
    pub ord: usize,
    pub _ptb: &'static str, // Penn Treebank
    pub letter: &'static str,
    pub _emoji: &'static str,
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
            _emoji: "📦",
            _name: "noun",
            _gng: "_NOUN_",
        },
        Pos::ProperNoun => POSInfo {
            letter: "O",
            ord: 2,
            _ptb: "NNP",
            _emoji: "📛",
            _name: "proper noun",
            _gng: "_PROPN_",
        },
        Pos::Verb => POSInfo {
            letter: "V",
            ord: 4,
            _ptb: "VB",
            _emoji: "🏃",
            _name: "verb",
            _gng: "_VERB_",
        },
        Pos::Adjective => POSInfo {
            letter: "J",
            ord: 5,
            _ptb: "JJ",
            _emoji: "🌈",
            _name: "adjective",
            _gng: "_ADJ_",
        },
        Pos::Adverb => POSInfo {
            letter: "R",
            ord: 6,
            _ptb: "RB",
            _emoji: "🤷",
            _name: "adverb",
            _gng: "_ADV_",
        },
        Pos::Conjunction => POSInfo {
            letter: "C",
            ord: 8,
            _ptb: "CC",
            _emoji: "🔗",
            _name: "conjunction",
            _gng: "_CONJ_",
        },
        Pos::Determiner => POSInfo {
            letter: "D",
            ord: 1,
            _ptb: "DT",
            _emoji: "👉",
            _name: "determiner",
            _gng: "_DET_",
        },
        Pos::Preposition => POSInfo {
            letter: "P",
            ord: 7,
            _ptb: "IN",
            _emoji: "📥",
            _name: "preposition",
            _gng: "_ADP_",
        },
        Pos::Pronoun => POSInfo {
            letter: "I",
            ord: 3,
            _ptb: "PRP",
            _emoji: "👤",
            _name: "pronoun",
            _gng: "_PRON_",
        },
    }
}
