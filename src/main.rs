#![expect(clippy::print_stdout)]

use oxc::allocator::Allocator;
use oxc::ast::ast::Statement::ImportDeclaration;
use oxc::parser::{ParseOptions, Parser};
use oxc::span::SourceType;
use pico_args::Arguments;
use std::fs;
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Arguments::from_env();
    println!("args====: {:?}", args);
    let dir: String = args
        .free_from_str()
        .expect("please provide a directory path");
    println!("Working dir====: {}", dir);

    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_entry(|e| e.file_name() != "node_modules")
    {
        let entry = entry.unwrap();
        let path = entry.path();
        // skip directories and all files that are NOT (JS|TS|JSX|TSX)
        println!("extension====={:?}", path.extension());
        match path.extension() {
            None => continue,
            Some(e) => match e.to_str() {
                None => panic!("failed to convert OsStr"),
                Some(r) => {
                    if !["ts", "tsx", "js", "jsx"].contains(&r) {
                        continue;
                    }
                }
            },
        }
        println!("walking====={:?}, {:?}", entry, path);

        let source_text = fs::read_to_string(path)?;

        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();

        let ret = Parser::new(&allocator, &source_text, source_type)
            .with_options(ParseOptions::default())
            .parse();

        for stmt in &ret.program.body {
            if let ImportDeclaration(import) = stmt {
                println!("Import from: {}", import.source);
            }
        }
    }

    Ok(())
}
