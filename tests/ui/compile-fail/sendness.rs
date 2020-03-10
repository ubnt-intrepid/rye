fn main() {}

#[rye::test(non_send)]
async fn incorrect_arg() {
    let rc = std::rc::Rc::new(());
    (async {}).await;
    drop(rc);
}

#[rye::test(?Send)]
fn nosend_in_sync_fn() {}
