use backend::wasm::compiler::compile_to_wasm;
use backend::wasm::executor::execute_wasm;

async fn compile_and_execute(source: &str) -> backend::wasm::executor::ExecutionResult {
    let compile_result = compile_to_wasm(source).await.unwrap();

    assert_eq!(
        compile_result.status, 0,
        "compilation failed: {}",
        compile_result.stderr
    );

    execute_wasm(compile_result.bin.as_deref().unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn exit_code_zero() {
    let result = compile_and_execute(
        r#"
        fn main() {
            println!("success");
            std::process::exit(0);
        }
        "#,
    )
    .await;

    assert_eq!(result.status, 0);
    assert_eq!(result.stdout, "success\n");
}

#[tokio::test]
async fn exit_code_one() {
    let result = compile_and_execute(
        r#"
        fn main() {
            eprintln!("failed");
            std::process::exit(1);
        }
        "#,
    )
    .await;

    assert_eq!(result.status, 1);
    assert_eq!(result.stderr, "failed\n");
}

#[tokio::test]
async fn exit_code_custom() {
    let result = compile_and_execute(
        r#"
        fn main() {
            eprintln!("custom exit");
            std::process::exit(42);
        }
        "#,
    )
    .await;

    // Somehow wasmtime turns nonzero exit codes to 1 always.
    assert_eq!(result.status, 1);
    assert_eq!(result.stderr, "custom exit\n");
}
