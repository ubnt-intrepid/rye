use std::fmt;

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Location {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[doc(hidden)] // private API.
#[macro_export]
macro_rules! __location {
    () => {
        $crate::_internal::Location {
            file: file!(),
            line: line!(),
            column: column!(),
        }
    };
}