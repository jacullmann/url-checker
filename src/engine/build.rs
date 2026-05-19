use crate::engine::{
    config,
    filter::{bit_indices, optimal_bit_size},
    normalization::normalize_url,
};
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
};

pub fn handle_build() {
    println!("downloading...");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("failed to create downloader");

    // the dataset is streamed twice: first to count elements for optimal filter sizing,
    // then to populate the filter. Counting upfront avoids resizing the memory-mapped file.
    let response = match client.get(config::DATA_URL).send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: failed to fetch data: {e}");
            return;
        }
    };

    let mut reader = BufReader::new(response);

    let mut line = String::new();

    let mut element_count = 0;

    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        let trimmed = line.trim();

        if !trimmed.starts_with('#') && !trimmed.is_empty() {
            element_count += 1;
        }

        line.clear();
    }

    if element_count == 0 {
        eprintln!("no valid data found");

        return;
    }

    let bit_size = optimal_bit_size(element_count, config::TARGET_FALSE_POSITIVE_RATE);

    println!("building...");

    let byte_size = bit_size.div_ceil(8);

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(config::FILTER_PATH)
        .expect("failed to create file");

    file.write_all(&(bit_size as u64).to_le_bytes())
        .expect("failed to write bit_size");

    file.write_all(&(config::HASH_COUNT as u64).to_le_bytes())
        .expect("failed to write hash_count");

    // pre-allocate the full file size before mapping — mmap requires the file to be
    // at least as large as the mapped region.
    file.set_len((16 + byte_size) as u64)
        .expect("failed to allocate file size");

    let mut mmap =
        unsafe { memmap2::MmapMut::map_mut(&file).expect("failed to map file for writing") };

    let bit_slice = &mut mmap[16..];

    let response = match client.get(config::DATA_URL).send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: failed to fetch data: {e}");
            return;
        }
    };

    let mut reader = BufReader::new(response);

    let mut processed = 0;

    while reader.read_line(&mut line).unwrap_or(0) > 0 {
        let trimmed = line.trim();

        if !trimmed.starts_with('#') && !trimmed.is_empty() {
            let normalized = normalize_url(trimmed);

            for idx in bit_indices(&normalized, config::HASH_COUNT, bit_size) {
                bit_slice[idx / 8] |= 1 << (idx % 8);
            }

            processed += 1;

            if processed % 25_000 == 0 || processed == element_count {
                print!("\rpreparing data [{}/{}]", processed, element_count);

                std::io::stdout().flush().unwrap();
            }
        }
        line.clear();
    }

    mmap.flush().expect("failed to flush memory map");

    println!();

    println!("done");
}
