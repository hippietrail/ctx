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
    /// Extracts two families in sorted order and returns both the sorted keys and families.
    /// This ensures consistent ordering across runs.
    pub fn into_two_families_sorted(
        mut self,
    ) -> (Vec<Option<String>>, AlternativeMap, AlternativeMap) {
        let mut keys: Vec<Option<String>> = self.0.keys().cloned().collect();
        keys.sort();
        let fam_a = self.0.remove(&keys[0]).unwrap();
        let fam_b = self.0.remove(&keys[1]).unwrap();
        (keys, fam_a, fam_b)
    }
    pub fn into_three_families_sorted(
        mut self,
    ) -> (
        Vec<Option<String>>,
        AlternativeMap,
        AlternativeMap,
        AlternativeMap,
    ) {
        let mut keys: Vec<Option<String>> = self.0.keys().cloned().collect();
        keys.sort();
        let fam_a = self.0.remove(&keys[0]).unwrap();
        let fam_b = self.0.remove(&keys[1]).unwrap();
        let fam_c = self.0.remove(&keys[2]).unwrap();
        (keys, fam_a, fam_b, fam_c)
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

    /// Parameterized method to collect unique POS references for a specific side.
    pub fn poses_for_side(&self, side: crate::Side) -> HashSet<&'static Pos> {
        self.values()
            .flat_map(|side_map| side_map.poses_for_side(side))
            .collect()
    }

    /// Builds a counting structure: POS → (canonical word → occurrence count)
    /// This counts how many times each word appears with each POS across all alternatives,
    /// using canonicalized spelling to merge case variants.
    pub fn pos_to_word_counts_for_side(
        &self,
        side: crate::Side,
        dict: &FstDictionary,
    ) -> HashMap<&'static Pos, HashMap<String, usize>> {
        self.values()
            .filter_map(|side_map| side_map.get(&side))
            .flat_map(|ctx_set| ctx_set.iter())
            .flat_map(|(word, pos_set)| {
                let (_, canonical) = get_canon_case(dict, word);
                pos_set.iter().map(move |&pos| (pos, canonical.clone()))
            })
            .fold(HashMap::new(), |mut acc, (pos, canonical)| {
                acc.entry(pos)
                    .or_default()
                    .entry(canonical)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
                acc
            })
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

    pub fn poses_for_side(&self, side: crate::Side) -> HashSet<&'static Pos> {
        self.get(&side)
            .map(|ctx_set| ctx_set.all_poses())
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
pub struct ContextSet(pub HashMap<String, HashSet<&'static Pos>>);

impl ContextSet {
    pub fn contexts(&self) -> HashSet<String> {
        self.keys().cloned().collect()
    }

    pub fn all_poses(&self) -> HashSet<&'static Pos> {
        self.values()
            .flat_map(|pos_set| pos_set.iter().copied())
            .collect()
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
    type Target = HashMap<String, HashSet<&'static Pos>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ContextSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
