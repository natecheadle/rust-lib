use rust_lib::{hashmap::HashMap, vector::Vector};

use std::{
    hash::{BuildHasher, Hasher},
    rc::Rc,
};

pub struct DroppingHasher(Rc<u64>);

impl Hasher for DroppingHasher {
    fn finish(&self) -> u64 {
        *self.0
    }

    fn write(&mut self, _: &[u8]) {}

    fn write_u64(&mut self, _: u64) {}
}

#[derive(Default)]
pub struct BuildDroppingHasher(Rc<u64>);

impl BuildDroppingHasher {
    fn new(rc: Rc<u64>) -> Self {
        BuildDroppingHasher { 0: rc }
    }
}

impl BuildHasher for BuildDroppingHasher {
    type Hasher = DroppingHasher;

    fn build_hasher(&self) -> Self::Hasher {
        DroppingHasher { 0: self.0.clone() }
    }
}

#[test]
fn iterator_returns_all_values() {
    let mut vector = Vector::<(i32, i32)>::default();
    let mut map = HashMap::<i32, i32>::new();
    for i in 0..100 {
        map.insert(i, i);
        vector.push((i, i));
    }

    let mut i = 0;
    for val in map.iter() {
        i += 1;
        assert!(
            vector
                .binary_search(&(val.0.clone(), val.1.clone()))
                .is_ok()
        );
    }

    assert_eq!(i, 100);
}

#[test]
fn into_iterator_returns_all_values() {
    let mut vector = Vector::<(i32, i32)>::default();
    let mut map = HashMap::<i32, i32>::new();
    for i in 0..100 {
        map.insert(i, i);
        vector.push((i, i));
    }

    let mut i = 0;
    for val in map {
        assert!(vector.binary_search(&val).is_ok());
        i += 1;
    }
    assert_eq!(i, 100);
}

#[test]
fn mutable_iterator_changes_all_values() {
    let mut map = HashMap::<i32, i32>::new();
    for i in 0..100 {
        map.insert(i, i);
    }

    for val in map.iter_mut() {
        *val.1 = val.0 * 2;
    }

    for i in 0..100 {
        assert_eq!(map.get(&i).unwrap(), &(i * 2));
    }
}

#[test]
fn into_iterator_drops_hasher() {
    let rc = Rc::<u64>::new(10);
    {
        let mut map = HashMap::<i32, i32, BuildDroppingHasher>::new_with_hasher(
            BuildDroppingHasher::new(rc.clone()),
        );
        assert_eq!(Rc::strong_count(&rc), 2);
        map.insert(1, 1);
        for _ in map {}

        assert_eq!(Rc::strong_count(&rc), 1);
    }
}
