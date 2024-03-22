use url::{Url, ParseError};


fn normalize_surt(surt: &str) -> String {
    let mut surt = surt.to_string();

    // replace whitespace with %20
    surt = surt.replace(" ", "%20");

    let query_index = surt.find('?').unwrap_or_else(|| 0);
    println!("{:?}", query_index);

    // remove trailing slashes unless it's the root path
    if query_index == 0 {
        if surt.ends_with("/") && !surt.ends_with(")/") {
            surt.pop();
        }
    }

    // remove trailing slash for SURTs with query parameters
    // unless it's the root path
    let start = &mut surt[..query_index].to_string();
    if start.ends_with("/") && !start.ends_with(")/") {
        start.pop();
    }
    surt = format!("{}{}", start, &surt[query_index..]);

    surt
}

fn normalize_url(url: &str) -> String {
    let mut url = url.to_string();

    // replace trailing slash
    if url.ends_with("/") {
        url.pop();
    }

    // remove www subdomain after scheme
    // TODO: make this less clunky
    if url.starts_with("http://www.") {
        url = url.replacen("http://www.", "http://", 1);
    } else if url.starts_with("https://www.") {
        url = url.replacen("https://www.", "https://", 1);
    }

    url
}

pub fn generate_surt(url: &str) -> Result<String, ParseError> {
    let parsed = Url::parse(&normalize_url(url))?;

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

    surt = normalize_surt(&surt);

    Ok(surt)
}

#[cfg(test)]
mod tests {
  use super::*;

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
}