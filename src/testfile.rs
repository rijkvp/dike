use crate::error::Error;
use std::{fs, path::PathBuf};
use tracing::error;

const INPUT_EXT: &str = "in";
const OUTPUT_EXT: &str = "out";

#[derive(Debug, Clone)]
pub struct TestFile {
    name: String,
    path: PathBuf,
}

impl TestFile {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        Self { name, path }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_input(&self) -> Result<String, Error> {
        Ok(fs::read_to_string(self.path.with_extension(INPUT_EXT))?)
    }

    pub fn get_output(&self) -> Result<String, Error> {
        Ok(fs::read_to_string(self.path.with_extension(OUTPUT_EXT))?)
    }
}

pub fn read_tests(test_dir: PathBuf) -> Result<Vec<TestFile>, Error> {
    let mut input_files = Vec::<PathBuf>::new();
    for entry in fs::read_dir(&test_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_str() == Some(INPUT_EXT) {
                    input_files.push(path);
                }
            }
        }
    }
    let mut tests = Vec::<TestFile>::new();
    for input_path in input_files {
        let output_path = input_path.with_extension(OUTPUT_EXT);
        if output_path.exists() {
            tests.push(TestFile::new( 
                input_path.with_extension(""),
            ));
        } else {
            error!("Input file {input_path:?} has no corresponding output {output_path:?}");
        }
    }
    return Ok(tests);
}
