#![no_std]
#![no_main]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;
#[cfg(feature = "alloc")]
extern crate alloc;


use std::{collections::HashMap, print, println, misc::random};


#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Running memory tests...");
    test_hashmap();
    // test_hashmap_get();
    #[cfg(feature = "alloc")]
    test_hashmap_random();
    println!("Memory tests run OK!");
}

fn test_hashmap() {
    const N: u32 = 50_000;
    let mut m = HashMap::new();
    for value in 0..N {
        let key = format!("key_{value}");
        m.insert(key, value);
    }
    for (k, v) in m.iter() {
        if let Some(k) = k.strip_prefix("key_") {
            assert_eq!(k.parse::<u32>().unwrap(), *v);
        }
    }
    println!("test_hashmap() OK!");
}

fn test_hashmap_get() {
    const N: u32 = 50_000;
    let mut m = HashMap::new();
    for value in 0..N {
        let key = format!("key_{value}");
        m.insert(key, value);
    }

    (0..N).rev().for_each(|i| {
        let key = format!("key_{i}");
        assert_eq!(m.get(&key), Some(i).as_ref());
        // if let Some(v) = m.get(&key) {
        //     println!("key = {}, value = {}", key, v);
        // }
    });
    assert_eq!(m.get(&format!("1111")), None);
    assert_eq!(m.get(&format!("2222")), None);
    assert_eq!(m.get(&format!("3333")), None);

    let (len, empty) = m.test_empty_bucket_size();
    println!("empty bucket test, tot len : {}, empty : {}", len, empty);
    println!("test_hashmap_get() OK!");
}

#[cfg(feature = "alloc")]
fn test_hashmap_random() {
    const N: u32 = 50_000;
    let mut m = HashMap::new();

    let mut v = Vec::new();
    for value in 0..N {
        let seed1 = random();
        let seed2 = random();
        let seed3 = random();
        let seed4 = random();
        let key = format!("{}{}{}{}", seed2, seed1, seed3, seed4);
        v.push((key.clone(), value));
        m.insert(key.clone(), value);
    }

    v.reverse();
    for (key, value) in v.iter() {
        assert_eq!(m.get(key), Some(value));
    }

    let (len, empty) = m.test_empty_bucket_size();
    println!("empty bucket test, tot len : {}, empty : {}", len, empty);
    println!("test_hashmap_random() OK!");
}
