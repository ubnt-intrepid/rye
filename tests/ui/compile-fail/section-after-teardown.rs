fn main() {}

#[rye::test]
fn section_after_teardown(cx: &mut rye::Context<'_>) {
    println!("startup1");
    println!("startup2");

    section!(cx, "section1", {});
    section!(cx, "section2", {});

    println!("teardown1");
    println!("teardown2");

    section!(cx, "section3", {});
}

#[rye::test]
fn section_after_teardown_nested(cx: &mut rye::Context<'_>) {
    section!(cx, "root", {
        println!("startup1");
        println!("startup2");

        section!(cx, "section1", {});
        section!(cx, "section2", {});

        println!("teardown1");
        println!("teardown2");

        section!(cx, "section3", {});
    });
}
