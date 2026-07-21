//! Linear search algorithm

/// Search for target using a linear scan, returning the index of the first match
pub fn search<T: PartialEq>(slice: &[T], target: &T) -> Option<usize> {
    search_pred(slice, target, |lhs, rhs| lhs == rhs)
}

/// Search using a custom predicate, returning the index of the first item for which
/// `pred` returns true
pub fn search_pred<T, Pred>(slice: &[T], target: &T, pred: Pred) -> Option<usize>
where
    Pred: Fn(&T, &T) -> bool,
{
    for (i, val) in slice.iter().enumerate() {
        if pred(val, target) {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod test {

    use super::search;

    #[test]
    fn find_successful_unsorted() {
        let test = [0, 2, 1, 3, 4];
        let find = 1;

        assert_eq!(Some(2), search(&test, &find));
    }

    #[test]
    fn find_successful_sorted() {
        let test = [0, 1, 2, 3, 4];
        let find = 1;

        assert_eq!(Some(1), search(&test, &find));
    }

    #[test]
    fn find_unsuccessful_doesnt_exist() {
        let test = [0, 1, 2, 3, 4];
        let find = 5;

        assert_eq!(None, search(&test, &find));
    }

    #[test]
    fn find_unsuccessful_empty() {
        let test = [];
        let find = 5;

        assert_eq!(None, search(&test, &find));
    }
}
