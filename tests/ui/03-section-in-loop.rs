fn main() {}

#[rye::test]
fn section_in_loop() {
    loop {
        section!("section", {
            assert!(true);
        });
    }
}

#[rye::test]
fn section_in_for_loop() {
    for _ in 0..10 {
        section!("section", {
            assert!(true);
        });
    }
}

#[rye::test]
fn section_in_while_loop() {
    while false {
        section!("section", {
            assert!(true);
        });
    }
}
