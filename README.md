# Rust SURT Library

This library provides a Rust implementation for generating a Sort-friendly URI Reordering Transform (SURT) from a given URL. These are predominantly used in the Web Archiving world.

## Usage

```rust
use surt::generate_surt;

let url = "http://example.com/path?query=value#fragment";
let surt = generate_surt(url).unwrap();
println!("{}", surt);  // prints: "com,example)/path?query=value#fragment"
```

## Functions

### `generate_surt(url: &str) -> Result<String, ParseError>`

Generates a SURT from the given URL. Returns a `Result` that contains the SURT as a `String` if the URL is valid, or a `ParseError` if the URL is not valid.

### `normalize_surt(surt: &str) -> String`

Normalizes the given SURT by replacing whitespace with '%20' and removing trailing slashes unless it's the root path.

### `normalize_url(url: &str) -> String`

Normalizes the given URL by removing trailing slashes and the 'www.' subdomain after the scheme.

## License

This project is licensed under the MIT License.