use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use w::parser::Parser;
use w::rust_codegen::RustCodeGenerator;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn benchmarks_dir() -> PathBuf {
    project_root().join("benchmarks")
}

fn programs_dir() -> PathBuf {
    benchmarks_dir().join("programs")
}

fn compile_w_program(source_path: &PathBuf) -> Result<PathBuf, String> {
    let source = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read source file: {}", e))?;

    let mut parser = Parser::new(source);
    let expr = parser.parse().ok_or("Failed to parse W program")?;

    let mut codegen = RustCodeGenerator::new();
    let rust_code = codegen
        .generate(&expr)
        .map_err(|e| format!("Failed to generate Rust code: {:?}", e))?;

    let rust_file = source_path.with_extension("rs");
    let mut file =
        fs::File::create(&rust_file).map_err(|e| format!("Failed to create Rust file: {}", e))?;
    file.write_all(rust_code.as_bytes())
        .map_err(|e| format!("Failed to write Rust file: {}", e))?;

    let binary = source_path.with_extension("");

    let status = Command::new("rustc")
        .arg(&rust_file)
        .arg("-o")
        .arg(&binary)
        .status()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    if !status.success() {
        return Err("Rust compilation failed".to_string());
    }

    Ok(binary)
}

fn run_program(binary: &PathBuf, args: &[&str]) -> Result<String, String> {
    let output = Command::new(binary)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run program: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Program exited with non-zero status: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn compile_and_run(source_name: &str, args: &[&str]) -> Result<String, String> {
    let source_path = programs_dir().join(source_name);
    let binary = compile_w_program(&source_path)?;
    let result = run_program(&binary, args);

    let _ = fs::remove_file(&binary);
    let _ = fs::remove_file(binary.with_extension("rs"));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fannkuch_redux() {
        let result = compile_and_run("fannkuch_redux.w", &["7"]);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
        let output = result.unwrap();
        assert!(
            output.contains("7"),
            "Output should contain '7': {}",
            output
        );
    }

    #[test]
    fn test_binary_trees() {
        let result = compile_and_run("binary_trees.w", &["10"]);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn test_spectral_norm() {
        let result = compile_and_run("spectral_norm.w", &["100"]);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    #[test]
    fn test_n_body() {
        let result = compile_and_run("n_body.w", &["1000"]);
        assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }
}
