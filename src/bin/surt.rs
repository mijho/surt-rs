use url::{ParseError};
use std::env;
use surt_rs::{generate_surt};


// This will generate the SURT for the given URL
fn main() -> Result<(), ParseError> {
    let args: Vec<String> = env::args().collect();
    let url = &args[1]; 
    let surt = generate_surt(url).unwrap_or_else(|_| url.to_string());

    println!("{}", surt);
    Ok(())
}
