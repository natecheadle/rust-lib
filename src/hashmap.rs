//! A hash map

pub mod fixed_hasher;
pub mod iterator;
mod key_value;

use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    hash::{BuildHasher, Hash, RandomState},
    mem,
    ops::{Index, IndexMut},
    ptr::{NonNull, drop_in_place},
    slice::{from_raw_parts, from_raw_parts_mut},
};

use key_value::KeyValue;

fn create_hashmap_layout<K, V>(capacity: usize) -> Layout {
    Layout::array::<Option<KeyValue<K, V>>>(capacity).expect("capacity overflow")
}

/// Hashmap structure
pub struct HashMap<K, V, S = RandomState> {
    data: NonNull<Option<KeyValue<K, V>>>,
    capacity: usize,
    size: usize,
    build_hasher: S,
}

fn get_probe_distance(pos: usize, ideal: usize, mask: usize) -> usize {
    pos.wrapping_sub(ideal) & mask
}

fn get_next_location(pos: usize, mask: usize) -> usize {
    pos.wrapping_add(1) & mask
}

#[allow(clippy::cast_possible_truncation)]
fn get_location_from_hash(hash: u64, mask: usize) -> usize {
    hash as usize & mask
}

/// Find an index of a slice to insert a new slot.
/// This uses the robin hood algorithm so takes a mutable
/// slice and swaps items based on the algorithm.
///
/// Panics:
/// - Will panic if no empty slot found
fn robin_hood_insert_slice<K, V>(
    slice: &mut [Option<KeyValue<K, V>>],
    key_value: KeyValue<K, V>,
) -> Option<V>
where
    K: Eq,
{
    debug_assert!(slice.len().is_power_of_two());
    let mask = slice.len() - 1;
    let mut key_value_mut = key_value;
    let mut current_idx = get_location_from_hash(key_value_mut.hash, mask);
    let mut ideal_idx = current_idx;
    let mut i = 0;
    loop {
        if let Some(current) = &slice[current_idx] {
            if current == &key_value_mut {
                let old = slice[current_idx].replace(key_value_mut).unwrap();

                return Some(old.value);
            }
            let seated_ideal_idx = get_location_from_hash(current.hash, mask);
            let seated_distance = get_probe_distance(current_idx, seated_ideal_idx, mask);
            let current_distance = get_probe_distance(current_idx, ideal_idx, mask);
            if current_distance > seated_distance {
                key_value_mut = slice[current_idx].replace(key_value_mut).unwrap();
                ideal_idx = seated_ideal_idx;
            }
        } else {
            slice[current_idx] = Some(key_value_mut);
            return None;
        }

        current_idx = current_idx.wrapping_add(1) & mask;
        i += 1;
        assert!(i < slice.len(), "Slice was full");
    }
}

fn get_index<K, V>(slice: &[Option<KeyValue<K, V>>], hash: u64, key: &K) -> Option<usize>
where
    K: Eq,
{
    if slice.is_empty() {
        return None;
    }

    debug_assert!(slice.len().is_power_of_two());
    let mask = slice.len() - 1;
    let mut loc_idx = get_location_from_hash(hash, mask);

    for _ in 0..slice.len() {
        if let Some(rslt) = &slice[loc_idx] {
            if rslt.hash == hash && &rslt.key == key {
                return Some(loc_idx);
            }
            loc_idx = get_next_location(loc_idx, mask);
        } else {
            break;
        }
    }

    None
}

impl<K, V, S> Drop for HashMap<K, V, S> {
    fn drop(&mut self) {
        let capacity = self.capacity;
        if capacity > 0 {
            {
                let data_slice = self.data_as_slice_mut();
                for item in data_slice.iter_mut().take(capacity) {
                    *item = None;
                }
            }

            let layout = create_hashmap_layout::<K, V>(capacity);

            // Safety:
            // This is safe because we are using the old layout
            // and we already dropped all the items
            unsafe { dealloc(self.data.as_ptr().cast::<u8>(), layout) };

            self.data = NonNull::dangling();
            self.size = 0;
            self.capacity = 0;
        }
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default,
{
    /// Build a new hashmap with the default constructed hasher
    #[must_use]
    pub fn new() -> Self {
        HashMap::<K, V, S>::default()
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher + Default,
{
    fn default() -> Self {
        HashMap {
            data: NonNull::dangling(),
            capacity: 0,
            size: 0,
            build_hasher: S::default(),
        }
    }
}

impl<K, V, S> HashMap<K, V, S> {
    fn data_as_slice(&self) -> &[Option<KeyValue<K, V>>] {
        // Safety:
        // Capacity and data are maintained together so this is safe to generate a slice from
        unsafe { from_raw_parts(self.data.as_ptr(), self.capacity) }
    }

    fn data_as_slice_mut(&mut self) -> &mut [Option<KeyValue<K, V>>] {
        // Safety:
        // Capacity and data are maintained together so this is safe to generate a slice from
        unsafe { from_raw_parts_mut(self.data.as_ptr(), self.capacity) }
    }
}

impl<K, V, S> Index<&K> for HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Output = V;

    fn index(&self, key: &K) -> &Self::Output {
        self.get(key).expect("Key is not in map")
    }
}

impl<K, V, S> IndexMut<&K> for HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    fn index_mut(&mut self, key: &K) -> &mut Self::Output {
        self.get_mut(key).expect("Key is not in map")
    }
}

struct CloneHelper<K, V> {
    data: NonNull<Option<KeyValue<K, V>>>,
    capacity: usize,
    initialized: usize,
}

impl<K, V> Drop for CloneHelper<K, V> {
    fn drop(&mut self) {
        for i in 0..self.initialized {
            // Safety:
            // This is safe because we are only droppin initialized objects
            unsafe { drop_in_place(self.data.add(i).as_ptr()) };
        }

        let layout = create_hashmap_layout::<K, V>(self.capacity);
        // Safety:
        // This is safe because we are freeing data that we own
        unsafe { dealloc(self.data.as_ptr().cast::<u8>(), layout) };
    }
}

impl<K, V, S> Clone for HashMap<K, V, S>
where
    K: Clone,
    V: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        if self.capacity == 0 {
            return HashMap::<K, V, S> {
                data: NonNull::dangling(),
                capacity: 0,
                size: self.size,
                build_hasher: self.build_hasher.clone(),
            };
        }

        let layout = create_hashmap_layout::<K, V>(self.capacity);

        // Safety:
        // This is safe because we built the laout from the same type and capacity
        let new_data = unsafe { alloc(layout).cast::<Option<KeyValue<K, V>>>() };
        if new_data.is_null() {
            handle_alloc_error(layout);
        }
        // Safety:
        // This is safe because the null check was just done
        let new_data = unsafe { NonNull::new_unchecked(new_data) };
        let mut new_data = CloneHelper {
            data: new_data,
            capacity: self.capacity,
            initialized: 0,
        };

        let existing_data_slice = self.data_as_slice();

        for (i, kv) in existing_data_slice.iter().enumerate() {
            // Safety:
            // This is safe because the memory is uninitialized at the destination.
            unsafe {
                new_data.data.add(i).write(kv.clone());
            }

            new_data.initialized += 1;
        }
        let new_obj = HashMap::<K, V, S> {
            data: new_data.data,
            capacity: new_data.capacity,
            size: self.size,
            build_hasher: self.build_hasher.clone(),
        };
        mem::forget(new_data);

        new_obj
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Build a new hashmap with a custom hasher
    pub fn new_with_hasher(build_hasher: S) -> Self {
        HashMap {
            data: NonNull::dangling(),
            capacity: 0,
            size: 0,
            build_hasher,
        }
    }

    fn rehash(&mut self, new_size: usize) {
        debug_assert!(new_size.is_power_of_two());
        debug_assert!(new_size >= self.size);

        let new_layout = create_hashmap_layout::<K, V>(new_size);

        // Safety:
        // This is safe because we later validate that the allocation succeeded
        let new_data = unsafe { alloc(new_layout).cast::<Option<KeyValue<K, V>>>() };
        if new_data.is_null() {
            handle_alloc_error(new_layout);
        }

        // Safety:
        // This is safe because we already validated that the pointer is not null
        let new_data = unsafe { NonNull::new_unchecked(new_data) };

        for i in 0..new_size {
            // Safety:
            // This is safe because we are indexing inside the known size.
            // and only overwriting currently uninitialized memory
            unsafe { new_data.add(i).write(None) };
        }

        // Safety:
        // This is safe because we are creating a slice the same size as the allocation
        let slice_new = unsafe { from_raw_parts_mut(new_data.as_ptr(), new_size) };
        // Move existing items to new buffer location
        for i in 0..self.capacity {
            // Safety:
            // This is safe because we are indexing into a any array less than its capacity
            let current_data = unsafe { self.data.add(i).as_mut() };
            if let Some(occupied_data) = current_data.take() {
                assert!(
                    robin_hood_insert_slice(slice_new, occupied_data).is_none(),
                    "slot was not expected to be occupied"
                );
            }
        }

        // Release existing memory
        if self.capacity > 0 {
            let old_layout = create_hashmap_layout::<K, V>(self.capacity);

            // Safety:
            // All the old data has already been moved to the new slice so this can be safely destroyed
            unsafe { dealloc(self.data.as_ptr().cast::<u8>(), old_layout) };
        }

        self.data = new_data;
        self.capacity = new_size;
    }

    /// Grow the existing capacity if necessary
    fn grow(&mut self) {
        if !self.should_grow() {
            return;
        }

        let new_size = if self.capacity == 0 {
            8
        } else {
            self.capacity * 2
        };

        self.rehash(new_size);
    }

    /// Insert the specified value given a specified key
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.grow();

        let new_value = KeyValue::<K, V>::new_with_hasher(&self.build_hasher, key, value);
        let old_value = robin_hood_insert_slice(self.data_as_slice_mut(), new_value);
        if old_value.is_none() {
            self.size += 1;
        }

        old_value
    }

    /// Get the value at the specied key location
    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.build_hasher.hash_one(key);
        let slice = self.data_as_slice();
        if let Some(rslt) = get_index(slice, hash, key)
            && let Some(rslt) = &slice[rslt]
        {
            Some(&rslt.value)
        } else {
            None
        }
    }

    /// Get the mutable value at the specied key location
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = self.build_hasher.hash_one(key);
        let slice = self.data_as_slice_mut();
        if let Some(rslt) = get_index(slice, hash, key)
            && let Some(rslt) = &mut slice[rslt]
        {
            Some(&mut rslt.value)
        } else {
            None
        }
    }

    /// Remove value at specified key and return the old value if the key existed.
    ///
    /// This uses backward shift deletions so other items might be moved at the same time.
    #[allow(clippy::missing_panics_doc)] // panics only occur if there are bugs in the code
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = self.build_hasher.hash_one(key);
        {
            let rslt = get_index(self.data_as_slice(), hash, key);
            if let Some(rslt) = rslt {
                let capacity = self.capacity;
                let slice = self.data_as_slice_mut();
                let old_value = slice[rslt].take();

                let mask = capacity - 1;
                let mut previous_location = rslt;
                let mut next_location = get_next_location(rslt, mask);
                while let Some(next_keyval) = &slice[next_location]
                    && get_location_from_hash(next_keyval.hash, mask) != next_location
                {
                    slice.swap(previous_location, next_location);
                    previous_location = next_location;
                    next_location = get_next_location(next_location, mask);
                }
                self.size -= 1;
                Some(old_value.expect("Slot was not actually occupied").value)
            } else {
                None
            }
        }
    }

    /// Returns the number of items in the map
    pub fn len(&self) -> usize {
        self.size
    }

    /// Returns true if the container is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn should_grow(&self) -> bool {
        self.size >= (self.capacity * 3) / 4
    }
}

#[cfg(test)]
mod test {

    use super::fixed_hasher::BuildFixedHasher;
    use super::{HashMap, get_next_location, get_probe_distance};

    #[test]
    fn grow_at_correct_times() {
        let mut map = HashMap::<i32, i32>::new();
        map.insert(0, 0);
        assert_eq!(map.capacity, 8);
        for i in 1..6 {
            map.insert(i, i * 2);
        }
        assert_eq!(map.capacity, 8);
        map.insert(6, 12);
        assert_eq!(map.capacity, 16);
    }

    #[test]
    fn wrap_around_properly_retrieves_values() {
        let build_hasher = BuildFixedHasher { hash: 7 };
        let mut map = HashMap::<i32, i32, BuildFixedHasher>::new_with_hasher(build_hasher);
        map.insert(1, 1);
        map.insert(2, 2);
        map.insert(3, 3);

        assert_eq!(map.remove(&1).unwrap(), 1);

        assert_eq!(map.get(&2).unwrap(), &2);
        assert_eq!(map.get(&3).unwrap(), &3);
    }

    #[test]
    fn probe_distance_handles_wraparound() {
        // ideal slot 6, wrapped forward past the end to position 1
        // true distance: 6 -> 7 -> 0 -> 1, three steps
        assert_eq!(3, get_probe_distance(1, 6, 7));
    }

    #[test]
    fn probe_distance_no_wraparound_matches_simple_subtraction() {
        assert_eq!(2, get_probe_distance(5, 3, 7));
    }

    #[test]
    fn probe_distance_underflow_returns_correct_value() {
        assert_eq!(1, get_probe_distance(0, 7, 7));
    }

    #[test]
    fn next_location_handles_wraparound() {
        assert_eq!(0, get_next_location(7, 7));
    }

    #[test]
    fn next_location_handles_overflow() {
        assert_eq!(0, get_next_location(usize::MAX, usize::MAX));
    }

    #[test]
    fn next_location_no_wraparound_adds_1() {
        assert_eq!(7, get_next_location(6, 7));
    }
}
