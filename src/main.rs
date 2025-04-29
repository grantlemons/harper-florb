#![allow(unused)]

use std::fs::File;
use std::io::BufReader;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::{fs, process};

use anyhow::format_err;
use clap::Parser;
use harper_comments::CommentParser;
use harper_core::linting::{LintGroup, Linter};
use harper_core::parsers::{Markdown, MarkdownOptions};
use harper_core::{
    CharStringExt, Dialect, Dictionary, Document, FstDictionary, MergedDictionary,
    MutableDictionary, TokenKind, TokenStringExt, WordId, WordMetadata, remove_overlaps,
};
use harper_literate_haskell::LiterateHaskellParser;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(version, about)]
struct CliArgs {
    #[arg(short, long)]
    pub comment: Option<String>,
    #[arg(short, long, default_value_t = String::from(""))]
    pub delimiter: String,
    #[arg(short, long)]
    pub fix: bool,
    #[arg(short, long)]
    pub number: bool,
    #[arg(short, long)]
    pub silent: bool,
    #[arg(long, default_value_t = Dialect::American)]
    pub dialect: Dialect,

    pub file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let markdown_options = MarkdownOptions::default();
    let dictionary = FstDictionary::curated();

    let (document, source) = load_file(&args.file, markdown_options, &dictionary)?;
    let mut linter = LintGroup::new_curated(Arc::new(dictionary), args.dialect);

    todo!()
}

fn load_file(
    file: &Path,
    markdown_options: MarkdownOptions,
    dictionary: &impl Dictionary,
) -> anyhow::Result<(Document, String)> {
    let source = std::fs::read_to_string(file)?;

    let parser: Box<dyn harper_core::parsers::Parser> =
        match file.extension().map(|v| v.to_str().unwrap()) {
            Some("md") => Box::new(Markdown::default()),
            Some("lhs") => Box::new(LiterateHaskellParser::new_markdown(
                MarkdownOptions::default(),
            )),
            Some("typ") => Box::new(harper_typst::Typst),
            _ => Box::new(
                CommentParser::new_from_filename(file, markdown_options)
                    .map(Box::new)
                    .ok_or(format_err!("Could not detect language ID."))?,
            ),
        };

    Ok((Document::new(&source, &parser, dictionary), source))
}

/// Sync version of harper-ls/src/dictionary_io@load_dict
fn load_dict(path: &Path) -> anyhow::Result<MutableDictionary> {
    let str = fs::read_to_string(path)?;

    let mut dict = MutableDictionary::new();
    dict.extend_words(
        str.lines()
            .map(|l| (l.chars().collect::<Vec<_>>(), WordMetadata::default())),
    );

    Ok(dict)
}

/// Path version of harper-ls/src/dictionary_io@file_dict_name
fn file_dict_name(path: &Path) -> PathBuf {
    let mut rewritten = String::new();

    for seg in path.components() {
        if !matches!(seg, Component::RootDir) {
            rewritten.push_str(&seg.as_os_str().to_string_lossy());
            rewritten.push('%');
        }
    }

    rewritten.into()
}
