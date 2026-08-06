use backend::wasm::compiler::compile;
use backend::wasm::engine::WasmEngine;
use backend::wasm::executor::execute;

#[tokio::test]
async fn execution_time_limit() {
    let source = r#"
        fn main() {
            loop {
                // infinite loop
            }
        }
    "#;

    let compile_result = compile(source).await.unwrap();

    assert_eq!(
        compile_result.status, 0,
        "compilation failed: {}",
        compile_result.stderr
    );

    let result = execute(
        compile_result.bin.as_deref().unwrap(),
        &WasmEngine::default(),
    )
    .await;

    dbg!(&result);

    assert!(
        result.unwrap().status.is_err(),
        "infinite program should have been stopped"
    );
}
