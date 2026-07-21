//! Binary search algorithm

use std::cmp::Ordering;

/// Convenience wrapper around [`search_by`] that uses [`Ord::cmp`] as the function
///
/// # Errors
/// If the there is no equal value this function returns an error with the index location
/// to insert the value at.
pub fn search<T: Ord>(slice: &[T], target: &T) -> Result<usize, usize> {
    search_by(slice, target, Ord::cmp)
}

/// Binary search a sorted slice using a custom comparison function, mirroring
/// `std::slice::binary_search_by`
///
/// # Errors
/// If the there is no equal value this function returns an error with the index location
/// to insert the value at.
///
/// # Notes
/// Might match in middle of a group of equal values.
pub fn search_by<T, F>(slice: &[T], target: &T, f: F) -> Result<usize, usize>
where
    F: Fn(&T, &T) -> Ordering,
{
    let mut lo = 0;
    let mut hi = slice.len();
    while hi > lo {
        let middle = (hi - lo) / 2 + lo;
        match f(&slice[middle], target) {
            Ordering::Equal => return Ok(middle),
            Ordering::Less => lo = middle + 1,
            Ordering::Greater => hi = middle,
        }
    }

    Err(lo)
}

/// Find the lower boundary using a binary search algorithm.
///
/// This is a convenience wrapper around [`lower_bound_by`] that uses
/// an Ordinal types `<` operator
pub fn lower_bound<T: Ord>(slice: &[T], target: &T) -> usize {
    lower_bound_by(slice, target, |lhs, rhs| -> bool { lhs < rhs })
}

/// Find the lower boundary using a binary search algorithm.
///
/// # Requirements
/// `pred` must implement a strict "less than" relationship. The same
/// function can be used by both [`lower_bound_by`] and [`upper_bound_by`]
pub fn lower_bound_by<T, Pred>(slice: &[T], target: &T, f: Pred) -> usize
where
    Pred: Fn(&T, &T) -> bool,
{
    let mut lo = 0;
    let mut hi = slice.len();
    while hi > lo {
        let middle = (hi - lo) / 2 + lo;
        if f(&slice[middle], target) {
            lo = middle + 1;
        } else {
            hi = middle;
        }
    }

    lo
}

/// Find the upper boundary using a binary search algorithm
///
/// This is a convenience wrapper around [`upper_bound_by`] that uses
/// an Ordinal types `<` operator
pub fn upper_bound<T: Ord>(slice: &[T], target: &T) -> usize {
    upper_bound_by(slice, target, |lhs, rhs| -> bool { lhs < rhs })
}

/// Find the upper boundary using a binary search algorithm
///
/// # Requirements
/// `pred` must implement a strict "less than" relationship. The same
/// function can be used by both [`lower_bound_by`] and [`upper_bound_by`]
pub fn upper_bound_by<T, F>(slice: &[T], target: &T, f: F) -> usize
where
    F: Fn(&T, &T) -> bool,
{
    let mut lo = 0;
    let mut hi = slice.len();
    while hi > lo {
        let middle = (hi - lo) / 2 + lo;
        if f(target, &slice[middle]) {
            hi = middle;
        } else {
            lo = middle + 1;
        }
    }

    lo
}

#[cfg(test)]
mod test {
    use crate::vector::Vector;

    use super::{lower_bound, search, upper_bound};

    #[test]
    fn empty_returns_zero_err() {
        let test = [];
        let find = 1;
        assert_eq!(Err(0), search(&test, &find));
    }

    #[test]
    fn size_1_match_returns_zero_ok() {
        let test = [1];
        let find = 1;
        assert_eq!(Ok(0), search(&test, &find));
    }

    #[test]
    fn size_1_less_than_returns_zero_err() {
        let test = [1];
        let find = 0;
        assert_eq!(Err(0), search(&test, &find));
    }

    #[test]
    fn size_1_greater_than_returns_one_err() {
        let test = [1];
        let find = 2;
        assert_eq!(Err(1), search(&test, &find));
    }

    #[test]
    fn search_exact_match_return_ok() {
        let test = [0, 10, 20, 30, 40, 50];
        let find = 20;
        assert_eq!(Ok(2), search(&test, &find));
    }

    #[test]
    fn search_not_exact_match_return_err_insert_index() {
        let test = [0, 10, 20, 30, 40, 50];
        let find = 25;
        assert_eq!(Err(3), search(&test, &find));
    }

    #[test]
    fn search_not_exact_match_return_err_insert_index_end() {
        let test = [0, 10, 20, 30, 40, 50];
        let find = 55;
        assert_eq!(Err(6), search(&test, &find));
    }

    #[test]
    fn search_not_exact_match_return_err_insert_index_begin() {
        let test = [0, 10, 20, 30, 40, 50];
        let find = -1;
        assert_eq!(Err(0), search(&test, &find));
    }

    #[test]
    fn search_multiples_middle() {
        let test = [0, 10, 20, 20, 20, 50, 60];
        let find = 20;
        assert_eq!(Ok(3), search(&test, &find));
    }

    #[test]
    fn search_multiples_not_middle() {
        let test = [0, 20, 20, 30, 40, 50, 60];
        let find = 20;
        assert_eq!(Ok(1), search(&test, &find));
    }

    #[test]
    fn search_lower_bound_has_exact_match() {
        let test = [0, 20, 20, 30, 40, 50, 60];
        let find = 20;
        assert_eq!(1, lower_bound(&test, &find));
    }

    #[test]
    fn search_lower_bound_has_no_exact_match() {
        let test = [0, 20, 20, 30, 40, 50, 60];
        let find = 10;
        assert_eq!(1, lower_bound(&test, &find));
    }

    #[test]
    fn search_upper_bound_has_no_exact_match() {
        let test = [0, 20, 20, 30, 40, 50, 60];
        let find = 25;
        assert_eq!(3, upper_bound(&test, &find));
    }

    #[test]
    fn search_upper_bound_has_exact_match() {
        let test = [0, 20, 20, 30, 40, 50, 60];
        let find = 20;
        assert_eq!(3, upper_bound(&test, &find));
    }

    const NUMBER_TEST_ITEMS: i32 = 100;

    fn create_sorted_vect() -> Vector<i32> {
        let mut test: Vector<i32> = Vector::new();
        test.reserve(NUMBER_TEST_ITEMS as usize);
        for _ in 0..NUMBER_TEST_ITEMS {
            test.push(rand::random_range(0..NUMBER_TEST_ITEMS));
        }

        test.sort_unstable();

        test
    }

    #[test]
    fn random_sorted_data_search() {
        for _ in 0..NUMBER_TEST_ITEMS {
            let test = create_sorted_vect();
            let target = rand::random_range(0..NUMBER_TEST_ITEMS);

            let rslt_std = test.binary_search(&target);
            if let Ok(idx) = rslt_std {
                assert_eq!(test[idx], test[search(&test, &target).unwrap()]);
            } else {
                assert_eq!(rslt_std, search(&test, &target));
            }
        }
    }

    #[test]
    fn random_sorted_data_successful_lower_bound() {
        for _ in 0..NUMBER_TEST_ITEMS {
            let test = create_sorted_vect();
            let target = rand::random_range(0..NUMBER_TEST_ITEMS);

            assert_eq!(
                test.partition_point(|x| x < &target),
                lower_bound(&test, &target)
            );
        }
    }

    #[test]
    fn random_sorted_data_successful_upper_bound() {
        for _ in 0..NUMBER_TEST_ITEMS {
            let test = create_sorted_vect();
            let target = rand::random_range(0..NUMBER_TEST_ITEMS);

            assert_eq!(
                test.partition_point(|x| x <= &target),
                upper_bound(&test, &target)
            );
        }
    }
}
