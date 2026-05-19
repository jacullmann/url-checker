# url-checker

A command-line tool that checks URLs against a list of known malicious domains using a Bloom filter.

## Usage

```
cargo build --release
./target/release/url-checker build
./target/release/url-checker check <url>
```

`build` downloads the latest list of malicious URLs from `URLhaus` and saves a prepared bloom filter to disk.  
`check` checks whether the entered url possibly appears in the list

## Tests

```
cargo test
```

## How it works

### Bloom filter

A Bloom filter checks whether a specific element is a member of a large set without storing the elements themselves. It uses a bitfield and multiple hash functions.

**Adding a URL:** the URL is hashed `k` times, producing `k` positions in the bitfield. Each of those bits is set to `1` in the bitfield.

**Checking a URL:** the same `k` positions are checked. If all are `1`, the URL is probably in the list. If any is `0`, it is definitely not.

This means the filter cannot have false negatives – a known malicious URL will never be reported as safe. It can however produce false positives — a safe URL might be flagged because its bits happen to have been set by other URLs.

### Parameters

Filter parameters can be calculated from the actual number of URLs in the dataset at build time, targeting a false-positive rate of `p = 1%`:

```
m = -(n * ln(p)) / ln(2)²   ->  optimal bit count
k = (m / n) * ln(2)         ->  optimal hash count
```

### Hashing

Two independent FNV hash values with different seeds are computed for each URL. All `k` positions are then derived via double hashing:

```
h(i) = h1 + i * h2   for i = 0..k
```

This avoids needing `k` independent hash functions while producing the same false-positive rate.

### URL normalization

Before a URL is added to the filter or checked against it, it is normalized:

- A scheme is added if missing (default: `https://`)
- Fragment and query are stripped (`#anchor`, `?param=value`)
- Trailing slashes are removed
- The result is lowercased

Both the dataset and user input go through the same normalization, so comparisons are consistent. Note that `http://` and `https://` variants are treated as distinct entries.

This does not meet the perfect standards of URL normalization, but for these purposes, it is sufficient and pragmatic.

## Data

The URL dataset is from [URLhaus](https://urlhaus.abuse.ch).

## References

- Bloom, B. H. (1970) — [Space/Time Trade-offs in Hash Coding with Allowable Errors](https://dl.acm.org/doi/10.1145/362686.362692)
- Kirsch, Mitzenmacher (2008) — [Less Hashing, Same Performance](https://www.eecs.harvard.edu/~michaelm/postscripts/rsa2008.pdf)

## License

MIT — see [LICENSE](./LICENSE).
```