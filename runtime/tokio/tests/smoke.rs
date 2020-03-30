use std::time::Duration;
use tokio::time::delay_for;

rye::test_harness!(runtime = rye_runtime_tokio::runtime);

#[rye::test]
async fn with_timer(_: &mut rye::Context<'_>) {
    delay_for(Duration::from_millis(10)).await;
}

#[rye::test]
fn blocking(_: &mut rye::Context<'_>) {
    std::thread::sleep(Duration::from_millis(10));
}
