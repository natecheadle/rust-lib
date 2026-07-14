use super::raw::RawVector;
use std::ptr::{self};

/// Into Iterator structure for `Vector<T>`.
/// This allows for a consuming iterator to be built on this container
pub struct IntoIter<T> {
    pub(super) buf: RawVector<T>,
    pub(super) start: usize,
    pub(super) end: usize,
}

impl<T> IntoIter<T> {
    /// Read value from start and increment start by 1
    ///
    /// # Safety
    /// The buffer must be legal and start must be less than end
    /// for this function to be safe.
    unsafe fn read_and_incr(&mut self) -> T {
        // Safety:
        // This is safe as long as the buffer is legal
        // and the offset increments to a real location in the buffer
        let rslt = unsafe { ptr::read(self.buf.data.add(self.start).cast::<T>().as_ptr()) };
        self.start += 1;
        rslt
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        let to_drop = self.end - self.start;

        for _ in 0..to_drop {
            // Safety:
            // start less than end already verified by positive to_drop
            let _ = unsafe { self.read_and_incr() };
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            // Safety:
            // start less than end already verified by comparison with end
            Some(unsafe { self.read_and_incr() })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end - self.start;
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            self.end -= 1;
            // Safety:
            // This is safe as long as the buffer is legal
            // The previous check already validated that the index is legal
            Some(unsafe { ptr::read(self.buf.data.add(self.end).cast::<T>().as_ptr()) })
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.end - self.start
    }
}
