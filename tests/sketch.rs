#![allow(clippy::len_zero)]

use rye::{section_id, SectionId, Sections};

#[test]
fn sketch() {
    let sections = Sections::new();

    while !sections.completed() {
        let mut section = sections.root();

        println!("setup");

        {
            const SECTION: SectionId = section_id!("section1");
            if let Some(mut section) = section.new_section(SECTION) {
                println!("section1:setup");

                {
                    static SECTION: SectionId = section_id!("section2");
                    if let Some(_section) = section.new_section(SECTION) {
                        println!("section2");
                    }
                }

                {
                    static SECTION: SectionId = section_id!("section3");
                    if let Some(_section) = section.new_section(SECTION) {
                        println!("section3");
                    }
                }

                println!("section1:teardown");
            }
        }

        println!("test");

        {
            static SECTION: SectionId = section_id!("section4");
            if let Some(_section) = section.new_section(SECTION) {
                println!("section4");
            }
        }

        println!("teardown");
        println!("----------");
    }
}
