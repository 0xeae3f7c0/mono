// badges: README badges (SVG) (commit platform, pkgs/badges)
//! Badge service: renders shields-style SVG badges for READMEs, e.g.
//! `GET /badge/commits/1.2k.svg` or `GET /badge/ci/passing.svg`.
//! Colour comes from the value (passing/failing) or `?color=`.

use actix_web::{App, HttpResponse, HttpServer, get, web};
use serde::Deserialize;

#[derive(Deserialize)]
struct Options {
    color: Option<String>,
}

/// Approximate text width in pixels for Verdana 11px (what shields.io uses).
fn text_width(text: &str) -> usize {
    text.chars()
        .map(|c| {
            if c.is_ascii_uppercase() || c == 'm' || c == 'w' {
                9
            } else {
                7
            }
        })
        .sum::<usize>()
        + 10
}

fn colour(value: &str, explicit: Option<&str>) -> String {
    if let Some(c) = explicit.filter(|c| c.len() == 6 && c.chars().all(|ch| ch.is_ascii_hexdigit()))
    {
        return format!("#{c}");
    }
    match value {
        "passing" | "success" | "ok" => "#4c1",
        "failing" | "failure" | "error" => "#e05d44",
        "pending" | "running" => "#dfb317",
        _ => "#007ec6",
    }
    .into()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn badge(label: &str, value: &str, colour: &str) -> String {
    let (lw, vw) = (text_width(label), text_width(value));
    let (label, value) = (escape(label), escape(value));
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="20" role="img" aria-label="{label}: {value}"><rect width="{lw}" height="20" fill="#555"/><rect x="{lw}" width="{vw}" height="20" fill="{colour}"/><g fill="#fff" font-family="Verdana,sans-serif" font-size="11" text-anchor="middle"><text x="{lx}" y="14">{label}</text><text x="{vx}" y="14">{value}</text></g></svg>"##,
        w = lw + vw,
        lx = lw / 2,
        vx = lw + vw / 2,
    )
}

#[get("/badge/{label}/{value}.svg")]
async fn render(path: web::Path<(String, String)>, query: web::Query<Options>) -> HttpResponse {
    let (label, value) = path.into_inner();
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .insert_header(("cache-control", "max-age=300"))
        .body(badge(
            &label,
            &value,
            &colour(&value, query.color.as_deref()),
        ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:8081".into());
    println!("listening on {addr}");
    HttpServer::new(|| App::new().service(render))
        .bind(addr)?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test as atest;

    #[test]
    fn colours_follow_status() {
        assert_eq!(colour("passing", None), "#4c1");
        assert_eq!(colour("failing", None), "#e05d44");
        assert_eq!(colour("1.2k", Some("ff00aa")), "#ff00aa");
        assert_eq!(colour("1.2k", Some("not-hex")), "#007ec6");
    }

    #[test]
    fn values_are_escaped() {
        assert!(badge("a<b", "\"x\"", "#000").contains("a&lt;b: &quot;x&quot;"));
    }

    #[actix_web::test]
    async fn serves_svg() {
        let app = atest::init_service(App::new().service(render)).await;
        let res = atest::call_service(
            &app,
            atest::TestRequest::get()
                .uri("/badge/ci/passing.svg")
                .to_request(),
        )
        .await;
        assert_eq!(res.headers().get("content-type").unwrap(), "image/svg+xml");
        let body = String::from_utf8(atest::read_body(res).await.to_vec()).unwrap();
        assert!(body.starts_with("<svg") && body.contains(">passing<") && body.contains("#4c1"));
    }
}
