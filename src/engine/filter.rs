use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

// n = item count
// p = false-positive rate
// m = -(n * ln(p)) / ln(2)²
pub fn optimal_bit_size(n: usize, p: f64) -> usize {
    let n = n as f64;

    let num = -n * p.ln();

    let den = 2.0f64.ln().powi(2);

    (num / den).ceil() as usize
}

pub fn bit_indices(item: &str, hash_count: usize, bit_size: usize) -> impl Iterator<Item = usize> {
    let bit_size = bit_size as u64;

    let mut h1 = FnvHasher::with_key(0);
    item.hash(&mut h1);
    let hash1 = h1.finish();

    let mut h2 = FnvHasher::with_key(1);
    item.hash(&mut h2);
    let hash2 = h2.finish();

    (0..hash_count).map(move |i| {
        let combined = hash1.wrapping_add((i as u64).wrapping_mul(hash2));

        (combined % bit_size) as usize
    })
}

pub struct BloomFilter<'a> {
    bits: &'a [u8],
    bit_size: usize,
    hash_count: usize,
}

impl<'a> BloomFilter<'a> {
    // file layout: [bit_size: u64 le][hash_count: u64 le][bitfield: bit_size/8 bytes]
    pub fn new(bits: &'a [u8], bit_size: usize, hash_count: usize) -> Self {
        Self {
            bits,
            bit_size,
            hash_count,
        }
    }

    pub fn contains(&self, item: &str) -> bool {
        bit_indices(item, self.hash_count, self.bit_size)
            .all(|idx| (self.bits[idx / 8] & (1 << (idx % 8))) != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bits(urls: &[&str]) -> (Vec<u8>, usize) {
        let n = urls.len();

        let bit_size = optimal_bit_size(n, 0.01);

        let mut bits = vec![0u8; bit_size.div_ceil(8)];

        for url in urls {
            for idx in bit_indices(url, 7, bit_size) {
                bits[idx / 8] |= 1 << (idx % 8);
            }
        }
        (bits, bit_size)
    }

    #[test]
    fn test_contains_added_url() {
        let (bits, bit_size) = make_bits(&["https://evil.com"]);

        let filter = BloomFilter::new(&bits, bit_size, 7);

        assert!(filter.contains("https://evil.com"));
    }

    #[test]
    fn test_does_not_contain_unknown_url() {
        let (bits, bit_size) = make_bits(&["https://evil.com"]);

        let filter = BloomFilter::new(&bits, bit_size, 7);

        assert!(!filter.contains("https://safe.com"));
    }

    #[test]
    fn test_empty_filter_contains_nothing() {
        let bits = vec![0u8; 128];

        let filter = BloomFilter::new(&bits, 1024, 7);

        assert!(!filter.contains("https://evil.com"));
    }

    #[test]
    fn test_calculate_optimal_m() {
        let m = optimal_bit_size(78_000, 0.01);

        assert!(m >= 700_000 && m <= 800_000);
    }
}
