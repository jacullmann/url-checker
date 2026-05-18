use url::Url;

pub fn normalize_url(input: &str) -> String {
    let url_with_scheme = if input.contains("://") {
        input.to_string()
    } else {
        format!("https://{}", input)
    };

    match Url::parse(&url_with_scheme) {
        Ok(mut u) => {
            u.set_fragment(None);
            u.set_query(None);
            u.to_string().trim_end_matches('/').to_lowercase()
        }
        Err(_) => input.to_lowercase(),
    }
}
