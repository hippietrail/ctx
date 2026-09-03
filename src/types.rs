// ============================================================================
// IMPORTS
// ============================================================================

// 1. Standard Library Imports
use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

// 2. Local Crate Imports
use crate::Pos;
use harper_core::spell::{Dictionary, FstDictionary};

// Helper function to canonicalize word spelling
fn get_correct_capitalization_of_string(dict: &FstDictionary, s: &str) -> String {
    let s_chars: Vec<char> = s.chars().collect();
    dict.get_correct_capitalization_of(&s_chars)
        .map(|v| v.iter().collect::<String>())
        .unwrap_or_else(|| s.to_string())
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
        let mut set = HashSet::new();
        for side_map in self.values() {
            set.extend(side_map.poses_for_side(side));
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
    ) -> HashMap<&'static Pos, HashMap<String, usize>> {
        let mut result: HashMap<&'static Pos, HashMap<String, usize>> = HashMap::new();

        for side_map in self.values() {
            if let Some(ctx_set) = side_map.get(&side) {
                for (word, pos_set) in ctx_set.iter() {
                    // Canonicalize the word before counting
                    let canonical_word = get_correct_capitalization_of_string(dict, word);

                    for &pos in pos_set {
                        result
                            .entry(pos)
                            .or_default()
                            .entry(canonical_word.clone())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    }
                }
            }
        }
        result
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
        let mut set = HashSet::new();
        for pos_set in self.values() {
            set.extend(pos_set.iter().copied());
        }
        set
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
