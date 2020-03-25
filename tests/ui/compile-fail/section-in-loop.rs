fn main() {}

#[rye::test]
fn section_in_loop(cx: &mut rye::Context<'_>) {
    loop {
        section!(cx, "section", {});
    }
}

#[rye::test]
fn section_in_for_loop(cx: &mut rye::Context<'_>) {
    for _ in 0..10 {
        section!(cx, "section", {});
    }
}

#[rye::test]
fn section_in_while_loop(cx: &mut rye::Context<'_>) {
    while false {
        section!(cx, "section", {});
    }
}
