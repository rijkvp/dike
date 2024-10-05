use std::{fs, path::Path};

use crate::error::Error;
use glob::glob;

const INPUT_EXT: &str = "in";
const OUTPUT_EXT: &str = "out";

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub input: Vec<u8>,
    pub output: Vec<u8>,
}

fn load_test(path: &Path) -> Result<Option<TestCase>, Error> {
    if path.extension().map(|e| e.to_str()) == Some(Some(INPUT_EXT)) {
        let output_path = path.with_extension(OUTPUT_EXT);
        if output_path.exists() {
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();
            let input = fs::read(path)?;
            let output = fs::read(output_path)?;
            return Ok(Some(TestCase {
                name,
                input,
                output,
            }));
        }
    }
    Ok(None)
}

pub fn load_tests(input_glob: String) -> Result<Vec<TestCase>, Error> {
    let mut tests = Vec::<TestCase>::new();
    for entry in glob(&input_glob)? {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    if let Some(test) = load_test(&path)? {
                        tests.push(test);
                    } else {
                        log::info!("Skip file: {path:?}");
                    }
                }
            }
            Err(e) => log::error!("Failed to read path: {e:?}"),
        }
    }
    // let mut input_files = Vec::<PathBuf>::new();
    // for entry in fs::read_dir(&test_dir)? {
    //     let entry = entry?;
    //     let path = entry.path();
    //     if path.is_file() {
    //         if let Some(ext) = path.extension() {
    //             if ext.to_str() == Some(INPUT_EXT) {
    //                 input_files.push(path);
    //             }
    //         }
    //     }
    // }
    // for input_path in input_files {
    //     let output_path = input_path.with_extension(OUTPUT_EXT);
    //     if output_path.exists() {
    //         tests.push(TestCase::new(input_path.with_extension("")));
    //     } else {
    //         error!("Input file {input_path:?} has no corresponding output {output_path:?}");
    //     }
    // }
    return Ok(tests);
}
