use url::{Url, ParseError};
use std::env;
use surt_rs;


// This will generate the SURT for the given URL
fn main() -> Result<(), ParseError> {
    let args: Vec<String> = env::args().collect();
    let url = &args[1];

    let mut parsed = Url::parse(url)?;

    let mut scheme = parsed.scheme();
    if parsed.cannot_be_a_base() {
        scheme = "https";
        parsed = Url::parse(&format!("{}://{}", scheme, url))?;
    }   

    let surt = surt_rs::generate_surt(url).unwrap();
    println!("{}", surt);
    Ok(())
}
