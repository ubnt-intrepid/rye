macro_rules! hidden_item {
    ( $( $item:item )* ) => {
        $(
            #[doc(hidden)]
            $item
        )*
    };
}
