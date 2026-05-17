use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

pub struct BitBloomFilter {
    bits: Vec<u8>,
    bit_size: usize,
    num_hashes: usize,
}

impl BitBloomFilter {
    // n = item count
    // p = acceptable false-positive rate
    //
    // m = -(n * ln(p)) / ln(2)²   ->  optimal bit count
    // k = (m / n) * ln(2)         ->  optimal hash count
    pub fn new(bit_size: usize, num_hashes: usize) -> Self {
        Self {
            bits: vec![0; bit_size.div_ceil(8)],
            bit_size,
            num_hashes,
        }
    }

    fn hashes<T: Hash>(&self, item: &T) -> impl Iterator<Item = usize> + use<T> {
        let bit_size = self.bit_size as u64;
        let num_hashes = self.num_hashes;

        let mut h1 = FnvHasher::with_key(0);
        item.hash(&mut h1);
        let hash1 = h1.finish();

        let mut h2 = FnvHasher::with_key(1);
        item.hash(&mut h2);
        let hash2 = h2.finish();

        // double hashing: h(i) = h1 + i*h2 avoids needing k independent hash functions
        (0..num_hashes).map(move |i| {
            let combined = hash1.wrapping_add((i as u64).wrapping_mul(hash2));
            (combined % bit_size) as usize
        })
    }

    pub fn add<T: Hash>(&mut self, item: &T) {
        for idx in self.hashes(item) {
            let byte_idx = idx / 8;
            let bit_idx = idx % 8;
            self.bits[byte_idx] |= 1 << bit_idx;
        }
    }

    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        self.hashes(item).all(|idx| {
            let byte_idx = idx / 8;
            let bit_idx = idx % 8;
            (self.bits[byte_idx] & (1 << bit_idx)) != 0
        })
    }
}
