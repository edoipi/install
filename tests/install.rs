extern crate install;
extern crate rustc;

use std::io::process::Command;
use std::io::File;
use std::io::fs::{unlink, PathExtensions};

static EXE: &'static str = "target/install";

fn cleanup(filename: &'static str) {
    let path = Path::new(filename);
    if path.exists() {
        unlink(&path).unwrap();
    }
}

#[test]
fn test_install_file_to_file() {
    let status = match Command::new(EXE).arg("tests/original").arg("tests/copy").status() {
        Ok(status) => status,
        Err(e) => panic!("failed to execute process: {}", e),
    }.success();
    
    assert_eq!(status, true);

    let contents = File::open(&Path::new("tests/copy")).read_to_string().unwrap();           
    assert_eq!(contents.as_slice(), "Interesting content.");

    cleanup("tests/copy");
}

#[test]
fn test_install_files_to_directory() {
    let status = match Command::new(EXE).arg("tests/source/a").arg("tests/original").arg("tests/source/b").arg("tests/dest").status() {
        Ok(status) => status,
        Err(e) => panic!("failed to execute process: {}", e),
    }.success();
    
    assert_eq!(status, true);
    
    let mut contents = File::open(&Path::new("tests/dest/a")).read_to_string().unwrap();
    assert_eq!(contents.as_slice(), "A content.");
    
    contents = File::open(&Path::new("tests/dest/b")).read_to_string().unwrap();
    assert_eq!(contents.as_slice(), "B content.");
    
    contents = File::open(&Path::new("tests/dest/original")).read_to_string().unwrap();
    assert_eq!(contents.as_slice(), "Interesting content.");
    
    cleanup("tests/dest/a");
    cleanup("tests/dest/b");
    cleanup("tests/dest/original");
}

#[test]
fn test_install_target_flag() {
    let status = match Command::new(EXE).arg("tests/source/a").arg("-t").arg("tests/dest").arg("tests/original").arg("tests/source/b").status() {
        Ok(status) => status,
        Err(e) => panic!("failed to execute process: {}", e),
    }.success();
    
    assert_eq!(status, true);
    
    let mut contents = File::open(&Path::new("tests/dest/a")).read_to_string().unwrap();
    assert_eq!(contents.as_slice(), "A content.");
    
    contents = File::open(&Path::new("tests/dest/b")).read_to_string().unwrap();
    assert_eq!(contents.as_slice(), "B content.");
    
    contents = File::open(&Path::new("tests/dest/original")).read_to_string().unwrap();
    assert_eq!(contents.as_slice(), "Interesting content.");
    
    cleanup("tests/dest/a");
    cleanup("tests/dest/b");
    cleanup("tests/dest/original");
}
