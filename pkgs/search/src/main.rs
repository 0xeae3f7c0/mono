//! Full-text search over Markdown files with tantivy.
//!
//!   search <dir> <query>

use std::path::Path;

use anyhow::Result;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{STORED, Schema, TEXT, Value};
use tantivy::{Index, TantivyDocument, doc};

/// Indexes (name, body) pairs in RAM and returns names of the best `limit` matches.
fn search(docs: &[(String, String)], query: &str, limit: usize) -> Result<Vec<String>> {
    let mut schema = Schema::builder();
    let name = schema.add_text_field("name", STORED);
    let body = schema.add_text_field("body", TEXT);
    let index = Index::create_in_ram(schema.build());

    let mut writer = index.writer(50_000_000)?;
    for (n, b) in docs {
        writer.add_document(doc!(name => n.as_str(), body => b.as_str()))?;
    }
    writer.commit()?;

    let searcher = index.reader()?.searcher();
    let query = QueryParser::for_index(&index, vec![body]).parse_query(query)?;
    let mut names = Vec::new();
    for (_score, address) in
        searcher.search(&query, &TopDocs::with_limit(limit).order_by_score())?
    {
        let found: TantivyDocument = searcher.doc(address)?;
        names.extend(
            found
                .get_first(name)
                .and_then(|v| v.as_str())
                .map(str::to_owned),
        );
    }
    Ok(names)
}

fn markdown_files(dir: &Path) -> Result<Vec<(String, String)>> {
    let mut docs = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "md") {
            docs.push((path.display().to_string(), std::fs::read_to_string(&path)?));
        }
    }
    Ok(docs)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [dir, query] = args.as_slice() else {
        anyhow::bail!("usage: search <dir> <query>");
    };
    for name in search(&markdown_files(Path::new(dir))?, query, 10)? {
        println!("{name}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_matching_documents() {
        let docs = vec![
            (
                "rust.md".to_owned(),
                "Rust is a systems language".to_owned(),
            ),
            ("go.md".to_owned(), "Go has goroutines".to_owned()),
            (
                "both.md".to_owned(),
                "Rust and Go are both compiled".to_owned(),
            ),
        ];
        let mut found = search(&docs, "rust", 10).unwrap();
        found.sort();
        assert_eq!(found, ["both.md", "rust.md"]);
        assert!(search(&docs, "python", 10).unwrap().is_empty());
    }
}
