#[cfg(test)]
mod tests;

use url::Host;

pub fn hostname_from_url(url: &str) -> Option<String> {
    url::Url::parse(url).ok().and_then(|url| match url.host() {
        Some(Host::Domain(domain)) => {
            if !domain.contains(['{', '}']) {
                Some(domain.to_string())
            } else {
                None
            }
        }
        _ => None,
    })
}
