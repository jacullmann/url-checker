use crate::engine::{config, filter::BloomFilter, normalization::normalize_url};
use memmap2::Mmap;
use std::{fs::File, io::Read};

pub fn handle_check(raw_url: &str) {
    if !std::path::Path::new(config::FILTER_PATH).exists() {
        println!("couldn't find prepared data. please run 'build' first");

        return;
    }

    let mut file = File::open(config::FILTER_PATH).expect("failed to open filter file");

    let mut u64_buf = [0u8; 8];

    file.read_exact(&mut u64_buf)
        .expect("malformed filter file header");

    let bit_size = u64::from_le_bytes(u64_buf) as usize;

    file.read_exact(&mut u64_buf)
        .expect("malformed filter file header");

    let num_hashes = u64::from_le_bytes(u64_buf) as usize;

    let mmap = unsafe { Mmap::map(&file).expect("Failed to initialize memory-mapped data lookup") };

    // skip the 16-byte header (bit_size + num_hashes) to get to the bitfield
    let bit_slice = &mmap[16..];

    let filter = BloomFilter::new(bit_slice, bit_size, num_hashes);

    let normalized = normalize_url(raw_url);

    if filter.contains(&normalized) {
        println!("POSSIBLY MALICIOUS");
    } else {
        println!("NOT FOUND – SAFE");
    }
}
