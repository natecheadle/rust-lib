//! Iterator types and implementations for [`HashMap`]

use super::{HashMap, KeyValue};

use std::{
    alloc::dealloc,
    mem::ManuallyDrop,
    ptr::{self, NonNull},
};

/// Iterator for [`HashMap`]
pub struct HashMapIterator<'a, K, V, S> {
    container: &'a HashMap<K, V, S>,
    current: usize,
}

/// Mutable Iterator for [`HashMap`]
pub struct HashMapIteratorMut<'a, K, V, S> {
    container: &'a mut HashMap<K, V, S>,
    current: usize,
}

/// Into-Iterator for [`HashMap`]
pub struct HashMapIntoIter<K, V> {
    data: NonNull<Option<KeyValue<K, V>>>,
    capacity: usize,
    current: usize,
}

impl<K, V> Drop for HashMapIntoIter<K, V> {
    fn drop(&mut self) {
        for i in self.current..self.capacity {
            // Safety:
            // This is safe because we know we have not yet consumed these items
            unsafe { self.data.add(i).drop_in_place() };
        }

        let layout = super::create_hashmap_layout::<K, V>(self.capacity);

        // Safety:
        // This is safe because we already dropped the items,
        // and we built the layout from the correct type and capacity
        unsafe { dealloc(self.data.cast::<u8>().as_ptr(), layout) };

        self.data = NonNull::dangling();
        self.current = 0;
        self.capacity = 0;
    }
}

impl<'a, K, V, S> HashMapIterator<'a, K, V, S> {
    /// Create a mutable iterator from a container
    pub fn new(container: &'a HashMap<K, V, S>) -> HashMapIterator<'a, K, V, S> {
        HashMapIterator {
            container,
            current: 0,
        }
    }
}

impl<'a, K, V, S> HashMapIteratorMut<'a, K, V, S> {
    /// Create a new mutable iterator from a container
    pub fn new(container: &'a mut HashMap<K, V, S>) -> HashMapIteratorMut<'a, K, V, S> {
        HashMapIteratorMut {
            container,
            current: 0,
        }
    }
}

impl<K, V> HashMapIntoIter<K, V> {
    /// Create a new consuming iterator from the internals of a container
    pub(super) fn new(
        data: NonNull<Option<KeyValue<K, V>>>,
        capacity: usize,
    ) -> HashMapIntoIter<K, V> {
        HashMapIntoIter {
            data,
            capacity,
            current: 0,
        }
    }
}

impl<'a, K, V, S> Iterator for HashMapIterator<'a, K, V, S> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        for i in self.current..self.container.capacity {
            // Safety:
            // This is safe because we are indexing into an array and already validated the size
            if let Some(current_val) = unsafe { self.container.data.add(i).as_ref() } {
                self.current = i + 1;
                return Some((&current_val.key, &current_val.value));
            }
        }
        None
    }
}

impl<'a, K, V, S> Iterator for HashMapIteratorMut<'a, K, V, S> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        for i in self.current..self.container.capacity {
            // Safety:
            // This is safe because we are indexing into an array and already validated the size
            if let Some(current_val) = unsafe { self.container.data.add(i).as_mut() } {
                self.current = i + 1;
                return Some((&current_val.key, &mut current_val.value));
            }
        }
        None
    }
}
impl<K, V> Iterator for HashMapIntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        for i in self.current..self.capacity {
            // Safety:
            // This is safe because we are indexing into an array and already validated the size
            if let Some(current_val) = unsafe { self.data.add(i).replace(None) } {
                self.current = i + 1;
                return Some((current_val.key, current_val.value));
            }
        }
        None
    }
}

impl<K, V, S> HashMap<K, V, S> {
    /// Create an iterator from the current object
    pub fn iter(&self) -> HashMapIterator<'_, K, V, S> {
        self.into_iter()
    }

    /// Create a mutable iterator from the current object
    pub fn iter_mut(&mut self) -> HashMapIteratorMut<'_, K, V, S> {
        self.into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = HashMapIterator<'a, K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        HashMapIterator::new(self)
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = HashMapIteratorMut<'a, K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        HashMapIteratorMut::new(self)
    }
}

impl<K, V, S> IntoIterator for HashMap<K, V, S> {
    type IntoIter = HashMapIntoIter<K, V>;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter {
        let mut this = ManuallyDrop::new(self);
        // Safety:
        // This is safe because we are going to mobe all the other fields out of the
        // object and need to manually drop this.
        unsafe { ptr::drop_in_place(&raw mut this.build_hasher) };
        Self::IntoIter::new(this.data, this.capacity)
    }
}
