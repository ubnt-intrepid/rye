#[doc(no_inline)]
pub use rye::*;

#[doc(hidden)]
#[macro_export]
macro_rules! __test_case {
    ( $item:item ) => {
        #[test_case]
        $item
    };
}
