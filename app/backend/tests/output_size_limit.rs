use backend::wasm::compiler::compile;
use backend::wasm::executor::execute;

async fn compile_and_execute(source: &str) -> backend::wasm::executor::ExecutionResult {
    let compile_result = compile(source).await.unwrap();

    assert_eq!(
        compile_result.status, 0,
        "compilation failed: {}",
        compile_result.stderr
    );

    execute(compile_result.bin.as_deref().unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn stdout_output_limit_ok() {
    let result = compile_and_execute(
        r#"
        fn main() {
            for _ in 0..1024 {
                println!("a");
            }
        }
        "#,
    )
    .await;

    assert!(
        result.stdout.len() <= 10 * 1024,
        "stdout exceeded limit: {} bytes",
        result.stdout.len()
    );
}

#[tokio::test]
async fn stdout_output_limit_fail() {
    let result = compile_and_execute(
        r#"
        fn main() {
            for _ in 0..(10 * 1024 * 2) {
                println!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
            }
        }
        "#,
    )
    .await;

    assert!(
        result.stdout.len() == 10 * 1024,
        "stdout exceeded limit: {} bytes",
        result.stdout.len()
    );
}
