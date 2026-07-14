//! A growable, heap-allocated vector that is analogous to `std::vec::Vec`.

mod into_iter;
mod raw;

pub use into_iter::IntoIter;
use raw::RawVector;
use std::fmt::Debug;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr::{self, drop_in_place};

/// Initial growth of capacity on initial push
pub const INITIAL_GROWTH: usize = 16;
/// Max amount of capacity that a vector will grow automatically
pub const MAX_GROWTH: usize = 1024;

/// Heap allocated array
pub struct Vector<T> {
    memory: RawVector<T>,
    size: usize,
}

impl<T> Vector<T> {
    #[must_use]
    /// Create a new empty vector
    pub fn new() -> Self {
        Vector {
            memory: raw::RawVector::new(),
            size: 0,
        }
    }

    /// Push a single value onto the back of the vector
    ///
    /// # Panics
    ///
    /// This function will panic if the push causes capacity to increase and the alloc/realloc fails
    pub fn push(&mut self, value: T) {
        // Skip all the dead code for zst types and just increment the size
        if size_of::<T>() > 0 {
            self.grow();

            let loc = unsafe {
                // Safety:
                // This is safe because we already increased the capacity to ensure we are
                // indexing into valid uninitialized memory.
                self.memory.data.add(self.size).as_mut()
            };

            loc.write(value);
        }
        self.size += 1;
    }

    /// Pop the value at the end of the vector. Return None if vector is empty
    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            None
        } else {
            let popped = unsafe {
                // Safety:
                // This is safe because the size is guarunteed to be the last value in the array.
                // For zst this works because it doesn't actually read the memory
                self.memory.data.add(self.size - 1).read().assume_init()
            };
            self.size -= 1;

            Some(popped)
        }
    }

    /// Remove and drop all items in the vector
    pub fn clear(&mut self) {
        if self.size > 0 {
            if size_of::<T>() > 0 {
                for i in 0..self.size {
                    let ptr = self.memory.data.as_ptr().cast::<T>();
                    unsafe {
                        // Safety:
                        // This is safe because we are only dropping initialized items
                        drop_in_place(ptr.add(i));
                    }
                }
            }
            self.size = 0;
        }
    }

    /// Allocate an additional amount of items without initializing
    ///
    /// # Panics
    /// Will panic if reserve causes overflow on usize type
    pub fn reserve(&mut self, to_reserve: usize) {
        self.memory.grow(
            self.size
                .checked_add(to_reserve)
                .expect("capacity overflow"),
        );
    }

    /// Allocate at least enough memory for the provided amount of items without initializing
    pub fn reserve_total(&mut self, to_reserve: usize) {
        self.memory.grow(to_reserve);
    }

    /// Get the current length of the vector
    #[must_use]
    pub fn len(&self) -> usize {
        self.size
    }

    /// Check whether the current vector is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the current allocated capacity of the vector
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.memory.capacity
    }

    /// Insert proviced value at the provided index location
    ///
    /// # Panics
    /// 1. This function will panic if index is greater than size
    /// 2. This function will panic if the new size needed exceeds allowed capacity
    pub fn insert(&mut self, index: usize, value: T) {
        assert!(
            index <= self.size,
            "index out of bounds: the len is {} but the index is {index}",
            self.size
        );

        if index == self.size {
            self.push(value);
            return;
        }

        // Conditioning on size_of::<T> to skip all the dead code in the zst case
        if size_of::<T>() > 0 {
            self.grow();

            let to_copy = self.size - index;

            // Safety:
            // This is safe because the capacity has already been validated wtih grow()
            // and the index was asserted to be less than the size
            unsafe {
                ptr::copy(
                    self.memory.data.add(index).as_ptr(),
                    self.memory.data.add(index + 1).as_ptr(),
                    to_copy,
                );
            }

            // Safety:
            // This is safe because the item was already removed in the previous copy
            // operation and we don't want it dropped.
            unsafe {
                ptr::write(
                    self.memory.data.add(index).as_ptr(),
                    MaybeUninit::new(value),
                );
            }
        }

        self.size += 1;
    }

    /// Remove an item from the vector
    ///
    /// # Panics
    /// This function will panic if index is not less than `len()`
    pub fn remove(&mut self, index: usize) -> T {
        assert!(
            index < self.size,
            "index out of bounds: the len is {} but the index is {index}",
            self.size
        );

        // Conditioning on size_of::<T> to skip all the dead code in the zst case
        if size_of::<T>() > 0 {
            // Safety:
            // This is safe because we already validated index.
            let rslt = unsafe { self.memory.data.add(index).read().assume_init() };

            let to_copy = self.size - index - 1;

            // Safety:
            // This is safe because index has already been validated and the
            // item has already been removed so can be safely copied over.
            unsafe {
                ptr::copy(
                    self.memory.data.add(index + 1).as_ptr(),
                    self.memory.data.add(index).as_ptr(),
                    to_copy,
                );
            }
            self.size -= 1;

            rslt
        } else {
            self.size -= 1;

            // Safety:
            // This a zst so this is safe
            unsafe { get_zst_default() }
        }
    }

    /// Execute the default grow algorithm to
    fn grow(&mut self) {
        let current_capacity = self.memory.capacity;

        if current_capacity > 0 {
            if current_capacity == self.size {
                let new_capacity = if current_capacity > MAX_GROWTH / 2 {
                    current_capacity + MAX_GROWTH
                } else {
                    current_capacity * 2
                };

                self.memory.grow(new_capacity);
            }
        } else {
            self.memory.grow(INITIAL_GROWTH);
        }
    }
}

/// Return a default T for a ZST
///
/// # Safety
/// This is safe as along as T is a ZST
#[allow(clippy::uninit_assumed_init)]
unsafe fn get_zst_default<T>() -> T {
    // Safety:
    // Instantiating a ZST from uninitialized or zeroed memory is completely safe
    // because it contains no actual data or invalid bit patterns to violate safety.
    unsafe { MaybeUninit::uninit().assume_init() }
}

impl<T> Default for Vector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T> Index<usize> for Vector<T> {
    type Output = T;

    /// # Panics
    /// Panics if the index is greater than the size of the array
    fn index(&self, index: usize) -> &Self::Output {
        assert!(
            index < self.size,
            "index out of bounds: the len is {} but the index is {index}",
            self.size
        );

        // Safety:
        // This is safe because the range has already been validated so
        // will always return a valid reference
        // For zst this is a no opp and guarutneed to work
        unsafe { self.memory.data.cast::<T>().add(index).as_ref() }
    }
}

impl<T> IndexMut<usize> for Vector<T> {
    /// # Panics
    /// Panics if the index is greater than the size of the array
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(
            index < self.size,
            "index out of bounds: the len is {} but the index is {index}",
            self.size
        );

        // Safety:
        // This is safe because the range has already been validated so
        // will always return a valid reference
        // For zst this is a no opp and guarutneed to work
        unsafe { self.memory.data.cast::<T>().add(index).as_mut() }
    }
}

impl<T> Clone for Vector<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new_obj = Vector::new();
        if size_of::<T>() > 0 {
            new_obj.reserve(self.size);
            for item in self.iter() {
                new_obj.push(item.clone());
            }
        } else {
            new_obj.size = self.size;
        }
        new_obj
    }
}

impl<T> Deref for Vector<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        // Safety:
        // This is safe because it uses the size that is guarunteed to be initialized.
        // For zst this is safe because the memory is never read
        unsafe { std::slice::from_raw_parts(self.memory.data.cast::<T>().as_ptr(), self.size) }
    }
}

impl<T> DerefMut for Vector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety:
        // This is safe because it uses the size that is guarunteed to be initialized.
        // For zst this is safe because the memory is never read
        unsafe { std::slice::from_raw_parts_mut(self.memory.data.cast::<T>().as_ptr(), self.size) }
    }
}

impl<T> PartialEq for Vector<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T> Debug for Vector<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let this = ManuallyDrop::new(self);
        // Safety:
        // This is safe because IntoIter is taking ownership of Vector<T> memory.
        let memory = unsafe { ptr::read(&raw const this.memory) };

        IntoIter {
            buf: memory,
            start: 0,
            end: this.size,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Vector;

    #[test]
    fn len_matches_size() {
        let mut vec: Vector<usize> = Vector::new();
        assert_eq!(vec.len(), vec.size);

        for i in 0..10 {
            vec.push(i);
            assert_eq!(vec.len(), vec.size);
        }
    }
}
