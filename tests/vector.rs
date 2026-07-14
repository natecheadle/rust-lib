//! This is the integration tests for the `Vector`
use std::rc::Rc;

use rust_lib::vector::{MAX_GROWTH, Vector};

fn initialize_vector_of_5_rc() -> (Rc<i64>, Vector<Rc<i64>>) {
    let obj: Rc<i64> = Rc::new(1);
    let mut vec: Vector<Rc<i64>> = Vector::new();

    for _ in 0..5 {
        vec.push(obj.clone());
    }

    (obj, vec)
}

#[test]
fn create_succeeds() {
    let _: Vector<i64> = Vector::new();
}

#[test]
#[allow(clippy::vec_init_then_push)]
fn push_one_succeeds() {
    let mut vec: Vector<i64> = Vector::new();
    vec.push(123);
}

#[test]
fn push_one_hundred_succeeds() {
    let mut vec: Vector<i64> = Vector::new();
    for i in 1..100 {
        vec.push(i);
    }

    assert_eq!(vec.len(), 99);
}

#[test]
fn push_pop_one_hundred_succeeds() {
    let obj: Rc<i64> = Rc::new(1);
    let mut vec: Vector<Rc<i64>> = Vector::new();
    for i in 1..100 {
        vec.push(obj.clone());
        assert_eq!(Rc::strong_count(&obj), i + 1);
    }

    for i in (1..100).rev() {
        let popped = vec.pop().unwrap();
        assert_eq!(*popped, 1);
        assert_eq!(Rc::strong_count(&obj), i + 1);
    }

    assert_eq!(None, vec.pop());
}

#[test]
fn push_100_drop_clears() {
    let obj: Rc<i64> = Rc::new(1);
    {
        let mut vec: Vector<Rc<i64>> = Vector::new();
        for i in 1..100 {
            vec.push(obj.clone());
            assert_eq!(Rc::strong_count(&obj), i + 1);
        }
        assert_eq!(Rc::strong_count(&obj), 100);
    }

    assert_eq!(Rc::strong_count(&obj), 1);
}

#[test]
fn push_past_max_growth() {
    let mut vec: Vector<usize> = Vector::new();
    for i in 0..(MAX_GROWTH + 100) {
        vec.push(i);
    }
    assert_eq!(vec.len(), MAX_GROWTH + 100);
}

#[test]
fn push_pop_arbitray_properly_drops() {
    let obj: Rc<i64> = Rc::new(1);
    let mut vec: Vector<Rc<i64>> = Vector::new();

    for _ in 0..10 {
        vec.push(obj.clone());
    }
    assert_eq!(Rc::strong_count(&obj), 11);

    for _ in 0..5 {
        vec.pop();
    }
    assert_eq!(Rc::strong_count(&obj), 6);

    for _ in 0..10 {
        vec.push(obj.clone());
    }
    assert_eq!(Rc::strong_count(&obj), 16);

    for _ in 0..5 {
        vec.pop();
    }
    assert_eq!(Rc::strong_count(&obj), 11);
}

#[test]
fn push_pop_arbitray_properly_orders() {
    let mut vec: Vector<i64> = Vector::new();

    for i in 0..10 {
        vec.push(i);
    }

    for i in (5..10).rev() {
        let popped = vec.pop().unwrap();
        assert_eq!(popped, i);
    }

    for i in 6..15 {
        vec.push(i);
    }

    for i in (10..15).rev() {
        let popped = vec.pop().unwrap();
        assert_eq!(popped, i);
    }
}

#[test]
fn is_empty_when_size_0() {
    let mut vec: Vector<usize> = Vector::new();
    assert!(vec.is_empty());
    vec.push(1);
    assert!(!vec.is_empty());
    vec.pop();
    assert!(vec.is_empty());
}

#[test]
fn clear_removes_and_drops_all_items() {
    let obj: Rc<i64> = Rc::new(1);
    let mut vec: Vector<Rc<i64>> = Vector::new();

    for _ in 0..10 {
        vec.push(obj.clone());
    }
    assert_eq!(Rc::strong_count(&obj), 11);

    vec.clear();
    assert!(vec.is_empty());
    assert_eq!(Rc::strong_count(&obj), 1);
}

#[test]
fn reserve_only_increases_memory_size() {
    let mut vec: Vector<i64> = Vector::new();
    vec.reserve(100);

    assert_eq!(vec.capacity(), 100);
    assert!(vec.is_empty());
}

#[test]
fn reserve_adds_to_memory_size() {
    let mut vec: Vector<i64> = Vector::new();
    vec.reserve(100);
    vec.push(1);
    vec.reserve(100);

    assert_eq!(vec.capacity(), 101);
}

#[test]
fn reserve_total_only_increases_memory_size() {
    let mut vec: Vector<i64> = Vector::new();
    vec.reserve_total(100);

    assert_eq!(vec.capacity(), 100);
    assert!(vec.is_empty());
}

#[test]
fn reserve_total_increases_only_to_memory_size() {
    let mut vec: Vector<i64> = Vector::new();
    vec.reserve_total(100);
    vec.reserve_total(100);

    assert_eq!(vec.capacity(), 100);
}

#[test]
fn accessing_via_index_returns_correct_value() {
    let mut vec: Vector<usize> = Vector::new();
    for i in 0..10 {
        vec.push(i);
        assert_eq!(vec[i], i);
    }
}

#[test]
fn can_mutate_value_in_vector_via_index() {
    let mut vec: Vector<usize> = Vector::new();
    for i in 0..10 {
        vec.push(i);
        vec[i] *= 10;
        assert_eq!(vec[i], i * 10);
    }
}

#[test]
#[should_panic = "index out of bounds: the len is 10 but the index is 11"]
fn access_index_out_of_range_panics() {
    let mut vec: Vector<usize> = Vector::new();
    for i in 0..10 {
        vec.push(i);
    }

    let _ = vec[11];
}

#[test]
#[should_panic = "index out of bounds: the len is 10 but the index is 11"]
fn access_mut_index_out_of_rane_panics() {
    let mut vec: Vector<usize> = Vector::new();
    for i in 0..10 {
        vec.push(i);
    }

    vec[11] = 11;
}

#[test]
fn clone_deep_copies() {
    let (obj, vec) = initialize_vector_of_5_rc();

    assert_eq!(Rc::strong_count(&obj), 6);

    let _vec2 = vec.clone();
    assert_eq!(Rc::strong_count(&obj), 11);
}

#[test]
fn deref_gives_slice_methods() {
    let mut vec: Vector<i32> = Vector::new();
    for i in 0..5 {
        vec.push(i);
    }
    assert_eq!(vec.first(), Some(&0)); // first() is a [T] method, not yours
    assert_eq!(vec.last(), Some(&4));
    assert!(vec.contains(&3));
}

#[test]
fn deref_slice_len_matches_size_not_capacity() {
    let mut vec: Vector<i32> = Vector::new();
    vec.reserve(100);
    for i in 0..5 {
        vec.push(i);
    }
    assert_eq!(vec.len(), 5);
    assert_eq!((*vec).len(), 5);
}

#[test]
fn deref_mut_allows_slice_mutation() {
    let mut vec: Vector<i32> = Vector::new();
    for i in 0..5 {
        vec.push(i);
    }
    vec.sort_by(|a, b| b.cmp(a));
    assert_eq!(vec[0], 4);
    assert_eq!(vec[4], 0);
}

#[test]
fn deref_on_empty_vector_gives_empty_slice() {
    let vec: Vector<i32> = Vector::new();
    assert!(vec.is_empty());
    assert_eq!(vec.first(), None);
}

#[test]
fn remove_works_on_first_item() {
    let mut vec: Vector<i32> = Vector::new();
    for i in 0..5 {
        vec.push(i);
    }

    let rslt = vec.remove(0);
    assert_eq!(rslt, 0);
    assert_eq!(vec.len(), 4);
    assert_eq!(vec[0], 1);
}

#[test]
fn remove_works_on_middle_item() {
    let mut vec: Vector<i32> = Vector::new();
    for i in 0..5 {
        vec.push(i);
    }

    let rslt = vec.remove(2);
    assert_eq!(rslt, 2);
    assert_eq!(vec.len(), 4);
    assert_eq!(vec[0], 0);
    assert_eq!(vec[2], 3);
}

#[test]
fn remove_works_on_last_item() {
    let mut vec: Vector<i32> = Vector::new();
    for i in 0..5 {
        vec.push(i);
    }

    let rslt = vec.remove(4);
    assert_eq!(rslt, 4);
    assert_eq!(vec.len(), 4);
    assert_eq!(vec[3], 3);
}

#[test]
fn insert_works_on_first_item() {
    let mut vec: Vector<i32> = Vector::new();
    for i in 0..5 {
        vec.push(i);
    }

    vec.insert(0, 5);
    assert_eq!(vec.len(), 6);
    assert_eq!(vec[0], 5);
    assert_eq!(vec[1], 0);
}

#[test]
fn insert_works_on_middle_item() {
    let mut vec: Vector<i32> = Vector::new();
    for i in 0..5 {
        vec.push(i);
    }

    vec.insert(2, 5);
    assert_eq!(vec.len(), 6);
    assert_eq!(vec[2], 5);
    assert_eq!(vec[1], 1);
    assert_eq!(vec[3], 2);
}

#[test]
fn insert_works_on_last_item() {
    let mut vec: Vector<i32> = Vector::new();
    for i in 0..5 {
        vec.push(i);
    }

    vec.insert(5, 5);
    assert_eq!(vec.len(), 6);
    assert_eq!(vec[5], 5);
    assert_eq!(vec[4], 4);
}

#[test]
fn insert_properly_moves_items() {
    let (obj, mut vec) = initialize_vector_of_5_rc();

    vec.insert(2, obj.clone());
    assert_eq!(vec.len(), 6);
    assert_eq!(Rc::strong_count(&obj), 7);
}

#[test]
fn remove_properly_drops_items() {
    let (obj, mut vec) = initialize_vector_of_5_rc();

    {
        let obj2 = vec.remove(2);
        assert_eq!(obj2, obj);
        assert_eq!(vec.len(), 4);
        assert_eq!(Rc::strong_count(&obj), 6);
    }
    assert_eq!(Rc::strong_count(&obj), 5);
}

#[test]
fn partial_eq_compares_contents() {
    let mut a: Vector<i32> = Vector::new();
    let mut b: Vector<i32> = Vector::new();
    for i in 0..5 {
        a.push(i);
        b.push(i);
    }
    assert_eq!(a, b);

    b.push(99);
    assert_ne!(a, b);
}

#[test]
fn debug_formats_like_slice() {
    let mut vec: Vector<i32> = Vector::new();
    for i in 0..3 {
        vec.push(i);
    }
    assert_eq!(format!("{vec:?}"), "[0, 1, 2]");
}

#[test]
fn push_and_pop_zst_works() {
    let mut vec: Vector<()> = Vector::new();
    for i in 0..5 {
        assert_eq!(vec.len(), i);
        assert_eq!(vec.capacity(), 0);
        vec.push(());
    }
    assert_eq!(vec.capacity(), 0);
    assert_eq!(vec.len(), 5);

    for i in (0..5).rev() {
        let rslt = vec.pop();
        assert_eq!(vec.len(), i);
        assert_eq!(vec.capacity(), 0);
        assert!(rslt.is_some());
    }

    assert_eq!(vec.capacity(), 0);
    assert!(vec.pop().is_none());
}

#[test]
fn index_zst_works() {
    let mut vec: Vector<()> = Vector::new();
    for _ in 0..5 {
        vec.push(());
    }

    let () = vec[2];
    vec[2] = ();
}

#[test]
fn inter_iter_zst_works() {
    let mut vec: Vector<()> = Vector::new();
    for _ in 0..5 {
        vec.push(());
    }

    for () in vec {}
}
