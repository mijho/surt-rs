# Rust SURT

A Rust implementation of the Sort-friendly URI Reordering Transform (SURT). SURTs are mainly used in web archiving to give a normalised, sortable key for a URL, for use at replay time.

## Usage

```rust
use surt_rs::generate_surt;

let surt = generate_surt("http://example.com/path?query=value#fragment").unwrap();
assert_eq!(surt, "com,example)/path?query=value#fragment");
```

### Command line

```sh
$ cargo run --bin surt -- "http://www.example.com/a/" "http://EXAMPLE.com/?B=1&a=2"
com,example)/a
com,example)/?a=2&b=1
```

URLs that cannot be converted are echoed unchanged, with a warning on stderr.

## Functions

### `generate_surt(url: &str) -> Result<String, ParseError>`

Generates a SURT from an `http` or `https` URL. Returns a `ParseError` if the URL cannot be parsed, has a different scheme, or has no host.

Normalisation applied:

- The host is lowercased, its labels are reversed and comma-separated, and a trailing dot is removed.
- A leading `www` or `wwwN` label is removed when a domain remains after it (`www2.example.com` becomes `example.com`, but `wwwtest.example.com` is kept).
- Query parameters are lowercased, sorted, and stripped of known session IDs (`phpsessid`, `jsessionid`, `sid`, and similar) and ColdFusion `cfid`/`cftoken` pairs.
- Trailing slashes are removed, except for the root path.
- Percent-escapes are decoded, except for those that would make the SURT ambiguous (`%3F`, `%23`, `%26`, `%29`, `%25`).

## Changes in 0.2.0

- Query parameters are now sorted after lowercasing, so `?B=1&a=2` gives `?a=2&b=1`.
- Session IDs are removed without leaving a trailing `&`.
- `wwwtest.example.com` is no longer treated as `www.`-prefixed.
- Encoded delimiters (`%3F`, `%26`, and others) stay encoded, so they cannot be confused with SURT structure.
- A trailing dot on a host is removed.
- The `surt` binary no longer panics without an argument and accepts several URLs.
- Dependencies are upgraded, `lazy_static` and `url-escape` are removed, and the minimum supported Rust version is 1.80.

## License

This project is licensed under the MIT License.
