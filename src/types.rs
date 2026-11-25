// ============================================================================
// IMPORTS
// ============================================================================

// 1. Standard Library Imports
use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

// 2. External Crate Imports
use harper_core::spell::{Dictionary, FstDictionary};

// 3. Local Crate Imports
use crate::Pos;

// Helper function to canonicalize word spelling
fn get_canon_case(dict: &FstDictionary, s: &str) -> (bool, String) {
    dict.get_correct_capitalization_of(&s.chars().collect::<Vec<char>>())
        .map(|v| (true, v.iter().collect::<String>()))
        .unwrap_or_else(|| (false, s.to_string()))
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug, Default)]
pub struct FamilyTree(pub HashMap<Option<String>, AlternativeMap>);

impl FamilyTree {
    /// Extracts all families in sorted key order.
    pub fn into_families_sorted(mut self) -> Vec<(String, AlternativeMap)> {
        let mut keys: Vec<Option<String>> = self.0.keys().cloned().collect();
        keys.sort();
        keys.into_iter()
            .map(|k| {
                let map = self.0.remove(&k).unwrap();
                let name = k.unwrap_or_else(|| "None".to_string());
                (name, map)
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct AlternativeMap(pub HashMap<String, SideMap>);

impl AlternativeMap {
    /// Collects unique context words from a specific side of every alternative.
    pub fn contexts_for_side(&self, side: crate::Side) -> HashSet<String> {
        let mut set = HashSet::new();
        for side_map in self.values() {
            set.extend(side_map.contexts_for_side(side));
        }
        set
    }

    /// Builds a counting structure: POS → (canonical word → occurrence count)
    /// This counts how many times each word appears with each POS across all alternatives,
    /// using canonicalized spelling to merge case variants.
    pub fn pos_to_word_counts_for_side(
        &self,
        side: crate::Side,
        dict: &FstDictionary,
    ) -> HashMap<&'static Pos, HashMap<String, f64>> {
        self.values()
            .filter_map(|side_map| side_map.get(&side))
            .flat_map(|ctx_set| ctx_set.iter())
            .flat_map(|(word, entry)| {
                let (_, canonical) = get_canon_case(dict, word);
                let freq = entry.freq;
                entry
                    .poses
                    .iter()
                    .map(move |&pos| (pos, canonical.clone(), freq))
            })
            .fold(HashMap::new(), |mut acc, (pos, canonical, freq)| {
                *acc.entry(pos).or_default().entry(canonical).or_insert(0.0) += freq;
                acc
            })
    }

    /// Builds a counting structure: canonical word → occurrence count across alternatives
    /// This counts how many alternatives in this family each word appears with.
    pub fn word_counts_for_side(
        &self,
        side: crate::Side,
        dict: &FstDictionary,
    ) -> HashMap<String, f64> {
        let mut counts = HashMap::new();
        for side_map in self.values() {
            if let Some(ctx_set) = side_map.get(&side) {
                for (word, entry) in ctx_set.iter() {
                    let (_, canonical) = get_canon_case(dict, word);
                    *counts.entry(canonical).or_insert(0.0) += entry.freq;
                }
            }
        }
        counts
    }
}

#[derive(Debug, Default)]
pub struct SideMap(pub HashMap<crate::Side, ContextSet>);

impl SideMap {
    pub fn contexts_for_side(&self, side: crate::Side) -> HashSet<String> {
        self.get(&side)
            .map(|ctx_set| ctx_set.contexts())
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
pub struct ContextEntry {
    pub poses: HashSet<&'static Pos>,
    pub freq: f64,
}

#[derive(Debug, Default)]
pub struct ContextSet(pub HashMap<String, ContextEntry>);

impl ContextSet {
    pub fn contexts(&self) -> HashSet<String> {
        self.keys().cloned().collect()
    }
}

// --- DEREF / DEREFMUT BOILERPLATE ---
// This allows your structs to implicitly behave like their inner collections!

impl Deref for FamilyTree {
    type Target = HashMap<Option<String>, AlternativeMap>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for FamilyTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for AlternativeMap {
    type Target = HashMap<String, SideMap>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for AlternativeMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for SideMap {
    type Target = HashMap<crate::Side, ContextSet>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for SideMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for ContextSet {
    type Target = HashMap<String, ContextEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ContextSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
