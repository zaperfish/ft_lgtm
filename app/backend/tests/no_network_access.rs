use backend::wasm::compiler::compile;
use backend::wasm::executor::execute;

#[tokio::test]
async fn no_network_access() {
    let source = r#"
        fn main() {
            let result = std::net::TcpStream::connect("example.com:80");

            match result {
                Ok(_) => {
                    println!("network access worked");
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("network blocked: {err}");
                    std::process::exit(1);
                }
            }
        }
    "#;

    let compile_result = compile(source).await.unwrap();

    assert_eq!(
        compile_result.status, 0,
        "compile failed: {}",
        compile_result.stderr
    );

    let execution_result = execute(compile_result.bin.as_deref().unwrap())
        .await
        .unwrap();

    assert_eq!(execution_result.status, 1, "expected network to be blocked");

    assert!(
        execution_result.stderr.contains("network blocked"),
        "unexpected stderr: {}",
        execution_result.stderr
    );
}
