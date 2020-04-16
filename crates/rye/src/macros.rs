macro_rules! hidden_item {
    ( $( $item:item )* ) => {
        $(
            #[doc(hidden)]
            $item
        )*
    };
}

/// Mark the current test case as skipped and then terminate its execution.
///
/// This macro can usually be used to disable some test cases that may not
/// success depending on the runtime context, such as network access or a
/// certain secret variables is not set.
#[macro_export]
macro_rules! skip {
    ( $ctx:ident ) => {
        $crate::skip!($ctx, "explicitly skipped");
    };
    ( $ctx:ident, $($arg:tt)+ ) => {{
        use $crate::_test_reexports as __rye;
        const LOCATION: __rye::Location = __rye::location!();
        return $ctx.skip(&LOCATION, __rye::format_args!($($arg)+));
    }};
}

/// Mark the current test case as failed and then terminate its execution.
#[macro_export]
macro_rules! fail {
    ($ctx:ident) => {
        $crate::fail!($ctx, "explicitly failed");
    };
    ($ctx:ident, $($arg:tt)+) => {{
        use $crate::_test_reexports as __rye;
        const LOCATION: __rye::Location = __rye::location!();
        return $ctx.fail(&LOCATION, __rye::format_args!($($arg)+));
    }};
}

#[doc(hidden)] // private API
#[macro_export]
macro_rules! __test_name {
    ($name:ident) => {{
        use $crate::_test_reexports as __rye;
        __rye::TestName {
            raw: __rye::concat!(__rye::module_path!(), "::", __rye::stringify!($name)),
        }
    }};
}

#[doc(hidden)] // private API
#[macro_export]
macro_rules! __test_fn {
    (@async $path:path) => {{
        use $crate::_test_reexports as __rye;
        __rye::TestFn::Async(|mut ctx_ptr| {
            __rye::Box::pin(async move {
                __rye::Termination::into_result($path(ctx_ptr.as_mut()).await)
            })
        })
    }};

    (@async_local $path:path) => {{
        use $crate::_test_reexports as __rye;
        __rye::TestFn::AsyncLocal(|mut ctx_ptr| {
            __rye::Box::pin(async move {
                __rye::Termination::into_result($path(ctx_ptr.as_mut()).await)
            })
        })
    }};

    (@blocking $path:path) => {{
        use $crate::_test_reexports as __rye;
        __rye::TestFn::Blocking(|mut ctx_ptr| {
            __rye::Termination::into_result($path(ctx_ptr.as_mut()))
        })
    }};
}

#[doc(hidden)] // private API
#[macro_export]
macro_rules! __location {
    () => {{
        use $crate::_test_reexports as __rye;
        __rye::Location {
            file: __rye::file!(),
            line: __rye::line!(),
            column: __rye::column!(),
        }
    }};
}

#[doc(hidden)] // private API
#[macro_export]
macro_rules! __section {
    ( $ctx:ident, $id:expr, $name:expr, $(#[$attr:meta])* $block:block ) => {
        $(#[$attr])*
        {
            use $crate::_test_reexports as __rye;

            const SECTION: __rye::Section = __rye::Section {
                id: $id,
                name: $name,
                location: __rye::location!(),
            };
            let section = $ctx.enter_section(&SECTION);
            if section.enabled() {
                $block
            }
            $ctx.leave_section(section);
        }
    };
}
