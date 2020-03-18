fn main() {}

#[rye::test]
fn section_after_teardown() {
    println!("startup1");
    println!("startup2");

    section!("section1", {});
    section!("section2", {});

    println!("teardown1");
    println!("teardown2");

    section!("section3", {});
}

#[rye::test]
fn section_after_teardown_nested() {
    section!("root", {
        println!("startup1");
        println!("startup2");

        section!("section1", {});
        section!("section2", {});

        println!("teardown1");
        println!("teardown2");

        section!("section3", {});
    });
}
