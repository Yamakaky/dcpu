use std::io::Write;
use std::process::*;

error_chain! {}

pub fn preprocess(asm: &str) -> Result<String> {
    let mut process = Command::new("cpp")
        .arg("-Wall")
        .args(&["-x", "assembler-with-cpp"])
        .arg("-nostdinc")
        .arg("-P")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .chain_err(|| "failed to execute process: {}\nIs gcc installed?")?;

    if let Some(ref mut stdin) = process.stdin {
        stdin.write_all(asm.as_bytes()).chain_err(|| "gcc input error")?
    } else {
        return Err("problem getting the assembler stdin".into())
    }
    let output = process.wait_with_output().chain_err(|| "gcc execution error")?;
    if output.status.success() {
        String::from_utf8(output.stdout).chain_err(|| "preprocessor output decoding")
    } else {
        Err("preprocessor error".into())
    }
}
