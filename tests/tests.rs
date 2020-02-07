macro_rules! section {
    ($section:ident, $name:expr, $body:block) => {{
        static SECTION: rye::_internal::SectionId = rye::_internal::SectionId::SubSection {
            name: $name,
            file: file!(),
            line: line!(),
            column: column!(),
        };
        if let Some($section) = $section.new_section(&SECTION) {
            #[allow(unused_mut, unused_variables)]
            let mut $section = $section;
            $body
        }
    }};
}

#[test]
fn no_section() {
    let mut history = vec![];
    let test_case = rye::_internal::TestCase::new();
    while !test_case.completed() {
        #[allow(unused_mut, unused_variables)]
        let mut section = test_case.root_section();
        {
            history.push("test");
        }
    }
    assert_eq!(history, vec!["test"]);
}

#[test]
fn one_section() {
    let mut history = vec![];
    let test_case = rye::_internal::TestCase::new();
    while !test_case.completed() {
        #[allow(unused_mut, unused_variables)]
        let mut section = test_case.root_section();
        {
            history.push("setup");

            section!(section, "section1", {
                history.push("section1");
            });

            history.push("teardown");
        }
    }
    assert_eq!(history, vec!["setup", "section1", "teardown"]);
}

#[test]
fn multi_section() {
    let mut history = vec![];
    let test_case = rye::_internal::TestCase::new();
    while !test_case.completed() {
        #[allow(unused_mut, unused_variables)]
        let mut section = test_case.root_section();
        {
            history.push("setup");

            section!(section, "section1", {
                history.push("section1");
            });

            section!(section, "section2", {
                history.push("section2");
            });

            history.push("teardown");
        }
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
    let test_case = rye::_internal::TestCase::new();
    while !test_case.completed() {
        #[allow(unused_mut, unused_variables)]
        let mut section = test_case.root_section();
        {
            history.push("setup");

            section!(section, "section1", {
                history.push("section1:setup");

                section!(section, "section2", {
                    history.push("section2");
                });

                section!(section, "section3", {
                    history.push("section3");
                });

                history.push("section1:teardown");
            });

            history.push("test");

            section!(section, "section4", {
                history.push("section4");
            });

            history.push("teardown");
        }
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

#[test]
fn smoke_async() {
    use futures_test::future::FutureTestExt as _;
    futures_executor::block_on(async {
        let mut history = vec![];
        let test_case = rye::_internal::TestCase::new();
        while !test_case.completed() {
            let mut section = test_case.root_section();
            {
                history.push("setup");
                async {}.pending_once().await;

                section!(section, "section1", {
                    history.push("section1:setup");
                    async {}.pending_once().await;

                    section!(section, "section2", {
                        async {}.pending_once().await;
                        history.push("section2");
                    });

                    section!(section, "section3", {
                        async {}.pending_once().await;
                        history.push("section3");
                    });

                    async {}.pending_once().await;
                    history.push("section1:teardown");
                });

                history.push("test");
                async {}.pending_once().await;

                section!(section, "section4", {
                    async {}.pending_once().await;
                    history.push("section4");
                });

                async {}.pending_once().await;
                history.push("teardown");
            }
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
    });
}
