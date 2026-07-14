//! Inegration tests for `IntoIter` for `Vector`

use rust_lib::vector::Vector;
use std::rc::Rc;

fn initialize_vector_of_5_rc() -> (Rc<i64>, Vector<Rc<i64>>) {
    let obj: Rc<i64> = Rc::new(1);
    let mut vec: Vector<Rc<i64>> = Vector::new();

    for _ in 0..5 {
        vec.push(obj.clone());
    }

    (obj, vec)
}

#[test]
fn into_iter_can_clear_all_values() {
    let (obj, vec) = initialize_vector_of_5_rc();

    for _ in vec {}

    assert_eq!(Rc::strong_count(&obj), 1);
}

#[test]
fn into_iter_incrementally_clears_values() {
    let (obj, vec) = initialize_vector_of_5_rc();
    let mut into_iter = vec.into_iter();
    {
        let _ = into_iter.next();
    }

    assert_eq!(Rc::strong_count(&obj), 5);
    drop(into_iter);
    assert_eq!(Rc::strong_count(&obj), 1);
}

#[test]
fn reverse_into_iter_can_clear_all_values() {
    let (obj, vec) = initialize_vector_of_5_rc();

    for _ in vec.into_iter().rev() {}

    assert_eq!(Rc::strong_count(&obj), 1);
}

#[test]
fn reverse_into_iter_incrementally_clears_values() {
    let (obj, vec) = initialize_vector_of_5_rc();
    let mut into_iter = vec.into_iter().rev();
    {
        let _ = into_iter.next();
    }

    assert_eq!(Rc::strong_count(&obj), 5);
    drop(into_iter);
    assert_eq!(Rc::strong_count(&obj), 1);
}

#[test]
fn into_iter_mixed_front_and_back_drains_exactly_once() {
    let (obj, vec) = initialize_vector_of_5_rc();
    let mut into_iter = vec.into_iter();

    let _ = into_iter.next();
    let _ = into_iter.next_back();
    assert_eq!(Rc::strong_count(&obj), 4);

    drop(into_iter);
    assert_eq!(Rc::strong_count(&obj), 1);
}
