// crawl: Release-note/changelog links for dependency PRs (commit platform, pkgs/crawl)
//! Collects release-note and changelog links from a dependency's web page, so
//! dependency PRs can link to what changed. Lists links as absolute URLs.
//!
//!   crawl https://example.com

use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;

/// Absolute, de-duplicated http(s) links found in `html`, in document order.
fn links(base: &Url, html: &str) -> Vec<Url> {
    let selector = Selector::parse("a[href]").expect("valid selector");
    let mut seen = Vec::new();
    for href in Html::parse_document(html)
        .select(&selector)
        .filter_map(|a| a.value().attr("href"))
    {
        if let Ok(url) = base.join(href)
            && matches!(url.scheme(), "http" | "https")
            && !seen.contains(&url)
        {
            seen.push(url);
        }
    }
    seen
}

async fn crawl(client: &Client, url: &Url) -> reqwest::Result<Vec<Url>> {
    let body = client
        .get(url.clone())
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(links(url, &body))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url: Url = std::env::args()
        .nth(1)
        .ok_or("usage: crawl <url>")?
        .parse()?;
    for link in crawl(&Client::new(), &url).await? {
        println!("{link}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::path;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn resolves_and_dedupes_links() {
        let base = Url::parse("https://example.com/docs/").unwrap();
        let html = r#"<a href="a">A</a><a href="/b">B</a><a href="a">again</a><a href="mailto:x@y">mail</a>"#;
        let got: Vec<_> = links(&base, html).into_iter().map(String::from).collect();
        assert_eq!(got, ["https://example.com/docs/a", "https://example.com/b"]);
    }

    #[tokio::test]
    async fn crawls_a_served_page() {
        let server = MockServer::start().await;
        Mock::given(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"<a href="/next">next</a>"#))
            .mount(&server)
            .await;
        let url = Url::parse(&server.uri()).unwrap();
        let got = crawl(&Client::new(), &url).await.unwrap();
        assert_eq!(got, [url.join("/next").unwrap()]);
    }
}
