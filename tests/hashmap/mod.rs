//! This is the integration tests for the [`HashMap`]
mod iterator;

use std::{ops::Index, rc::Rc};

use rust_lib::hashmap::{HashMap, fixed_hasher::BuildFixedHasher};

#[test]
fn get_empty_returns_none() {
    let map = HashMap::<i32, i32>::new();
    assert_eq!(None, map.get(&1));
}

#[test]
fn insert_empty() {
    let mut map = HashMap::<i32, i32>::new();
    assert_eq!(None, map.insert(1, 2));
}

#[test]
fn inserted_value_returned() {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 2);

    assert_eq!(&2, map.get(&1).unwrap());
}

#[test]
fn insert_get_many() {
    let mut map = HashMap::<i32, i32>::new();
    for i in 0..100 {
        map.insert(i, i * 2);
    }

    for i in 0..100 {
        let expected = i * 2;
        assert_eq!(&expected, map.get(&i).unwrap());
    }
}

#[test]
fn insert_many_get_nonexistant() {
    let mut map = HashMap::<i32, i32>::new();
    for i in 0..100 {
        map.insert(i, i * 2);
    }

    assert_eq!(None, map.get(&101));
}

#[test]
fn insert_overwrite_returns_previous() {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(10, 100);
    assert_eq!(100, map.insert(10, 101).unwrap());

    assert_eq!(&101, map.get(&10).unwrap());
    assert_eq!(map.len(), 1);
}

#[test]
fn insert_increments_size() {
    let mut map = HashMap::<i32, i32>::new();
    for i in 0..100 {
        assert_eq!(i as usize, map.len());
        map.insert(i, i);
    }
    assert_eq!(100, map.len());
}

#[test]
fn insert_then_remove_successful() {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(0, 0);
    assert_eq!(0, map.remove(&0).unwrap());
}

#[test]
fn insert_then_remove_then_insert_successful() {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(0, 0);
    map.remove(&0);

    map.insert(0, 1);
    assert_eq!(&1, map.get(&0).unwrap());
}

#[test]
fn remove_empty_returns_none() {
    let mut map = HashMap::<i32, i32>::new();
    assert_eq!(None, map.remove(&0));
}

#[test]
fn remove_nonexistant_returns_none() {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 0);
    assert_eq!(None, map.remove(&0));
}

#[test]
fn insert_decrements_size() {
    let mut map = HashMap::<i32, i32>::new();
    for i in 0..100 {
        map.insert(i, i);
    }

    for i in (0..100).rev() {
        map.remove(&i);
        assert_eq!(i as usize, map.len());
    }
}

#[test]
fn insert_then_access_via_index_successful() {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 2);
    assert_eq!(2, map[&1]);
}

#[test]
#[should_panic]
fn index_into_empty_map_panics() {
    let map = HashMap::<i32, i32>::new();
    let _ = map.index(&1);
}

#[test]
#[should_panic]
fn index_into_wrong_key_panics() {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 2);
    let _ = map.index(&0);
}

#[test]
fn clone_empty_successful() {
    let map = HashMap::<i32, i32>::new();
    let _map2 = map.clone();
}

#[test]
fn clone_successful() {
    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 2);
    map.insert(2, 4);
    let map2 = map.clone();

    for kv in map {
        assert_eq!(map2.get(&kv.0).unwrap(), &kv.1);
    }
}

#[test]
fn clone_copies_complex_objects() {
    let rc = Rc::<u64>::new(10);
    let mut map = HashMap::<i32, Rc<u64>>::new();
    map.insert(1, rc.clone());
    map.insert(2, rc.clone());

    assert_eq!(Rc::strong_count(&rc), 3);

    let _map2 = map.clone();
    assert_eq!(Rc::strong_count(&rc), 5);
}

struct PanicClone(Rc<u64>);

impl Clone for PanicClone {
    fn clone(&self) -> Self {
        if Rc::strong_count(&self.0) > 5 {
            panic!("panicing after 5th clone");
        }
        PanicClone(self.0.clone())
    }
}

#[test]
#[should_panic]
fn clone_does_not_leak_or_double_drop_panicing_object() {
    let rc = Rc::<u64>::new(10);
    let mut map = HashMap::<i32, PanicClone>::new();
    for i in 0..3 {
        map.insert(i, PanicClone(rc.clone()));
    }

    let _map2 = map.clone();
}

#[test]
fn insert_remove_after_collison_successful() {
    let mut map = HashMap::<u64, u64, BuildFixedHasher>::new();
    map.insert(0, 0);
    map.insert(1, 1);
    map.insert(2, 2);

    map.remove(&0);
    assert_eq!(Some(1), map.insert(1, 2));
}
