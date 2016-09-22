use std::io::Write;
use std::process::*;

pub fn preprocess(asm: &str) -> Option<String> {
    let mut process = Command::new("cpp")
        .arg("-Wall")
        .args(&["-x", "assembler-with-cpp"])
        .arg("-nostdinc")
        .arg("-P")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn().unwrap_or_else(|e| panic!("failed to execute process: {}\nIs gcc installed?", e));

    if let Some(ref mut stdin) = process.stdin {
        if stdin.write_all(asm.as_bytes()).is_err() {
            return None;
        }
    } else {
        return None;
    }
    let output = process.wait_with_output().unwrap();
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}
