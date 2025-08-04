use std::sync::LazyLock;

use regex::Regex;
use url::{ParseError, Url};

fn normalize_surt(surt: &str) -> String {
    let mut surt = surt.to_string();

    // decode surt
    surt = url_escape::decode(&surt).to_string();

    // replace whitespace with %20
    surt = surt.replace(' ', "%20");

    let query_index = surt.find('?').unwrap_or(0);

    // remove trailing slashes unless it's the root path
    if query_index == 0 && surt.ends_with('/') && !surt.ends_with(")/") {
        surt.pop();
    }

    // remove trailing slash for SURTs with query parameters
    // unless it's the root path
    let start = &mut surt[..query_index].to_string();
    if start.ends_with('/') && !start.ends_with(")/") {
        start.pop();
    }
    surt = format!("{}{}", start, &surt[query_index..]);

    surt
}

static SESSION_REGEXP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(&|^)(?:jsessionid=[0-9a-z$]{10,}|sessionid=[0-9a-z]{16,}|phpsessid=[0-9a-z]{16,}|sid=[0-9a-z]{16,}|aspsessionid[a-z]{8}=[0-9a-z]{16,}|cfid=[0-9]+&cftoken=[0-9a-z-]+)(&|$)").unwrap()
});
static WWW_REGEXP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^www(\w?)+\.(.*\.+)").unwrap());

fn normalize_url(mut parsed: Url) -> String {
    // lowercase and sort query parameters
    if parsed.query().is_some() {
        let mut query = parsed.query().unwrap().split('&').collect::<Vec<&str>>();
        query.sort();
        let mut query = query.join("&").to_lowercase();
        query = SESSION_REGEXP.replace_all(&query, "$1$3").to_string();
        parsed.set_query(Some(&query));
    }

    if parsed.host_str().is_some() {
        // remove www(ish) subdomain
        let host_str = parsed.host_str().unwrap();
        let host_str = WWW_REGEXP.replace(host_str, "${2}").to_string();

        // lowercase host
        let host_str = host_str.to_lowercase();

        parsed.set_host(Some(&host_str)).unwrap();
    }

    let mut url = parsed.to_string();

    // replace trailing slash unless it's the root path
    if url.ends_with('/') && parsed.path() != "/" {
        url.pop();
    }

    // replace trailing ?
    if url.ends_with('?') {
        url.pop();
    }

    url
}

pub fn generate_surt(url: &str) -> Result<String, ParseError> {
    let mut parsed = Url::parse(url)?;
    parsed = Url::parse(&normalize_url(parsed))?;

    let scheme = parsed.scheme();
    match scheme == "https" || scheme == "http" {
        true => scheme,
        _ => return Err(ParseError::RelativeUrlWithoutBase),
    };

    if parsed.host_str().is_none() {
        return Err(ParseError::RelativeUrlWithoutBase);
    }
    let host_str = parsed.host_str().unwrap().to_lowercase();
    let mut host_split = host_str.split('.').collect::<Vec<&str>>();
    host_split.reverse();
    let mut surt = host_split.join(",");

    if parsed.port().is_some() {
        let port = parsed.port().unwrap();
        surt += &format!(":{}", port);
    }

    if parsed.path() != "" {
        let path = parsed.path().to_lowercase();
        surt += &format!("){}", path);
    }

    if parsed.query().is_some() {
        let query = parsed.query().unwrap().to_lowercase();
        surt += &format!("?{}", query);
    }

    if parsed.fragment().is_some() {
        let fragment = parsed.fragment().unwrap().to_lowercase();
        surt += &format!("#{}", fragment);
    }

    surt = normalize_surt(&surt);

    Ok(surt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::BufReader;

    fn load_test_data() -> HashMap<String, HashMap<String, String>> {
        let file = File::open("./test_data/surt.json").unwrap();
        let reader = BufReader::new(file);
        let v: Value = serde_json::from_reader(reader).unwrap();
        v.as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| {
                let inner_map = v
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
                    .collect();
                (k.clone(), inner_map)
            })
            .collect()
    }

    #[test]
    fn test_surt() {
        let test_data = load_test_data();

        for (section, examples) in test_data {
            // if section does not include surt case insensitive skip
            if !section.to_lowercase().contains("surt") {
                continue;
            }
            println!("Testing section: {}", section);

            for (input, expected) in examples {
                println!("Testing example: {}", input);
                let surt = generate_surt(&input).unwrap();
                assert_eq!(surt, expected);
            }
        }
    }

    #[test]
    fn test_url_normalization() {
        let test_data = load_test_data();

        for (section, examples) in test_data {
            // if section does not include url_normalization case insensitive skip
            if !section.to_lowercase().contains("url_normalization") {
                continue;
            }
            println!("Testing section: {}", section);

            for (input, expected) in examples {
                println!("Testing example: {}", input);
                let parsed = Url::parse(&input);
                println!("parsed: {:?}", parsed);
                let url = normalize_url(Url::parse(&input).unwrap());
                assert_eq!(url, expected);
            }
        }
    }

    #[test]
    fn test_generate_surt_with_valid_url() {
        let url = "http://example.com/path?query=value#fragment";
        let expected = "com,example)/path?query=value#fragment";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_url_without_scheme() {
        let url = "example.com";
        assert!(generate_surt(url).is_err());
    }

    #[test]
    fn test_generate_surt_with_relative_url() {
        let url = "/path";
        assert!(generate_surt(url).is_err());
    }

    #[test]
    fn test_generate_surt_with_url_without_host() {
        let url = "http://";
        assert!(generate_surt(url).is_err());
    }

    #[test]
    fn test_generate_surt_with_url_with_port() {
        let url = "http://example.com:8080";
        let expected = "com,example:8080)/";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_url_with_query() {
        let url = "http://example.com?query=value";
        let expected = "com,example)/?query=value";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_url_with_query_and_trailing_slash_after_path() {
        let url = "http://example.com/foo/bar/?query=value";
        let expected = "com,example)/foo/bar?query=value";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_url_with_fragment() {
        let url = "http://example.com#fragment";
        let expected = "com,example)/#fragment";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_url_with_uppercase() {
        let url = "http://EXAMPLE.COM/PATH?QUERY=VALUE#FRAGMENT";
        let expected = "com,example)/path?query=value#fragment";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_url_with_space() {
        let url = "http://example.com/path with space";
        let expected = "com,example)/path%20with%20space";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_url_with_trailing_slash() {
        let url = "http://example.com/";
        let expected = "com,example)/";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_url_with_trailing_slash_after_path() {
        let url = "http://example.com/foo/bar/";
        let expected = "com,example)/foo/bar";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_url_with_www_subdomain() {
        let url = "http://www.example.com";
        let expected = "com,example)/";
        assert_eq!(generate_surt(url).unwrap(), expected);
    }

    #[test]
    fn test_generate_surt_with_ftp_url() {
        let url = "ftp://www.example.com";
        assert!(generate_surt(url).is_err());
    }

    #[test]
    fn test_normalize_url_with_www_subdomain_and_https() {
        let url = Url::parse("https://www.example.com").unwrap();
        let expected = "https://example.com/";
        assert_eq!(normalize_url(url), expected);
    }

    #[test]
    fn test_normalize_surt_root_with_trailing_slash() {
        let url = "com,example)/";
        let expected = "com,example)/";
        assert_eq!(normalize_surt(url), expected);
    }

    #[test]
    fn test_normalize_surt_with_trailing_slash() {
        let url = "com,example)/foo/bar/";
        let expected = "com,example)/foo/bar";
        assert_eq!(normalize_surt(url), expected);
    }
}
