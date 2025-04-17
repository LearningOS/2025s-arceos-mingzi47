#[cfg(feature = "alloc")]
extern crate alloc;

use core::{
    hash::{self, Hash, Hasher},
    ops::Mul,
    u64::MAX,
};

use crate::misc::random;
#[cfg(feature = "alloc")]
use alloc::{vec, vec::Vec};

///
#[cfg(feature = "alloc")]
pub struct HashMap<K: Hash + Eq, V> {
    #[cfg(feature = "alloc")]
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
    seed: u128,
}

#[cfg(feature = "alloc")]
impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    ///
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "alloc")]
            buckets: Vec::new(),
            items: 0,
            seed: random(),
        }
    }

    ///
    pub fn insert(&mut self, key: K, value: V) {
        // cpp un_set ?
        if self.items == 0 || (self.buckets.len() < self.items * 3 / 5) {
            self.resize();
        }

        let mut hasher = RandomHasher::new(self.seed);
        key.hash(&mut hasher);
        let bucket = (hasher.finish() % self.buckets.len() as u64) as usize;

        self.items += 1;
        self.buckets[bucket].push((key, value));
    }

    ///
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut hasher = RandomHasher::new(self.seed);
        key.hash(&mut hasher);
        let bucket = (hasher.finish() % self.buckets.len() as u64) as usize;

        self.buckets[bucket]
            .iter()
            .find(|(ekey, _)| ekey == key)
            .map(|(_, evalue)| evalue)
    }

    // return (buckets.len, empty_b_num)
    pub fn test_empty_bucket_size(&self) -> (usize, usize) {
        let mut emptys: usize = 0;
        for b in self.buckets.iter() {
            emptys += if b.is_empty() { 1 } else { 0 }
        }

        (self.buckets.len(), emptys)
    }

    ///
    fn resize(&mut self) {
        let new_size = match self.buckets.len() {
            0 => 1,
            n => 2 * n,
        };

        let mut new_buckets = Vec::with_capacity(new_size);
        (0..new_size).for_each(|_| {
            new_buckets.push(Vec::new());
        });

        let seed = self.seed;
        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = RandomHasher::new(seed);
            key.hash(&mut hasher);
            let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, value));
        }

        core::mem::replace(&mut self.buckets, new_buckets);
    }

    ///
    pub fn iter(&self) -> Iter<K, V> {
        self.into_iter()
    }
}


#[cfg(feature = "alloc")]
impl<'a, K, V> IntoIterator for &'a HashMap<K, V>
where
    K: Hash + Eq,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: &self,
            bucket: 0,
            at: 0,
        }
    }
}

///
#[cfg(feature = "alloc")]
pub struct Iter<'a, K: Hash + Eq, V> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

#[cfg(feature = "alloc")]
impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Hash + Eq,
{
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => match bucket.get(self.at) {
                    Some((key, value)) => {
                        self.at += 1;
                        break Some((key, value));
                    }
                    None => {
                        self.bucket += 1;
                        self.at = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

pub struct RandomHasher {
    k1: u64,
    k2: u64,
    h1: u64,
    h2: u64,
}

impl RandomHasher {
    fn new(seed: u128) -> Self {
        Self {
            k1: (seed >> 8) as u64,
            k2: (seed & MAX as u128) as u64,
            h1: 0,
            h2: 0,
        }
    }
}

impl Hasher for RandomHasher {
    fn write(&mut self, bytes: &[u8]) {
        let mut tmp1 = 1;
        let mut tmp2 = 1;
        for &byte in bytes.iter() {
            self.h1 += tmp1 * byte as u64;
            tmp1 *= self.k1;

            self.h2 += tmp2 * byte as u64;
            tmp2 *= self.k2;
        }
    }

    fn finish(&self) -> u64 {
        self.h1 ^ self.h2 ^ self.k1 ^ self.k2
    }
}
