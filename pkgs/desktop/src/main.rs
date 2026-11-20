//! Desktop status app (Tauri). The web UI in `dist/` pastes or polls CI run
//! data from the status worker and calls `ci_summary` to show pass/fail counts
//! and the overall state.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Run {
    conclusion: Option<String>,
}

#[derive(Serialize, Debug, PartialEq)]
struct Summary {
    passed: usize,
    failed: usize,
    running: usize,
    state: &'static str,
}

#[tauri::command]
fn ci_summary(runs_json: &str) -> Result<Summary, String> {
    let runs: Vec<Run> = serde_json::from_str(runs_json).map_err(|e| e.to_string())?;
    let count = |c: Option<&str>| runs.iter().filter(|r| r.conclusion.as_deref() == c).count();
    let (passed, failed, running) = (count(Some("success")), count(Some("failure")), count(None));
    let state = if failed > 0 {
        "failing"
    } else if running > 0 {
        "running"
    } else {
        "passing"
    };
    Ok(Summary {
        passed,
        failed,
        running,
        state,
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ci_summary])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_runs() {
        let json = r#"[{"conclusion":"success"},{"conclusion":"failure"},{"conclusion":null},{"conclusion":"success"}]"#;
        assert_eq!(
            ci_summary(json).unwrap(),
            Summary {
                passed: 2,
                failed: 1,
                running: 1,
                state: "failing"
            }
        );
        assert_eq!(ci_summary("[]").unwrap().state, "passing");
    }

    #[test]
    fn rejects_bad_json() {
        assert!(ci_summary("not json").is_err());
    }
}
