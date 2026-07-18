//! Bubble sorting algorithm

use super::helper::default_pred;

/// Sort slice using the bubble sort algorithm
pub fn sort<T: Ord>(slice: &mut [T]) {
    sort_pred(slice, default_pred);
}

/// Sort slice using a custom defined predicate
pub fn sort_pred<T, Pred>(slice: &mut [T], pred: Pred)
where
    Pred: Fn(&T, &T) -> bool,
{
    let length = slice.len();
    if length <= 1 {
        return;
    }

    for i in 0..length {
        for j in 0..length - i - 1 {
            if pred(&slice[j + 1], &slice[j]) {
                slice.swap(j, j + 1);
            }
        }
    }
}

#[cfg(test)]
mod test {

    use super::sort;
    use crate::vector::Vector;

    #[test]
    fn sort_numbers() {
        let mut to_sort: Vector<i32> = Vector::new();
        for val in [3, 1, 5, 6] {
            to_sort.push(val);
        }

        sort(&mut to_sort);

        assert_eq!([1, 3, 5, 6], *to_sort);
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
}
