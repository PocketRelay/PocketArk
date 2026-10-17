use std::{
    collections::HashMap,
    hash::{BuildHasher, Hasher},
};

/// Alias for a [`HashMap`] that used [`IntHasher`] as its [`Hasher`]
pub type IntHashMap<K, V> = HashMap<K, V, BuildIntHasher>;

/// Const safe [`BuildHasher`] implementation for [`IntHasher`]
#[derive(Default)]
pub struct BuildIntHasher;

impl BuildHasher for BuildIntHasher {
    type Hasher = IntHasher;

    fn build_hasher(&self) -> Self::Hasher {
        IntHasher(0)
    }
}

/// Const safe init shorthand function for [`IntHashMap`] that
/// doesn't allocate anything until used
#[inline(always)]
pub const fn int_hash_map<K, V>() -> IntHashMap<K, V> {
    IntHashMap::with_hasher(BuildIntHasher)
}

/// Hasher implementation that directly uses an integer value
/// instead of any specific hashing algorithm
///
/// Only implements hashing for [u32] and [u64]
///
/// Used for hashing packet component paths and type Ids
#[derive(Default)]
pub struct IntHasher(u64);

impl Hasher for IntHasher {
    fn write(&mut self, _: &[u8]) {
        panic!("Attempted to use int hasher to hash bytes")
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.0 = i as u64;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}
