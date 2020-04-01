macro_rules! hidden_item {
    ( $( $item:item )* ) => {
        $(
            #[doc(hidden)]
            $item
        )*
    };
}

#[doc(hidden)]
#[cfg(not(feature = "harness"))]
#[macro_export]
macro_rules! __cfg_harness {
    ( $( $item:item )* ) => {};
}

#[doc(hidden)]
#[cfg(feature = "harness")]
#[macro_export]
macro_rules! __cfg_harness {
    ( $( $item:item )* ) =>  {
        $( $item )*
    };
}

#[doc(hidden)]
#[cfg(not(feature = "frameworks"))]
#[macro_export]
macro_rules! __cfg_frameworks {
    ( $( $item:item )* ) => {};
}

#[doc(hidden)]
#[cfg(feature = "frameworks")]
#[macro_export]
macro_rules! __cfg_frameworks {
    ( $( $item:item )* ) => {
        $( $item )*
    };
}
