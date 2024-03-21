use url::{Url, ParseError};

fn main() -> Result<(), ParseError> {
    let url = "https://github.com:8080/rust-lang/rust/issues?labels=E-easy&state=open#issuecomment-396527541";
    let mut parsed = Url::parse(url)?;
    println!("Parsed: {:#?}", parsed);

    let mut scheme = parsed.scheme();
    if parsed.cannot_be_a_base() {
        scheme = "https";
        parsed = Url::parse(&format!("{}://{}", scheme, url))?;
    }
    match scheme == "https" || scheme == "http" {
        true => scheme,
        _ => return Err(ParseError::RelativeUrlWithoutBase)
    };
    println!("Scheme: {}", scheme);

    if parsed.path() != "" {
        let path = parsed.path().to_lowercase();
        println!("Path: {}", path);
    }

    if parsed.query().is_some() {
        let query = parsed.query().unwrap().to_lowercase();
        println!("Query: {}", query);
    }

    if parsed.fragment().is_some() {
        let fragment = parsed.fragment().unwrap().to_lowercase();
        println!("Fragment: {}", fragment);
    }

    if parsed.host_str().is_none() {
        return Err(ParseError::RelativeUrlWithoutBase);
    }
    let host_str = parsed.host_str().unwrap().to_lowercase();
    println!("Host: {}", host_str);


    if parsed.port().is_some() {
        let port = parsed.port().unwrap();
        println!("Port: {}", port);
    }

    Ok(())
}
