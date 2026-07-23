//! A `Hasher` that always returns the same value.

use std::hash::{BuildHasher, Hasher};

#[derive(Default)]
/// Struct for hasher that always returns the same value
pub struct FixedHasher {
    /// Hash value to always return
    pub hash: u64,
}

impl Hasher for FixedHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, _: &[u8]) {}

    fn write_u64(&mut self, _: u64) {}
}

#[derive(Default)]
/// Struct for building a fixed hasher that always returns the same value
pub struct BuildFixedHasher {
    /// Hash value to always return
    pub hash: u64,
}

impl BuildHasher for BuildFixedHasher {
    type Hasher = FixedHasher;

    fn build_hasher(&self) -> Self::Hasher {
        FixedHasher { hash: self.hash }
    }
}
