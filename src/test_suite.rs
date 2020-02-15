use crate::test_case::TestCase;

pub struct TestSuite<'a> {
    pub(crate) test_cases: &'a mut Vec<TestCase>,
}

impl TestSuite<'_> {
    #[doc(hidden)] // private API
    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }
}
