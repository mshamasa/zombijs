#![expect(clippy::print_stdout)]

use oxc::allocator::Allocator;
use oxc::ast::ast::Statement::ImportDeclaration;
use oxc::parser::{ParseOptions, Parser};
use oxc::span::SourceType;
use pico_args::Arguments;
use std::collections::HashSet;
use std::fs;
use walkdir::WalkDir;

fn setup_collections(dir: &str) -> (HashSet<String>, Vec<String>) {
    let mut all_files_set = HashSet::new();
    let mut imports_set = HashSet::new();

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
        let p = path.to_str().unwrap_or_default();
        all_files_set.insert(p.to_string());

        let source_text = fs::read_to_string(path).unwrap_or_default();

        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();

        let ret = Parser::new(&allocator, &source_text, source_type)
            .with_options(ParseOptions::default())
            .parse();

        for stmt in &ret.program.body {
            if let ImportDeclaration(import) = stmt {
                let source = import.source.to_string();
                println!("Import from: {}", source);
                // todo - imports need to be full paths so dir + source
                // todo - check path against common shorthand imports
                // ./components/Card === ./components/Card/index.(ts|tsx)
                if source.starts_with(".") {
                    imports_set.insert(source);
                }
            }
        }
    }

    let imports_queue = imports_set.into_iter().collect();

    (all_files_set, imports_queue)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Arguments::from_env();
    println!("args====: {:?}", args);
    let dir: String = args
        .free_from_str()
        .expect("The first argument is required. Example: cargo run -- <valid directory>");
    println!("Working dir====: {}", dir);

    let (all_files_set, imports_queue) = setup_collections(&dir);

    println!("all files====={:?}", all_files_set);
    println!("queue====={:?}", imports_queue);

    Ok(())
}
