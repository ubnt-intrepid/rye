#[cfg(frameworks)]
#[doc(hidden)]
#[macro_export]
macro_rules! test_case {
    ($($item:item)*) => {
        $(
            #[test_case]
            $item
        )*
    };
}

#[cfg(not(frameworks))]
#[doc(hidden)]
#[macro_export]
macro_rules! test_case {
    ($($t:tt)*) => {
        compile_error!("custom_test_frameworks is not available on this platform");
    };
}
