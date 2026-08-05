use backend::wasm::compiler::compile_to_wasm;

#[tokio::test]
async fn long_compile_timeout() {
    let source = r#"
        #![allow(long_running_const_eval)]

        const fn fib(n: u64) -> u64 {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }

        const _: u64 = fib(60);

        fn main() {}
    "#;

    let compile_result = compile_to_wasm(source).await.unwrap();

    assert_ne!(
        compile_result.status, 0,
        "expected compilation timeout, but compilation succeeded"
    );

    assert!(
        compile_result.stderr.contains("timed out")
            || compile_result.stderr.contains("killed")
            || compile_result.status == 124,
        "unexpected compile result: status={}, stderr={}",
        compile_result.status,
        compile_result.stderr
    );
}
