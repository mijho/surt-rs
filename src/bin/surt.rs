use std::env;
use std::process::ExitCode;
use surt_rs::generate_surt;

/// Print the SURT for each URL given on the command line, one per line.
/// URLs that cannot be converted are echoed unchanged, with a warning on stderr.
fn main() -> ExitCode {
    let urls: Vec<String> = env::args().skip(1).collect();
    if urls.is_empty() {
        eprintln!("usage: surt <url> [url...]");
        return ExitCode::from(2);
    }

    for url in &urls {
        match generate_surt(url) {
            Ok(surt) => println!("{surt}"),
            Err(err) => {
                eprintln!("surt: could not convert {url:?}: {err}");
                println!("{url}");
            }
        }
    }
    ExitCode::SUCCESS
}
