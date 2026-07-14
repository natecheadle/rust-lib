use std::alloc::{Layout, alloc, dealloc, handle_alloc_error, realloc};
use std::mem::{self, MaybeUninit};
use std::ptr::NonNull;

pub(super) struct RawVector<T> {
    pub(super) data: NonNull<MaybeUninit<T>>,
    pub(super) capacity: usize,
}

impl<T> RawVector<T> {
    #[must_use]
    pub(super) fn new() -> Self {
        RawVector {
            data: NonNull::<MaybeUninit<T>>::dangling(),
            capacity: 0,
        }
    }
    pub(super) fn grow(&mut self, new_size: usize) {
        assert!(mem::size_of::<T>() != 0, "We're not ready to handle ZSTs");
        if new_size == 0 || self.capacity >= new_size {
            // Nothing to do already sufficient size
        } else {
            let new_layout = Layout::array::<T>(new_size);
            let new_layout = new_layout.expect("capacity overflow");

            if self.capacity > 0 {
                let old_layout = Layout::array::<T>(self.capacity);
                let old_layout = old_layout.expect("capacity overflow");

                let ptr = unsafe {
                    // Safety:
                    // Returned pointer must be validated to confirm allocation succeeded.
                    realloc(
                        self.data.as_ptr().cast::<u8>(),
                        old_layout,
                        new_layout.size(),
                    )
                    .cast::<MaybeUninit<T>>()
                };

                if ptr.is_null() {
                    handle_alloc_error(new_layout);
                }

                self.data = unsafe {
                    // Safety:
                    // Pointer validated to be not null so doesn't require check.
                    NonNull::new_unchecked(ptr)
                };
            } else {
                let ptr = unsafe {
                    // Safety:
                    // Returned pointer must be validated to confirm allocation succeeded.
                    alloc(new_layout).cast::<MaybeUninit<T>>()
                };
                if ptr.is_null() {
                    handle_alloc_error(new_layout);
                }
                self.data = unsafe {
                    // Safety:
                    // Pointer validated to be not null so doesn't require check.
                    NonNull::new_unchecked(ptr)
                };
            }
            self.capacity = new_size;
        }
    }
}

impl<T> Drop for RawVector<T> {
    // Deallocates the memory allocating by this structure
    //
    // Safety:
    // Any initialized values must be dropped before calling this drop function
    fn drop(&mut self) {
        if self.capacity > 0 {
            let layout = Layout::array::<T>(self.capacity);
            let layout = layout.expect("capacity overflow");
            unsafe {
                // Safety:
                // Pointer is valid if capacity is greater than 0
                dealloc(self.data.as_ptr().cast::<u8>(), layout);
            }

            self.capacity = 0;
        }
    }
}

#[cfg(test)]
mod test {
    use super::RawVector;

    #[test]
    fn create_succeeds() {
        let raw: RawVector<i64> = RawVector::new();
        assert_eq!(raw.capacity, 0);
    }

    #[test]
    fn grow_zero_first_does_nothing() {
        let mut raw: RawVector<i64> = RawVector::new();
        raw.grow(0);

        assert_eq!(raw.capacity, 0);
    }

    #[test]
    fn grow_once_succeeds() {
        const NEW_CAPACITY: usize = 10;
        let mut raw: RawVector<i64> = RawVector::new();
        raw.grow(NEW_CAPACITY);
        assert_eq!(raw.capacity, NEW_CAPACITY);
    }

    #[test]
    fn grow_less_than_capacity_does_nothing() {
        let mut raw: RawVector<i64> = RawVector::new();
        raw.grow(100);
        raw.grow(10);
        assert_eq!(raw.capacity, 100);
    }

    #[test]
    fn grow_equal_capacity_does_nothing() {
        let mut raw: RawVector<i64> = RawVector::new();
        raw.grow(100);
        raw.grow(100);
        assert_eq!(raw.capacity, 100);
    }

    #[test]
    fn grow_twice_is_successful() {
        let mut raw: RawVector<i64> = RawVector::new();
        raw.grow(100);
        raw.grow(1000);
        assert_eq!(raw.capacity, 1000);
    }
}
