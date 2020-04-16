use std::time::Duration;
use tokio::{task, time::delay_for};

rye_harness::test_harness!();

#[rye::test_main]
#[rye(runtime = rye_runtime_tokio::runtime)]
async fn test_main(sess: &mut rye::Session<'_>) -> anyhow::Result<()> {
    sess.run().await?;
    Ok(())
}

#[rye::test]
async fn with_timer(_: &mut rye::Context<'_>) {
    delay_for(Duration::from_millis(10)).await;
}

#[rye::test(?Send)]
async fn nonsend(_: &mut rye::Context<'_>) {
    let _ = std::rc::Rc::new(());
    task::yield_now().await;
}

#[rye::test]
fn blocking(_: &mut rye::Context<'_>) {
    std::thread::sleep(Duration::from_millis(10));
}

#[rye::test]
async fn spawn(_: &mut rye::Context<'_>) {
    let _ = task::spawn(delay_for(Duration::from_millis(10))).await;
    let _ = task::spawn_blocking(|| std::thread::sleep(Duration::from_millis(10))).await;
}

#[rye::test(?Send)]
async fn spawn_local(_: &mut rye::Context<'_>) {
    let _ = task::spawn_local(delay_for(Duration::from_millis(10))).await;
}
