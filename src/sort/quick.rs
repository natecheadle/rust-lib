//! Quick sort sorting algorithm module

use super::helper::default_pred;

#[derive(Debug, PartialEq)]
struct Partition {
    pub upper: usize,
    pub lower: usize,
}

fn three_sample_median<T, Pred>(slice: &mut [T], pred: &Pred) -> usize
where
    Pred: Fn(&T, &T) -> bool,
{
    debug_assert!(
        slice.len() > 3,
        "Slice length should be greater than 3 or this has many no-ops"
    );

    let mut options: [usize; 3] = [0, slice.len() >> 1, slice.len() - 1];
    super::bubble::sort_pred(&mut options, |a, b| pred(&slice[*a], &slice[*b]));
    options[1]
}

// Partition the slice using the dutch flag method
fn partition<T, Pred>(slice: &mut [T], pred: &Pred) -> Partition
where
    Pred: Fn(&T, &T) -> bool,
{
    let median_idx = three_sample_median(slice, pred);
    let mut pivot_idx = slice.len() >> 1;
    slice.swap(median_idx, pivot_idx);

    let mut less_than: usize = 0;
    let mut greater_than: usize = slice.len() - 1;
    let mut equals: usize = 0;

    while equals <= greater_than {
        if pred(&slice[equals], &slice[pivot_idx]) {
            slice.swap(equals, less_than);
            // pivot_idx never equals `equals` because it is "less_than"
            if pivot_idx == less_than {
                pivot_idx = equals;
            }
            less_than += 1;
            equals += 1;
        } else if pred(&slice[pivot_idx], &slice[equals]) {
            slice.swap(equals, greater_than);
            // pivot_idx never equals `equals` because it is "less than"
            if pivot_idx == greater_than {
                pivot_idx = equals;
            }
            greater_than -= 1;
        } else {
            equals += 1;
        }
    }

    Partition {
        upper: greater_than,
        lower: less_than,
    }
}

fn sort_pred_priv<T, Pred>(slice: &mut [T], pred: &Pred)
where
    Pred: Fn(&T, &T) -> bool,
{
    let length = slice.len();
    if length <= 3 {
        super::bubble::sort_pred(slice, pred);
        return;
    }

    let partitions = partition(slice, pred);

    if partitions.lower >= 1 {
        sort_pred_priv(&mut slice[0..partitions.lower], pred);
    }
    if partitions.upper < length - 1 {
        sort_pred_priv(&mut slice[partitions.upper + 1..length], pred);
    }
}

/// Use the quick-sort algorithm to sort a slice
pub fn sort<T: Ord>(slice: &mut [T]) {
    sort_pred_priv(slice, &default_pred);
}

/// Use the quick-sort algorithm to sort a slice
pub fn sort_pred<T, Pred>(slice: &mut [T], pred: Pred)
where
    Pred: Fn(&T, &T) -> bool,
{
    sort_pred_priv(slice, &pred);
}

#[cfg(test)]
mod test {
    use super::{Partition, partition, sort};
    use crate::{sort::helper::default_pred, vector::Vector};

    const PERMUTATIONS_OF_3: [[i32; 3]; 6] = [
        [1, 2, 3],
        [1, 3, 2],
        [2, 1, 3],
        [2, 3, 1],
        [3, 1, 2],
        [3, 2, 1],
    ];

    #[test]
    fn partition_dutch_correct_partitions_no_multiples() {
        let mut to_sort: Vector<i32> = Vector::new();
        for val in [3, 1, 5, 6, 2, 4] {
            to_sort.push(val);
        }

        let pivot_id = partition(&mut to_sort, &default_pred);

        assert_eq!(pivot_id, Partition { upper: 3, lower: 3 });
    }

    #[test]
    fn partition_dutch_correct_partitions_with_multiples() {
        let mut to_sort: Vector<i32> = Vector::new();
        for val in [3, 1, 5, 5, 2, 5] {
            to_sort.push(val);
        }

        let pivot_id = partition(&mut to_sort, &default_pred);

        assert_eq!(pivot_id, Partition { upper: 5, lower: 3 });
    }

    #[test]
    fn sort_numbers() {
        let mut to_sort: Vector<i32> = Vector::new();
        for val in [3, 4, 5, 1, 2, 6] {
            to_sort.push(val);
        }

        sort(&mut to_sort);

        assert_eq!([1, 2, 3, 4, 5, 6], *to_sort);
    }

    #[test]
    fn sort_numbers_random() {
        const LENGTH: usize = 100;

        for _ in 0..LENGTH {
            let mut to_sort: Vector<usize> = Vector::new();
            to_sort.reserve(LENGTH);
            for _ in 0..LENGTH {
                to_sort.push(rand::random_range(0..LENGTH));
            }

            let mut clone_to_sort = to_sort.clone();

            clone_to_sort.sort_unstable();
            sort(&mut to_sort);

            assert_eq!(to_sort, clone_to_sort);
        }
    }

    #[test]
    fn sort_numbers_multiples() {
        let mut to_sort: Vector<i32> = Vector::new();
        for val in [3, 3, 3, 1, 2, 6] {
            to_sort.push(val);
        }

        sort(&mut to_sort);

        assert_eq!([1, 2, 3, 3, 3, 6], *to_sort);
    }

    #[test]
    fn sort_small() {
        let mut to_sort: Vector<i32> = Vector::new();
        for perms in PERMUTATIONS_OF_3 {
            for val in perms {
                to_sort.push(val);
            }

            sort(&mut to_sort);

            assert_eq!([1, 2, 3], *to_sort);
            to_sort.clear();
        }
    }
}
