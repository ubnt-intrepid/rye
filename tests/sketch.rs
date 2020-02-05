#![allow(clippy::len_zero)]

#[test]
fn sketch() {
    rye::test_case(|| {
        println!("setup");

        rye::section!("section1", {
            println!("section1:setup");

            rye::section!("section2", {
                println!("section2");
            });

            rye::section!("section3", {
                println!("section3");
            });

            println!("section1:teardown");
        });

        println!("test");

        rye::section!("section4", {
            println!("section4");
        });

        println!("teardown");
        println!("----------");
    });
}

#[async_std::test]
async fn sketch_async() {
    use futures_test::future::FutureTestExt as _;

    rye::test_case_async(|| async {
        println!("setup");
        async {}.pending_once().await;

        rye::section!("section1", {
            println!("section1:setup");
            async {}.pending_once().await;

            rye::section!("section2", {
                async {}.pending_once().await;
                println!("section2");
            });

            rye::section!("section3", {
                async {}.pending_once().await;
                println!("section3");
            });

            async {}.pending_once().await;
            println!("section1:teardown");
        });

        println!("test");
        async {}.pending_once().await;

        rye::section!("section4", {
            async {}.pending_once().await;
            println!("section4");
        });

        async {}.pending_once().await;
        println!("teardown");
        println!("----------");
    })
    .await;
}
