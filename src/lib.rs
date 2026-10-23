//! Generate SURTs (Sort-friendly URI Reordering Transform) from URLs.
//!
//! A SURT reverses the host labels, so that URLs from the same site sort
//! together, and normalises the rest of the URL so that equivalent URLs
//! produce the same key.

use regex::Regex;
use std::sync::LazyLock;
use url::{ParseError, Url};

/// Percent-escapes that must stay encoded in the SURT head (`host)/path`).
/// `?` and `#` would start a new section, `)` ends the host, `%` keeps
/// decoding unambiguous.
const HEAD_KEEP_ENCODED: &[u8] = b"?#%)";

/// Percent-escapes that must stay encoded in the SURT query/fragment.
/// `&` would split parameters and `#` would start a fragment.
const TAIL_KEEP_ENCODED: &[u8] = b"&#%";

/// Session-identifier query parameters that are dropped during normalisation.
static SESSION_PARAM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?:jsessionid=[0-9a-z$]{10,}|sessionid=[0-9a-z]{16,}|phpsessid=[0-9a-z]{16,}|sid=[0-9a-z]{16,}|aspsessionid[a-z]{8}=[0-9a-z]{16,})$",
    )
    .expect("valid session regex")
});

/// ColdFusion `cfid`/`cftoken` pairs are only dropped when they appear together.
static CFID_PARAM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^cfid=[0-9]+$").expect("valid cfid regex"));
static CFTOKEN_PARAM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^cftoken=[0-9a-z-]+$").expect("valid cftoken regex"));

/// Decode percent-escapes in a SURT, except those listed in `keep`, which are
/// left encoded so the SURT's structure stays unambiguous.
fn percent_decode_keep(s: &str, keep: &[u8]) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                let byte = (hi << 4) | lo;
                if keep.contains(&byte) {
                    out.extend_from_slice(&bytes[i..i + 3]);
                } else {
                    out.push(byte);
                }
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn normalize_surt(surt: &str) -> String {
    // The first literal '?' is the SURT query delimiter; any '?' produced by
    // decoding is kept encoded in the head and so cannot be mistaken for it.
    let decoded = match surt.split_once('?') {
        Some((head, tail)) => format!(
            "{}?{}",
            percent_decode_keep(head, HEAD_KEEP_ENCODED),
            percent_decode_keep(tail, TAIL_KEEP_ENCODED)
        ),
        None => percent_decode_keep(surt, HEAD_KEEP_ENCODED),
    };
    let decoded = decoded.replace(' ', "%20");

    let (head, rest) = match decoded.find('?') {
        Some(i) => decoded.split_at(i),
        None => (decoded.as_str(), ""),
    };

    // Drop a trailing slash from the path, but keep the root path `)/`.
    let head = match head.strip_suffix('/') {
        Some(stripped) if !head.ends_with(")/") => stripped,
        _ => head,
    };

    format!("{head}{rest}")
}

/// Remove a `www`/`wwwN` label, but only when a real domain remains after it.
/// `wwwtest.example.com` and `www2foo.com` are left alone.
fn strip_www(host: &str) -> &str {
    if let Some(rest) = host.strip_prefix("www") {
        let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
        if let Some(domain) = rest.strip_prefix('.') {
            if domain.contains('.') {
                return domain;
            }
        }
    }
    host
}

/// Lowercase, drop session parameters, and sort the rest.
/// Empty parameters are kept, matching the reference SURT fixtures.
fn normalize_query(query: &str) -> String {
    let mut params: Vec<&str> = query.split('&').collect();
    params.sort_unstable();

    let mut kept: Vec<&str> = Vec::with_capacity(params.len());
    let mut i = 0;
    while i < params.len() {
        let is_cf_pair = i + 1 < params.len()
            && CFID_PARAM.is_match(params[i])
            && CFTOKEN_PARAM.is_match(params[i + 1]);
        if is_cf_pair {
            i += 2;
            continue;
        }
        if !SESSION_PARAM.is_match(params[i]) {
            kept.push(params[i]);
        }
        i += 1;
    }
    kept.join("&")
}

fn normalize_url(mut parsed: Url) -> String {
    if let Some(query) = parsed.query() {
        let normalized = normalize_query(&query.to_lowercase());
        let new_query = (!normalized.is_empty()).then_some(normalized);
        parsed.set_query(new_query.as_deref());
    }

    if let Some(host) = parsed.host_str().map(str::to_owned) {
        let trimmed = strip_www(host.trim_end_matches('.'));
        if trimmed != host {
            // A suffix of a valid host is itself a valid host, so this only
            // fails for input that was already rejected by the URL parser.
            let _ = parsed.set_host(Some(trimmed));
        }
    }

    let mut url = parsed.to_string();

    // Remove a trailing slash unless it is the root path.
    if url.ends_with('/') && parsed.path() != "/" {
        url.pop();
    }

    url
}

/// Generate the SURT for an `http` or `https` URL.
///
/// Returns an error if the URL cannot be parsed, is not `http`/`https`, or
/// has no host.
pub fn generate_surt(url: &str) -> Result<String, ParseError> {
    let parsed = Url::parse(url)?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ParseError::RelativeUrlWithoutBase);
    }

    let normalized = Url::parse(&normalize_url(parsed))?;
    let host = normalized
        .host_str()
        .ok_or(ParseError::RelativeUrlWithoutBase)?;

    let mut surt = host.rsplit('.').collect::<Vec<&str>>().join(",");

    if let Some(port) = normalized.port() {
        surt += &format!(":{port}");
    }

    surt.push(')');
    surt.push_str(&normalized.path().to_lowercase());

    if let Some(query) = normalized.query() {
        surt.push('?');
        surt.push_str(&query.to_lowercase());
    }

    if let Some(fragment) = normalized.fragment() {
        surt.push('#');
        surt.push_str(&fragment.to_lowercase());
    }

    Ok(normalize_surt(&surt))
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
            for (input, expected) in examples {
                let surt = generate_surt(&input).unwrap();
                assert_eq!(surt, expected, "input: {input}");
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
            for (input, expected) in examples {
                let url = normalize_url(Url::parse(&input).unwrap());
                assert_eq!(url, expected, "input: {input}");
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
        assert!(generate_surt("example.com").is_err());
    }

    #[test]
    fn test_generate_surt_with_relative_url() {
        assert!(generate_surt("/path").is_err());
    }

    #[test]
    fn test_generate_surt_with_url_without_host() {
        assert!(generate_surt("http://").is_err());
    }

    #[test]
    fn test_generate_surt_with_magic_ampersand() {
        // A trailing session parameter must not leave a dangling `&`.
        let url = "http://www.example.com/khxc/index.php?app=ccp0&ns=catshow&ref=kawasaki&prodsort=PRICEUP&sid=ze2509b1v54ldiu17199jlcf8n88cquz";
        let expected =
            "com,example)/khxc/index.php?app=ccp0&ns=catshow&prodsort=priceup&ref=kawasaki";
        assert_eq!(generate_surt(url).unwrap(), expected);
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
    fn test_generate_surt_keeps_non_www_subdomains_starting_with_www() {
        assert_eq!(
            generate_surt("http://wwwtest.example.com/x").unwrap(),
            "com,example,wwwtest)/x"
        );
    }

    #[test]
    fn test_generate_surt_with_ftp_url() {
        assert!(generate_surt("ftp://www.example.com").is_err());
    }

    #[test]
    fn test_generate_surt_sorts_query_case_insensitively() {
        let url = "http://example.com/?B=1&a=2";
        assert_eq!(generate_surt(url).unwrap(), "com,example)/?a=2&b=1");
    }

    #[test]
    fn test_generate_surt_drops_session_params_without_trailing_ampersand() {
        let expected = "com,example)/?a=1";
        for url in [
            "http://example.com/?a=1&phpsessid=0123456789abcdef",
            "http://example.com/?phpsessid=0123456789abcdef&a=1",
        ] {
            assert_eq!(generate_surt(url).unwrap(), expected, "input: {url}");
        }
        assert_eq!(
            generate_surt("http://example.com/?phpsessid=0123456789abcdef").unwrap(),
            "com,example)/"
        );
    }

    #[test]
    fn test_generate_surt_drops_cfid_token_pairs() {
        let url = "http://example.com/?cfid=12&cftoken=abc-def&a=1";
        assert_eq!(generate_surt(url).unwrap(), "com,example)/?a=1");
    }

    #[test]
    fn test_generate_surt_ignores_trailing_dot_on_host() {
        assert_eq!(
            generate_surt("http://example.com./x").unwrap(),
            "com,example)/x"
        );
    }

    #[test]
    fn test_generate_surt_keeps_encoded_delimiters_encoded() {
        assert_eq!(
            generate_surt("http://example.com/a%3Fb").unwrap(),
            "com,example)/a%3fb"
        );
        assert_eq!(
            generate_surt("http://example.com/?q=a%26b=c").unwrap(),
            "com,example)/?q=a%26b=c"
        );
        assert_eq!(
            generate_surt("http://example.com/a%29b").unwrap(),
            "com,example)/a%29b"
        );
    }

    #[test]
    fn test_normalize_url_with_www_subdomain_and_https() {
        let url = Url::parse("https://www.example.com").unwrap();
        assert_eq!(normalize_url(url), "https://example.com/");
    }

    #[test]
    fn test_normalize_surt_root_with_trailing_slash() {
        assert_eq!(normalize_surt("com,example)/"), "com,example)/");
    }

    #[test]
    fn test_normalize_surt_with_trailing_slash() {
        assert_eq!(
            normalize_surt("com,example)/foo/bar/"),
            "com,example)/foo/bar"
        );
    }

    #[test]
    fn test_normalize_surt_decodes_unambiguous_escapes() {
        assert_eq!(
            normalize_surt("com,example)/a%20b%3A%2F"),
            "com,example)/a%20b:"
        );
    }
}
