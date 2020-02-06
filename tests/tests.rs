use rye::TestCase;

macro_rules! section {
    ($name:expr, $body:block) => {{
        static SECTION: rye::_internal::SectionId = rye::_internal::SectionId::SubSection {
            name: $name,
            file: file!(),
            line: line!(),
            column: column!(),
        };
        if let Some(section) = rye::_internal::new_section(&SECTION) {
            let _guard = rye::_internal::Guard::set(Some(Box::new(section)));
            $body
        }
    }};
}

#[test]
fn no_section() {
    let mut history = vec![];
    let mut test_case = TestCase::new();
    while !test_case.completed() {
        test_case.run(|| {
            history.push("test");
        });
    }
    assert_eq!(history, vec!["test"]);
}

#[test]
fn one_section() {
    let mut history = vec![];
    let mut test_case = TestCase::new();
    while !test_case.completed() {
        test_case.run(|| {
            history.push("setup");

            section!("section1", {
                history.push("section1");
            });

            history.push("teardown");
        });
    }
    assert_eq!(history, vec!["setup", "section1", "teardown"]);
}

#[test]
fn multi_section() {
    let mut history = vec![];
    let mut test_case = TestCase::new();
    while !test_case.completed() {
        test_case.run(|| {
            history.push("setup");

            section!("section1", {
                history.push("section1");
            });

            section!("section2", {
                history.push("section2");
            });

            history.push("teardown");
        });
    }
    assert_eq!(
        history,
        vec![
            // phase 1
            "setup", "section1", "teardown", //
            // phase 2
            "setup", "section2", "teardown",
        ]
    );
}

#[test]
fn nested_section() {
    let mut history = vec![];
    let mut test_case = TestCase::new();
    while !test_case.completed() {
        test_case.run(|| {
            history.push("setup");

            section!("section1", {
                history.push("section1:setup");

                section!("section2", {
                    history.push("section2");
                });

                section!("section3", {
                    history.push("section3");
                });

                history.push("section1:teardown");
            });

            history.push("test");

            section!("section4", {
                history.push("section4");
            });

            history.push("teardown");
        });
    }
    assert_eq!(
        history,
        vec![
            // phase 1
            "setup",
            "section1:setup",
            "section2",
            "section1:teardown",
            "test",
            "teardown",
            // phase 2
            "setup",
            "section1:setup",
            "section3",
            "section1:teardown",
            "test",
            "teardown",
            // phase 3
            "setup",
            "test",
            "section4",
            "teardown",
        ]
    );
}

#[cfg(feature = "futures")]
#[test]
fn smoke_async() {
    use futures_test::future::FutureTestExt as _;

    let mut history = vec![];
    let mut test_case = TestCase::new();
    while !test_case.completed() {
        futures_executor::block_on(test_case.run_async(async {
            history.push("setup");
            async {}.pending_once().await;

            section!("section1", {
                history.push("section1:setup");
                async {}.pending_once().await;

                section!("section2", {
                    async {}.pending_once().await;
                    history.push("section2");
                });

                section!("section3", {
                    async {}.pending_once().await;
                    history.push("section3");
                });

                async {}.pending_once().await;
                history.push("section1:teardown");
            });

            history.push("test");
            async {}.pending_once().await;

            section!("section4", {
                async {}.pending_once().await;
                history.push("section4");
            });

            async {}.pending_once().await;
            history.push("teardown");
        }));
    }

    assert_eq!(
        history,
        vec![
            // phase 1
            "setup",
            "section1:setup",
            "section2",
            "section1:teardown",
            "test",
            "teardown",
            // phase 2
            "setup",
            "section1:setup",
            "section3",
            "section1:teardown",
            "test",
            "teardown",
            // phase 3
            "setup",
            "test",
            "section4",
            "teardown",
        ]
    );
}
