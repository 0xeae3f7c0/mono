// reports-pdf: Renders the report to PDF (commit platform, pkgs/reports-pdf)
//! Renders the weekly activity report (a Typst document) to PDF using the
//! bundled fonts. `pipelines` produces the .typ file; `archive` uploads the PDF.
//!
//!   typeset report.typ report.pdf

use std::path::Path;

use anyhow::{Context, Result, anyhow};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;

/// A single-file world: one main source plus the fonts shipped in typst-assets.
struct SingleFile {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
}

impl SingleFile {
    fn new(text: String) -> Self {
        let fonts: Vec<Font> = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::new(data)))
            .collect();
        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            main: Source::new(
                FileId::new(RootedPath::new(
                    VirtualRoot::Project,
                    VirtualPath::new("main.typ").expect("valid path"),
                )),
                text,
            ),
        }
    }
}

impl World for SingleFile {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }
    fn main(&self) -> FileId {
        self.main.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::AccessDenied)
        }
    }
    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::AccessDenied)
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }
    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

/// Compiles Typst markup to PDF bytes.
fn render(text: &str) -> Result<Vec<u8>> {
    let world = SingleFile::new(text.to_owned());
    let document: PagedDocument = typst::compile(&world).output.map_err(|errors| {
        anyhow!(
            "{}",
            errors
                .iter()
                .map(|e| e.message.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        )
    })?;
    typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .map_err(|errors| anyhow!("{} PDF export error(s)", errors.len()))
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        anyhow::bail!("usage: typeset <input.typ> <output.pdf>");
    };
    let text = std::fs::read_to_string(&input).with_context(|| format!("reading {input}"))?;
    std::fs::write(Path::new(&output), render(&text)?)
        .with_context(|| format!("writing {output}"))?;
    println!("wrote {output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_pdf() {
        let pdf = render("= Report\nHello *world*.").unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
    }

    #[test]
    fn reports_syntax_errors() {
        let error = render("#let x = ").unwrap_err().to_string();
        assert!(!error.is_empty());
    }
}
