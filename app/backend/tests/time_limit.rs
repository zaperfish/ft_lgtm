use backend::wasm::compiler::compile_to_wasm;
use backend::wasm::executor::execute_wasm;

#[tokio::test]
async fn execution_time_limit() {
    let source = r#"
        fn main() {
            loop {
                // infinite loop
            }
        }
    "#;

    let compile_result = compile_to_wasm(source).await.unwrap();

    assert_eq!(
        compile_result.status, 0,
        "compilation failed: {}",
        compile_result.stderr
    );

    let result = execute_wasm(compile_result.bin.as_deref().unwrap()).await;

    dbg!(&result);

    assert!(
        result.unwrap().status == 1,
        "infinite program should have been stopped"
    );
}
