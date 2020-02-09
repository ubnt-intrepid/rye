use crate::{context::TestContext, section::Section};

/// Metadata for executing a test case.
#[derive(Debug)]
pub struct TestCase {
    pub sections: &'static [Section],
}

impl TestCase {
    fn running_sections(&self) -> impl Iterator<Item = &Section> + '_ {
        enum Either<A, B> {
            A(A),
            B(B),
        }

        impl<A, B, T> Iterator for Either<A, B>
        where
            A: Iterator<Item = T>,
            B: Iterator<Item = T>,
        {
            type Item = T;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Self::A(a) => a.next(),
                    Self::B(b) => b.next(),
                }
            }
        }

        if self.sections.is_empty() {
            Either::A(Some(&Section::ROOT).into_iter())
        } else {
            Either::B(self.sections.iter().filter(|section| section.is_leaf()))
        }
    }

    #[inline]
    pub fn run<F>(&self, f: F)
    where
        F: Fn(),
    {
        for section in self.running_sections() {
            TestContext::new(section).scope(&f);
        }
    }

    #[cfg(feature = "futures")]
    #[inline]
    pub async fn run_async<F, Fut>(&self, f: F)
    where
        F: Fn() -> Fut,
        Fut: futures_core::Future<Output = ()>,
    {
        for section in self.running_sections() {
            TestContext::new(section).scope_async(f()).await;
        }
    }
}
