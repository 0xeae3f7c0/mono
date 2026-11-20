//! Alert rules editor (Dioxus web app): configure when the platform should
//! alert on CI failures and quiet repositories, with a live preview of the
//! message that would be sent. Run in a browser with `dx serve`.

use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/")]
    Failures {},
    #[route("/quiet")]
    Quiet {},
}

/// Alert text for a repository whose last `window` CI runs had `failed` failures.
fn failure_alert(repo: &str, failed: u32, window: u32, threshold: u32) -> Option<String> {
    (failed >= threshold).then(|| format!("{repo}: {failed} of the last {window} CI runs failed"))
}

/// Alert text for a repository with no commits for `idle_days` (threshold in days).
fn quiet_alert(repo: &str, idle_days: u32, threshold: u32) -> Option<String> {
    (idle_days >= threshold).then(|| format!("{repo}: no commits for {idle_days} days"))
}

#[component]
fn Threshold(label: &'static str, value: Signal<u32>) -> Element {
    let mut value = value;
    rsx! {
        label { "{label} " }
        input {
            r#type: "number",
            min: "1",
            value: "{value}",
            oninput: move |e| value.set(e.value().parse().unwrap_or(1)),
        }
    }
}

#[component]
fn Failures() -> Element {
    let threshold = use_signal(|| 3_u32);
    let preview =
        failure_alert("owner/repo", 4, 10, threshold()).unwrap_or_else(|| "no alert".into());
    rsx! {
        h2 { "CI failures" }
        Threshold { label: "Alert after failures in last 10 runs:", value: threshold }
        p { "Preview (4 of 10 failed): {preview}" }
    }
}

#[component]
fn Quiet() -> Element {
    let threshold = use_signal(|| 14_u32);
    let preview = quiet_alert("owner/repo", 21, threshold()).unwrap_or_else(|| "no alert".into());
    rsx! {
        h2 { "Quiet repositories" }
        Threshold { label: "Alert after idle days:", value: threshold }
        p { "Preview (idle 21 days): {preview}" }
    }
}

#[component]
fn App() -> Element {
    rsx! {
        nav {
            Link { to: Route::Failures {}, "CI failures" }
            " | "
            Link { to: Route::Quiet {}, "Quiet repositories" }
        }
        Router::<Route> {}
    }
}

fn main() {
    dioxus::launch(App);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_alert_respects_threshold() {
        assert_eq!(failure_alert("o/r", 2, 10, 3), None);
        assert_eq!(
            failure_alert("o/r", 3, 10, 3).as_deref(),
            Some("o/r: 3 of the last 10 CI runs failed")
        );
    }

    #[test]
    fn quiet_alert_respects_threshold() {
        assert_eq!(quiet_alert("o/r", 13, 14), None);
        assert_eq!(
            quiet_alert("o/r", 14, 14).as_deref(),
            Some("o/r: no commits for 14 days")
        );
    }

    #[test]
    fn routes_round_trip() {
        assert_eq!(Route::Quiet {}.to_string(), "/quiet");
        assert!(matches!("/".parse::<Route>(), Ok(Route::Failures {})));
    }
}
