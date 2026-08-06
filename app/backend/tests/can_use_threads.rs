use backend::wasm::compiler::compile;
use backend::wasm::engine::WasmEngine;
use backend::wasm::executor::execute;

#[tokio::test]
async fn no_network_access() {
    let source = r#"
        fn main() {
            println!("trying network");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    "#;

    let compile_result = compile(source).await.unwrap();
    assert_eq!(compile_result.status, 0);

    let result = execute(&compile_result.bin.unwrap(), &WasmEngine::default())
        .await
        .unwrap();
    assert!(result.status.is_ok());
}
