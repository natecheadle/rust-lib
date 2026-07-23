use std::hash::{BuildHasher, Hash};

pub(super) struct KeyValue<K, V> {
    pub(super) hash: u64,
    pub(super) key: K,
    pub(super) value: V,
}

impl<K, V> PartialEq for KeyValue<K, V>
where
    K: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.key == other.key
    }
}

impl<K, V> KeyValue<K, V>
where
    K: Hash,
{
    pub(super) fn new_with_hasher<S: BuildHasher>(build_hasher: &S, key: K, value: V) -> Self {
        KeyValue {
            hash: build_hasher.hash_one(&key),
            key,
            value,
        }
    }
}

impl<K, V> Clone for KeyValue<K, V>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        KeyValue {
            hash: self.hash,
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}
