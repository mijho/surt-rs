use url::{Url, ParseError};

pub fn generate_surt(url: &str) -> Result<String, ParseError> {
    let parsed = Url::parse(url)?;

    let scheme = parsed.scheme();
    match scheme == "https" || scheme == "http" {
        true => scheme,
        _ => return Err(ParseError::RelativeUrlWithoutBase)
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
        let mut query = parsed.query().unwrap().split('&').collect::<Vec<&str>>();
        query.sort();
        surt += &format!("?{}", query.join("&").to_lowercase());
    }

    if parsed.fragment().is_some() {
        let fragment = parsed.fragment().unwrap().to_lowercase();
        surt += &format!("#{}", fragment);
    }

    Ok(surt)
}
