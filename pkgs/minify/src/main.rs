//! Minifies the embeddable `widget` script with SWC before it is published,
//! keeping the snippet sites paste into their pages small.
//!
//!   minify input.js > output.min.js

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::sync::Lrc;
use swc_core::common::{FileName, GLOBALS, Globals, Mark, SourceMap};
use swc_core::ecma::ast::Program;
use swc_core::ecma::codegen::{Config, Emitter, text_writer::JsWriter};
use swc_core::ecma::minifier::optimize;
use swc_core::ecma::minifier::option::{ExtraOptions, MangleOptions, MinifyOptions};
use swc_core::ecma::parser::{Syntax, parse_file_as_program};
use swc_core::ecma::transforms::base::{fixer::fixer, resolver};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// JavaScript file to minify
    file: std::path::PathBuf,
}

fn minify(source: &str) -> Result<String> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(Lrc::new(FileName::Anon), source.to_owned());
    GLOBALS.set(&Globals::new(), || {
        let unresolved = Mark::new();
        let top_level = Mark::new();
        let comments = SingleThreadedComments::default();
        let program = parse_file_as_program(
            &fm,
            Syntax::Es(Default::default()),
            Default::default(),
            Some(&comments),
            &mut vec![],
        )
        .map_err(|e| anyhow!("parse error: {:?}", e.kind()))?
        .apply(resolver(unresolved, top_level, false));

        let program: Program = optimize(
            program,
            cm.clone(),
            Some(&comments),
            None,
            &MinifyOptions {
                compress: Some(Default::default()),
                mangle: Some(MangleOptions {
                    top_level: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
            &ExtraOptions {
                unresolved_mark: unresolved,
                top_level_mark: top_level,
                mangle_name_cache: None,
            },
        )
        .apply(fixer(None));

        let mut out = Vec::new();
        Emitter {
            cfg: Config::default().with_minify(true),
            cm: cm.clone(),
            comments: None,
            wr: JsWriter::new(cm.clone(), "\n", &mut out, None),
        }
        .emit_program(&program)
        .context("emitting code")?;
        Ok(String::from_utf8(out)?)
    })
}

fn main() -> Result<()> {
    let args = Args::parse();
    let source = std::fs::read_to_string(&args.file)
        .with_context(|| format!("reading {}", args.file.display()))?;
    println!("{}", minify(&source)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinks_and_keeps_behaviour() {
        let src = "function add(first, second) {\n  // sum\n  return first + second;\n}\nconsole.log(add(1, 2));\n";
        let out = minify(src).unwrap();
        assert!(out.len() < src.len() * 2 / 3, "{out}");
        assert!(
            !out.contains("first") && !out.contains("second"),
            "params not mangled: {out}"
        );
        assert!(
            out.contains("console.log") && !out.contains("// sum"),
            "{out}"
        );
    }

    #[test]
    fn syntax_errors_are_reported() {
        assert!(minify("function (").is_err());
    }
}
