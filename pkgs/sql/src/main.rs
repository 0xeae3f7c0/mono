//! Ad-hoc SQL over activity exports (CSV from `events-api` or `gitstat`) with
//! DataFusion.
//!
//!   query --table commits=commits.csv "SELECT author, count(*) FROM commits GROUP BY author"

use clap::Parser;
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::error::Result;
use datafusion::prelude::{CsvReadOptions, SessionContext};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Register a CSV file as a table: NAME=PATH (repeatable)
    #[arg(long = "table", value_parser = parse_table)]
    tables: Vec<(String, String)>,
    /// SQL to run
    sql: String,
}

fn parse_table(s: &str) -> std::result::Result<(String, String), String> {
    s.split_once('=')
        .map(|(n, p)| (n.to_owned(), p.to_owned()))
        .ok_or_else(|| format!("expected NAME=PATH, got `{s}`"))
}

async fn run(tables: &[(String, String)], sql: &str) -> Result<String> {
    let ctx = SessionContext::new();
    for (name, path) in tables {
        ctx.register_csv(name, path, CsvReadOptions::new()).await?;
    }
    let batches = ctx.sql(sql).await?.collect().await?;
    Ok(pretty_format_batches(&batches)?.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    println!("{}", run(&args.tables, &args.sql).await?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn aggregates_csv() {
        let dir = std::env::temp_dir().join(format!("query-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("commits.csv");
        std::fs::write(&file, "author,n\nbob,3\nalice,5\nbob,4\n").unwrap();

        let out = run(
            &[("commits".into(), file.display().to_string())],
            "SELECT author, sum(n) AS total FROM commits GROUP BY author ORDER BY total DESC",
        )
        .await
        .unwrap();
        let rows: Vec<_> = out
            .lines()
            .filter(|l| l.starts_with("| ") && !l.contains("author"))
            .collect();
        assert_eq!(rows, ["| bob    | 7     |", "| alice  | 5     |"]);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn table_flag_requires_equals() {
        assert!(parse_table("commits").is_err());
        assert_eq!(
            parse_table("a=b.csv").unwrap(),
            ("a".into(), "b.csv".into())
        );
    }
}
