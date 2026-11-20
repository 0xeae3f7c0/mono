// weekly-page: Server-rendered "this week" page (commit platform, pkgs/weekly-page)
//! Server-rendered "this week" page (Leptos SSR on axum): what shipped and
//! what is still open, linked from the weekly report e-mail.

use axum::Router;
use axum::response::Html;
use axum::routing::get;
use leptos::prelude::*;

#[component]
fn TaskList(tasks: Vec<(String, bool)>) -> impl IntoView {
    let done = tasks.iter().filter(|(_, d)| *d).count();
    view! {
        <h1>"This week"</h1>
        <p class="summary">{format!("{done} of {} done", tasks.len())}</p>
        <ul>
            {tasks
                .into_iter()
                .map(|(title, done)| view! { <li class:done=done>{title}</li> })
                .collect_view()}
        </ul>
    }
}

fn render(tasks: Vec<(String, bool)>) -> String {
    let owner = Owner::new();
    owner.with(|| view! { <TaskList tasks=tasks /> }.to_html())
}

fn sample() -> Vec<(String, bool)> {
    vec![
        ("Release v0.4.0".into(), true),
        ("Upgrade runners".into(), false),
    ]
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app = Router::new().route("/", get(|| async { Html(render(sample())) }));
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:3001".into());
    println!("listening on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_summary_and_items() {
        let html = render(sample());
        assert!(html.contains("1 of 2 done"), "{html}");
        assert!(
            html.contains("Release v0.4.0") && html.contains("Upgrade runners"),
            "{html}"
        );
        assert!(html.contains(r#"class="done""#), "{html}");
    }
}
