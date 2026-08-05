use backend::wasm::compiler::compile_to_wasm;
use backend::wasm::executor::execute_wasm;

#[tokio::test]
async fn memory_limit_exceeded() {
    let source = r#"
        fn main() {
            // Allocate ~10 MB
            let mut data = Vec::new();

            for _ in 0..10 {
                data.extend(vec![0u8; 1024 * 1024]);
            }

            println!("allocated {} bytes", data.len());
        }
    "#;

    let compile_result = compile_to_wasm(source).await.unwrap();

    assert_eq!(
        compile_result.status, 0,
        "compile failed: {}",
        compile_result.stderr
    );

    let execution_result = execute_wasm(compile_result.bin.as_deref().unwrap()).await;

    dbg!(&execution_result);

    assert!(
        execution_result.unwrap().status == 1,
        "expected memory limit to be exceeded"
    );
}
