use backend::wasm::compiler::compile_to_wasm;
use backend::wasm::executor::execute_wasm;

#[tokio::test]
async fn no_filesystem_access() {
    let source = r#"
        fn main() {
            let result = std::fs::read_to_string("/etc/passwd");

            match result {
                Ok(_) => {
                    println!("filesystem access worked");
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("filesystem access blocked: {err}");
                    std::process::exit(1);
                }
            }
        }
    "#;

    let compile_result = compile_to_wasm(source).await.unwrap();

    assert_eq!(
        compile_result.status, 0,
        "compile failed: {}",
        compile_result.stderr
    );

    let execution_result = execute_wasm(compile_result.bin.as_deref().unwrap())
        .await
        .unwrap();

    assert_eq!(execution_result.status, 1, "expected network to be blocked");

    assert!(
        execution_result
            .stderr
            .contains("filesystem access blocked"),
        "unexpected stderr: {}",
        execution_result.stderr
    );
}
