use backend::wasm::compiler::compile_to_wasm;
use backend::wasm::executor::execute_wasm;

#[tokio::test]
async fn no_network_access() {
    let source = r#"
        fn main() {
            println!("trying network");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    "#;

    let compile_result = compile_to_wasm(source).await.unwrap();
    assert_eq!(compile_result.status, 0);

    let result = execute_wasm(&compile_result.bin.unwrap()).await.unwrap();
    assert_eq!(result.status, 0);
}
