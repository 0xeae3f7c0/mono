// shortlinks: Short links for shared reports (commit platform, pkgs/shortlinks)
//! Short links for shared reports and dashboards (e.g. links posted in chat
//! by `pipelines`). Stores links in memory; Rocket web service.
//!
//!   POST /links {"url": "..."} -> {"code": "1"}
//!   GET  /<code>               -> 303 redirect

use std::sync::Mutex;

use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::Deserialize;
use rocket::serde::json::{Json, Value, json};
use rocket::{State, get, launch, post, routes};

#[derive(Default)]
struct Links(Mutex<Vec<String>>);

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct NewLink {
    url: String,
}

#[post("/links", data = "<link>")]
fn create(links: &State<Links>, link: Json<NewLink>) -> Result<Json<Value>, Status> {
    if !(link.url.starts_with("http://") || link.url.starts_with("https://")) {
        return Err(Status::UnprocessableEntity);
    }
    let mut all = links.0.lock().unwrap();
    all.push(link.into_inner().url);
    Ok(Json(json!({ "code": all.len().to_string() })))
}

#[get("/<code>")]
fn follow(links: &State<Links>, code: usize) -> Option<Redirect> {
    let all = links.0.lock().unwrap();
    code.checked_sub(1)
        .and_then(|i| all.get(i))
        .map(|url| Redirect::to(url.clone()))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Links::default())
        .mount("/", routes![create, follow])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::ContentType;
    use rocket::local::blocking::Client;

    #[test]
    fn shortens_and_redirects() {
        let client = Client::tracked(rocket()).unwrap();
        let res = client
            .post("/links")
            .header(ContentType::JSON)
            .body(r#"{"url":"https://example.com"}"#)
            .dispatch();
        assert_eq!(res.into_json::<Value>().unwrap(), json!({ "code": "1" }));

        let res = client.get("/1").dispatch();
        assert_eq!(res.status(), Status::SeeOther);
        assert_eq!(
            res.headers().get_one("Location"),
            Some("https://example.com")
        );
        assert_eq!(client.get("/9").dispatch().status(), Status::NotFound);
    }

    #[test]
    fn rejects_non_http_urls() {
        let client = Client::tracked(rocket()).unwrap();
        let res = client
            .post("/links")
            .header(ContentType::JSON)
            .body(r#"{"url":"ftp://x"}"#)
            .dispatch();
        assert_eq!(res.status(), Status::UnprocessableEntity);
    }
}
