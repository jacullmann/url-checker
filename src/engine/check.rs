use crate::engine::{config, filter::BloomFilter, normalization::normalize_url};
use anyhow::{Context, Result};
use memmap2::Mmap;
use std::{fs::File, io::Read};

pub fn handle_check(raw_url: &str) -> Result<()> {
    if !std::path::Path::new(config::FILTER_PATH).exists() {
        println!("couldn't find prepared data. please run 'prepare' first");
        return Ok(());
    }

    let mut file = File::open(config::FILTER_PATH)
        .with_context(|| format!("failed to open filter file at {}", config::FILTER_PATH))?;

    let mut u64_buf = [0u8; 8];

    file.read_exact(&mut u64_buf)
        .context("malformed filter file: could not read bit_size from header")?;
    let bit_size = u64::from_le_bytes(u64_buf) as usize;

    file.read_exact(&mut u64_buf)
        .context("malformed filter file: could not read hash_count from header")?;
    let hash_count = u64::from_le_bytes(u64_buf) as usize;

    let mmap = unsafe { Mmap::map(&file).context("failed to memory-map filter file")? };

    // skip the 16-byte header (bit_size + hash_count) to get to the bitfield
    let bit_slice = &mmap[16..];

    let filter = BloomFilter::new(bit_slice, bit_size, hash_count);

    let normalized = normalize_url(raw_url);

    if filter.contains(&normalized) {
        println!("POSSIBLY MALICIOUS");
    } else {
        println!("NOT FOUND – SAFE");
    }

    Ok(())
}
