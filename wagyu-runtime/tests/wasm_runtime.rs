use std::{ffi::OsStr, fs, io::Result};

use wagyu_runtime::*;

#[test]
/// # Panics
fn parse_wasm_files() {
    let entries = fs::read_dir("tests/wasm").expect("`tests/wasm` directory must exist");

    let mut file_paths: Vec<_> = entries
        .filter_map(Result::ok)
        .filter_map(|x| x.path().is_file().then_some(x.path()))
        .collect();

    file_paths.sort();

    let valid_file_paths = file_paths.iter().filter(|p| {
        p.file_name()
            .and_then(OsStr::to_str)
            .map(|_| true)
            .is_some_and(|p| p)
    });

    // test for valid files
    let mut err_messages = vec![];
    for file_path in valid_file_paths {
        let buffer = fs::read(file_path).expect("failed to read a file");

        println!("compiling: {file_path:?}");

        if let Err(e) = compile(&buffer) {
            err_messages.push(format!("{}) {file_path:?} {e}", err_messages.len() + 1));
        }
    }

    // summary
    let err_msg = err_messages.join("\n\n");
    assert!(err_msg.is_empty(), "{err_msg}");
}
