#![expect(clippy::print_stdout)]

use oxc::allocator::Allocator;
use oxc::ast::ast::Statement::ImportDeclaration;
use oxc::parser::{ParseOptions, Parser};
use oxc::span::SourceType;
use pico_args::Arguments;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn add_ext(dir: &str, p: &Path, set: &mut HashSet<String>) {
    for ext in ["index.ts", "index.tsx"] {
        let temp = Path::new(dir)
            .join(p)
            .join(ext)
            .canonicalize()
            .unwrap_or_default();
        set.insert(String::from(temp.to_str().unwrap()));
    }
}

fn setup_collections(dir: &str) -> (HashSet<String>, HashSet<String>) {
    let mut all_files_set = HashSet::new();
    let mut imports_queue_set: HashSet<String> = HashSet::new();

    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_entry(|e| e.file_name() != "node_modules")
    {
        let entry = entry.unwrap();
        let path = entry.path();
        // skip directories and all files that are NOT (JS|TS|JSX|TSX)
        println!("walking=====dir={:?}, path={:?}", entry, path);
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
        let full_path = Path::new(path).canonicalize();
        match full_path {
            Ok(p) => {
                all_files_set.insert(String::from(p.to_str().unwrap()));
            }
            Err(_e) => {}
        }

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
                    let path = Path::new(&source);
                    println!("path====={:?}", path.extension());
                    match path.extension() {
                        Some(ext)
                            if !["ts", "tsx", "js", "jsx"].contains(&ext.to_str().unwrap()) =>
                        {
                            continue;
                        }
                        None => {
                            add_ext(dir, path, &mut imports_queue_set);
                        }
                        _ => {
                            let x = Path::new(&dir)
                                .join(&source)
                                .canonicalize()
                                .unwrap_or_default();
                            imports_queue_set.insert(String::from(x.to_str().unwrap()));
                        }
                    }
                }
            }
        }
    }

    all_files_set.remove("");
    imports_queue_set.remove("");

    (all_files_set, imports_queue_set)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Arguments::from_env();
    println!("args====: {:?}", args);
    let dir: String = args
        .free_from_str()
        .expect("The first argument is required. Example: cargo run -- <valid directory>");
    println!("Working dir====: {}", dir);

    let (all_files_set, imports_queue_set) = setup_collections(&dir);

    println!("all files====={:?}", all_files_set);
    println!("queue====={:?}", imports_queue_set);

    Ok(())
}
