mod bloom;

use bloom::BitBloomFilter;

use std::io::{self, Write};

use url::Url;

const URL_DATA: &str = include_str!("../data/urls.txt");

fn main() {
    // n = 78_000 urls, p = 0.01 -> m ≈ 4_320_000 bits, k ≈ 7
    let mut filter = BitBloomFilter::new(4_320_000, 7);

    for line in URL_DATA
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        filter.add(&normalize_url(line.trim()));
    }

    print_welcome_screen();

    run_interactive_loop(&filter);
}

fn print_welcome_screen() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  QUICK URL CHECKER");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Enter a URL to check  ('exit' to quit)\n");
}

fn normalize_url(input: &str) -> String {
    let url_with_scheme = if input.contains("://") {
        input.to_string()
    } else {
        format!("https://{}", input) // http:// and https:// variants are treated as distinct entries
    };

    match Url::parse(&url_with_scheme) {
        Ok(mut u) => {
            u.set_fragment(None);
            u.set_query(None);
            u.to_string().trim_end_matches('/').to_lowercase()
        }
        Err(_) => input.to_lowercase(), // unparsable urls are still checked as-is
    }
}

fn run_interactive_loop(filter: &BitBloomFilter) {
    loop {
        print!("URL: ");

        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();

        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading input");
            continue;
        }

        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            println!("Program terminated by user.");
            break;
        }

        if input.is_empty() {
            continue;
        }

        let normalized = normalize_url(input);

        let flagged = filter.contains(&normalized);

        if flagged {
            println!("This URL might be dangerous!");
        } else {
            println!("This URL is safe!");
        }
        println!("--------------------------------------------------");
    }
}
