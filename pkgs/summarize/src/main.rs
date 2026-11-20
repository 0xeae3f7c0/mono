// summarize: Group-by summaries of exports (commit platform, pkgs/summarize)
//! Summarizes an activity export (CSV) grouped by one column using polars,
//! e.g. commits per author or CI minutes per repository.
//!
//!   dataframe data.csv --by author --value commits

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use polars::prelude::*;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// CSV file with a header row
    file: PathBuf,
    /// Column to group by
    #[arg(long)]
    by: String,
    /// Numeric column to aggregate
    #[arg(long)]
    value: String,
}

/// Count, sum and mean of `value` per `by` group, largest sum first.
fn summarize(frame: LazyFrame, by: &str, value: &str) -> PolarsResult<DataFrame> {
    frame
        .group_by([col(by)])
        .agg([
            col(value).count().alias("count"),
            col(value).sum().alias("sum"),
            col(value).mean().alias("mean"),
        ])
        .sort(
            ["sum", by],
            SortMultipleOptions::default().with_order_descending_multi([true, false]),
        )
        .collect()
}

fn main() -> Result<()> {
    let args = Args::parse();
    let frame = LazyCsvReader::new(PlRefPath::try_from_path(&args.file)?)
        .with_has_header(true)
        .finish()?;
    println!("{}", summarize(frame, &args.by, &args.value)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_and_sorts_by_sum() {
        let frame = df!(
            "author" => ["bob", "alice", "bob", "carol"],
            "commits" => [3i64, 5, 4, 1],
        )
        .unwrap()
        .lazy();
        let out = summarize(frame, "author", "commits").unwrap();
        let authors: Vec<_> = out
            .column("author")
            .unwrap()
            .str()
            .unwrap()
            .iter()
            .map(Option::unwrap)
            .collect();
        let sums: Vec<_> = out
            .column("sum")
            .unwrap()
            .i64()
            .unwrap()
            .into_no_null_iter()
            .collect();
        assert_eq!(authors, ["bob", "alice", "carol"]);
        assert_eq!(sums, [7, 5, 1]);
    }
}
