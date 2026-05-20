use crate::engine::{
    config,
    filter::{bit_indices, optimal_bit_size},
    normalization::normalize_url,
};
use anyhow::{Context, Result, bail};
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
};

pub fn handle_prepare() -> Result<()> {
    println!("downloading...");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .context("failed to create HTTP client")?;

    // the dataset is streamed twice: first to count elements for optimal filter sizing,
    // then to populate the filter. Counting upfront avoids resizing the memory-mapped file.
    let response = client
        .get(config::DATA_URL)
        .send()
        .with_context(|| format!("failed to fetch data from {}", config::DATA_URL))?
        .error_for_status()
        .context("server returned an error")?;

    let mut reader = BufReader::new(response);

    let mut line = String::new();

    let mut element_count = 0;

    loop {
        line.clear();

        let bytes_read = reader
            .read_line(&mut line)
            .context("failed to read line from data stream during count pass")?;

        if bytes_read == 0 {
            break;
        }

        let trimmed = line.trim();

        if !trimmed.starts_with('#') && !trimmed.is_empty() {
            element_count += 1;
        }
    }

    if element_count == 0 {
        bail!("no valid data found in dataset");
    }

    let bit_size = optimal_bit_size(element_count, config::TARGET_FALSE_POSITIVE_RATE);

    println!("preparing...");

    let byte_size = bit_size.div_ceil(8);

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(config::FILTER_PATH)
        .with_context(|| format!("failed to create {}", config::FILTER_PATH))?;

    // header: [bit_size: 8 bytes][hash_count: 8 bytes] = 16 bytes total
    file.write_all(&(bit_size as u64).to_le_bytes())
        .context("failed to write bit_size to filter header")?;

    file.write_all(&(config::HASH_COUNT as u64).to_le_bytes())
        .context("failed to write hash_count to filter header")?;

    // pre-allocate the full file size before mapping — mmap requires the file to be
    // at least as large as the mapped region.
    file.set_len((16 + byte_size) as u64)
        .context("failed to pre-allocate filter file size")?;

    let mut mmap = unsafe {
        memmap2::MmapMut::map_mut(&file).context("failed to memory-map filter file for writing")?
    };

    let bit_slice = &mut mmap[16..];

    let response = client
        .get(config::DATA_URL)
        .send()
        .with_context(|| {
            format!(
                "failed to fetch data from {} (second pass)",
                config::DATA_URL
            )
        })?
        .error_for_status()
        .context("server returned an error")?;

    let mut reader = BufReader::new(response);

    let mut processed = 0;

    loop {
        line.clear();

        let bytes_read = reader
            .read_line(&mut line)
            .context("failed to read line from data stream during fill pass")?;

        if bytes_read == 0 {
            break;
        }

        let trimmed = line.trim();

        if !trimmed.starts_with('#') && !trimmed.is_empty() {
            let normalized = normalize_url(trimmed);

            for idx in bit_indices(&normalized, config::HASH_COUNT, bit_size) {
                bit_slice[idx / 8] |= 1u8 << (idx % 8);
            }

            processed += 1;

            if processed % 25_000 == 0 || processed == element_count {
                print!("\rpreparing data [{}/{}]", processed, element_count);
                std::io::stdout()
                    .flush()
                    .context("failed to flush stdout")?;
            }
        }
    }

    mmap.flush().context("failed to flush memory map to disk")?;

    println!();

    println!("done");

    Ok(())
}
