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
  fn test_generate_surt_with_url_with_fragment() {
    let url = "http://example.com#fragment";
    let expected = "com,example)/#fragment";
    assert_eq!(generate_surt(url).unwrap(), expected);
  }
}