//! Validation report shared by every `commit` command, with text and JSON renderers.

use serde::Serialize;

#[derive(Serialize, Default, Debug)]
pub struct Report {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub workflows: Vec<String>,
    pub changed_files: Vec<String>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("report serializes")
    }

    /// Human-readable rendering: `(stdout, stderr)`.
    pub fn to_text(&self) -> (String, String) {
        let mut out = String::new();
        let mut err = String::new();
        for error in &self.errors {
            err.push_str(&format!("✗ {error}\n"));
        }
        for warning in &self.warnings {
            err.push_str(&format!("! {warning}\n"));
        }
        out.push_str(&format!("Workflows ({}):\n", self.workflows.len()));
        for workflow in &self.workflows {
            out.push_str(&format!("  • {workflow}\n"));
        }
        if !self.changed_files.is_empty() {
            out.push_str(&format!("Changed files ({}):\n", self.changed_files.len()));
            for file in &self.changed_files {
                out.push_str(&format!("  • {file}\n"));
            }
        }
        if self.ok() {
            out.push_str("✓ Repository validation passed.\n");
        }
        (out, err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_report_is_ok_and_says_so() {
        let report = Report::default();
        assert!(report.ok());
        assert!(report.to_text().0.contains("validation passed"));
    }

    #[test]
    fn errors_go_to_stderr_and_fail() {
        let report = Report {
            errors: vec!["boom".into()],
            ..Default::default()
        };
        assert!(!report.ok());
        assert_eq!(report.to_text().1, "✗ boom\n");
    }

    #[test]
    fn text_rendering_snapshot() {
        let report = Report {
            warnings: vec!["careful".into()],
            workflows: vec!["ci.yml".into()],
            changed_files: vec!["src/lib.rs".into()],
            ..Default::default()
        };
        let (out, err) = report.to_text();
        insta::assert_snapshot!(out, @r"
        Workflows (1):
          • ci.yml
        Changed files (1):
          • src/lib.rs
        ✓ Repository validation passed.
        ");
        insta::assert_snapshot!(err, @"! careful");
    }

    #[test]
    fn json_has_all_fields() {
        let json: serde_json::Value = serde_json::from_str(&Report::default().to_json()).unwrap();
        for key in ["errors", "warnings", "workflows", "changed_files"] {
            assert!(json.get(key).is_some(), "missing {key}");
        }
    }
}
