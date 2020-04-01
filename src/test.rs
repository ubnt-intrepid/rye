#![allow(missing_docs)]

use crate::{
    report::{Outcome, Reporter, TestCaseSummary},
    runtime::Spawner,
    termination::Termination,
};
use futures_channel::oneshot;
use futures_core::{
    future::{BoxFuture, Future, LocalBoxFuture},
    task::{self, Poll},
};
use maybe_unwind::{maybe_unwind, FutureMaybeUnwindExt as _, Unwind};
use pin_project::pin_project;
use std::{fmt, marker::PhantomData, panic::AssertUnwindSafe, pin::Pin, ptr::NonNull};

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

/// Metadata about a test case.
#[derive(Debug)]
pub struct TestDesc {
    pub name: TestName,
    pub location: Location,
}

impl TestDesc {
    /// Return the name of test case.
    ///
    /// Test cases are uniquely named by their relative path from
    /// the root module.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

#[derive(Debug)]
pub struct TestPlan {
    pub target: Option<SectionId>,
    pub ancestors: &'static [SectionId],
}

impl TestPlan {
    pub(crate) fn is_enabled(&self, id: SectionId) -> bool {
        self.target.map_or(false, |target| target == id) || self.ancestors.contains(&id)
    }
}

pub(crate) type SectionId = u64;

#[derive(Debug)]
pub enum TestFn {
    Async(fn(ContextPtr) -> BoxFuture<'static, anyhow::Result<()>>),
    AsyncLocal(fn(ContextPtr) -> LocalBoxFuture<'static, anyhow::Result<()>>),
    Blocking(fn(ContextPtr) -> anyhow::Result<()>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TestName {
    pub raw: &'static str,
}

impl AsRef<str> for TestName {
    fn as_ref(&self) -> &str {
        self.raw.splitn(2, "::").nth(1).unwrap()
    }
}

pub trait TestCase: Send + Sync {
    fn desc(&self) -> &'static TestDesc;
    fn test_fn(&self) -> TestFn;
    fn test_plans(&self) -> &'static [TestPlan];
}

impl<T: ?Sized> TestCase for &T
where
    T: TestCase,
{
    fn desc(&self) -> &'static TestDesc {
        (**self).desc()
    }

    fn test_fn(&self) -> TestFn {
        (**self).test_fn()
    }

    fn test_plans(&self) -> &'static [TestPlan] {
        (**self).test_plans()
    }
}

impl dyn TestCase + '_ {
    #[cfg(test)]
    pub(crate) async fn run<R>(&self, reporter: &mut R) -> TestCaseSummary
    where
        R: Reporter + Send + 'static,
    {
        let mut inner = TestInner {
            desc: self.desc(),
            plans: self.test_plans(),
        };
        match self.test_fn() {
            TestFn::Async(f) => inner.run_async(reporter, f).await,
            TestFn::AsyncLocal(f) => inner.run_async(reporter, f).await,
            TestFn::Blocking(f) => inner.run_blocking(reporter, f),
        }
    }

    pub(crate) fn spawn<R>(&self, spawner: &mut dyn Spawner, reporter: R) -> anyhow::Result<Handle>
    where
        R: Reporter + Send + 'static,
    {
        let mut inner = TestInner {
            desc: self.desc(),
            plans: self.test_plans(),
        };
        let mut reporter = reporter;

        let (tx, rx) = oneshot::channel();
        match self.test_fn() {
            TestFn::Async(f) => {
                spawner.spawn(Box::pin(async move {
                    let summary = inner.run_async(&mut reporter, f).await;
                    let _ = tx.send(summary);
                }))?;
            }
            TestFn::AsyncLocal(f) => {
                spawner.spawn_local(Box::pin(async move {
                    let summary = inner.run_async(&mut reporter, f).await;
                    let _ = tx.send(summary);
                }))?;
            }
            TestFn::Blocking(f) => {
                spawner.spawn_blocking(Box::new(move || {
                    let summary = inner.run_blocking(&mut reporter, f);
                    let _ = tx.send(summary);
                }))?;
            }
        }

        Ok(Handle {
            rx,
            desc: self.desc(),
        })
    }
}

#[pin_project]
pub(crate) struct Handle {
    #[pin]
    rx: oneshot::Receiver<TestCaseSummary>,
    desc: &'static TestDesc,
}

impl Future for Handle {
    type Output = TestCaseSummary;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        match futures_util::ready!(me.rx.poll(cx)) {
            Ok(summary) => Poll::Ready(summary),
            Err(..) => todo!("report cancellation"),
        }
    }
}

struct TestInner {
    desc: &'static TestDesc,
    plans: &'static [TestPlan],
}

impl TestInner {
    async fn run_async<Fut>(
        &mut self,
        reporter: &mut (dyn Reporter + Send + 'static),
        f: fn(ContextPtr) -> Fut,
    ) -> TestCaseSummary
    where
        Fut: Future<Output = anyhow::Result<()>>,
    {
        reporter.test_case_starting(&self.desc);

        let mut outcome = Outcome::Passed;
        for plan in self.plans {
            let mut ctx = Context::new(reporter, plan);
            let result = AssertUnwindSafe(f(unsafe { ctx.transmute() }))
                .maybe_unwind()
                .await;
            if let Some(o) = ctx.check_outcome(result) {
                outcome = o;
                break;
            }
        }

        let summary = TestCaseSummary {
            desc: self.desc,
            outcome,
        };
        reporter.test_case_ended(&summary);

        summary
    }

    fn run_blocking(
        &mut self,
        reporter: &mut (dyn Reporter + Send),
        f: fn(ContextPtr) -> anyhow::Result<()>,
    ) -> TestCaseSummary {
        reporter.test_case_starting(&self.desc);

        let mut outcome = Outcome::Passed;
        for plan in self.plans {
            let mut ctx = Context::new(reporter, plan);
            let result = maybe_unwind(AssertUnwindSafe(|| f(unsafe { ctx.transmute() })));
            if let Some(o) = ctx.check_outcome(result) {
                outcome = o;
                break;
            }
        }

        let summary = TestCaseSummary {
            desc: self.desc,
            outcome,
        };
        reporter.test_case_ended(&summary);

        summary
    }
}

#[repr(transparent)]
pub struct ContextPtr(NonNull<Context<'static>>);

unsafe impl Send for ContextPtr {}

impl ContextPtr {
    #[allow(clippy::should_implement_trait)]
    #[inline]
    pub fn as_mut(&mut self) -> &mut Context<'static> {
        unsafe { self.0.as_mut() }
    }
}

/// Context values while running the test case.
pub struct Context<'a> {
    plan: &'a TestPlan,
    #[allow(dead_code)]
    reporter: &'a mut (dyn Reporter + Send),
    current_section: Option<&'static Section>,
    outcome: Option<Outcome>,
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(reporter: &'a mut (dyn Reporter + Send), plan: &'a TestPlan) -> Self {
        Self {
            plan,
            reporter,
            current_section: None,
            outcome: None,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub(crate) unsafe fn transmute(&mut self) -> ContextPtr {
        ContextPtr(NonNull::from(&mut *self).cast::<Context<'static>>())
    }

    #[cfg(test)]
    pub(crate) fn current_section_name(&self) -> Option<&'static str> {
        self.current_section.map(|section| section.name)
    }

    pub(crate) fn check_outcome(
        &mut self,
        result: Result<anyhow::Result<()>, Unwind>,
    ) -> Option<Outcome> {
        match result {
            Ok(Ok(())) => self.outcome.take(),
            Ok(Err(err)) => Some(Outcome::Errored(err)),
            Err(unwind) => Some(Outcome::Panicked(unwind)),
        }
    }

    #[inline]
    fn exit<T>(&mut self) -> T
    where
        T: Termination,
    {
        T::exit()
    }
}

hidden_item! {
    impl Context<'_> {
        pub fn enter_section(&mut self, section: &'static Section) -> EnterSection {
            let enabled = self.plan.is_enabled(section.id);
            let last_section = self.current_section.replace(section);
            EnterSection {
                enabled,
                last_section,
            }
        }

        pub fn leave_section(&mut self, enter: EnterSection) {
            self.current_section = enter.last_section;
        }


        pub fn skip<T>(&mut self, location: &'static Location, reason: fmt::Arguments<'_>) -> T
        where
            T: Termination,
        {
            debug_assert!(self.outcome.is_none());
            self.outcome.replace(Outcome::Skipped {
                location,
                reason: reason.to_string(),
            });
            self.exit()
        }

        pub fn fail<T>(&mut self, location: &'static Location, reason: fmt::Arguments<'_>) -> T
        where
            T: Termination,
        {
            debug_assert!(self.outcome.is_none());
            self.outcome.replace(Outcome::Failed {
                location,
                reason: reason.to_string(),
            });
            self.exit()
        }
    }
}

pub struct EnterSection {
    enabled: bool,
    last_section: Option<&'static Section>,
}

impl EnterSection {
    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

pub struct Section {
    pub id: SectionId,
    pub name: &'static str,
    pub location: Location,
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
        $crate::fail!($ctx:ident, "explicitly failed");
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
macro_rules! __test_case {
    ( $item:item ) => {
        $crate::__cfg_harness! {
            #[$crate::_test_reexports::distributed_slice($crate::_test_reexports::TEST_CASES)]
            #[linkme(crate = $crate::_test_reexports::linkme)]
            $item
        }
        $crate::__cfg_frameworks! {
            #[test_case]
            $item
        }
    };
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

#[cfg(all(test, not(feature = "frameworks")))]
mod tests {
    use super::*;
    use crate::{
        report::Summary,
        test::{TestCase, TestDesc},
    };
    use futures::executor::block_on;
    use scoped_tls_async::{scoped_thread_local, ScopedKeyExt as _};
    use std::cell::RefCell;

    type HistoryLog = (&'static str, Option<&'static str>);

    scoped_thread_local!(static HISTORY: RefCell<Vec<HistoryLog>>);

    fn append_history(ctx: &mut Context<'_>, msg: &'static str) {
        let current_section = ctx.current_section_name();
        HISTORY.with(|history| history.borrow_mut().push((msg, current_section)));
    }

    struct NullReporter;

    impl Reporter for NullReporter {
        fn test_run_starting(&self, _: &[&dyn TestCase]) {}
        fn test_run_ended(&self, _: &Summary) {}
        fn test_case_starting(&self, _: &TestDesc) {}
        fn test_case_ended(&self, _: &TestCaseSummary) {}
    }

    fn run(t: &dyn TestCase) -> Vec<HistoryLog> {
        let history = RefCell::new(vec![]);
        let _summary = block_on(HISTORY.set_async(&history, t.run(&mut NullReporter)));
        history.into_inner()
    }

    #[test]
    fn no_section() {
        #[crate::test]
        #[rye(crate = crate)]
        fn test_case(ctx: &mut Context<'_>) {
            append_history(ctx, "test");
        }

        let history = run(test_case);
        assert_eq!(history, vec![("test", None)]);
    }

    #[test]
    fn one_section() {
        #[crate::test]
        #[rye(crate = crate)]
        fn test_case(ctx: &mut Context<'_>) {
            append_history(ctx, "setup");

            section!(ctx, "section1", {
                append_history(ctx, "section1");
            });

            append_history(ctx, "teardown");
        }

        let history = run(test_case);
        assert_eq!(
            history,
            vec![
                ("setup", None),
                ("section1", Some("section1")),
                ("teardown", None)
            ]
        );
    }

    #[test]
    fn multi_section() {
        #[crate::test]
        #[rye(crate = crate)]
        fn test_case(ctx: &mut Context<'_>) {
            append_history(ctx, "setup");

            section!(ctx, "section1", {
                append_history(ctx, "section1");
            });

            section!(ctx, "section2", {
                append_history(ctx, "section2");
            });

            append_history(ctx, "teardown");
        }

        let history = run(test_case);
        assert_eq!(
            history,
            vec![
                // phase 1
                ("setup", None),
                ("section1", Some("section1")),
                ("teardown", None),
                // phase 2
                ("setup", None),
                ("section2", Some("section2")),
                ("teardown", None),
            ]
        );
    }

    #[test]
    fn nested_section() {
        #[crate::test]
        #[rye(crate = crate)]
        fn test_case(ctx: &mut Context<'_>) {
            append_history(ctx, "setup");

            section!(ctx, "section1", {
                append_history(ctx, "section1:setup");

                section!(ctx, "section2", {
                    append_history(ctx, "section2");
                });

                section!(ctx, "section3", {
                    append_history(ctx, "section3");
                });

                append_history(ctx, "section1:teardown");
            });

            section!(ctx, "section4", {
                append_history(ctx, "section4");
            });

            append_history(ctx, "teardown");
        }

        let history = run(test_case);
        assert_eq!(
            history,
            vec![
                // phase 1
                ("setup", None),
                ("section1:setup", Some("section1")),
                ("section2", Some("section2")),
                ("section1:teardown", Some("section1")),
                ("teardown", None),
                // phase 2
                ("setup", None),
                ("section1:setup", Some("section1")),
                ("section3", Some("section3")),
                ("section1:teardown", Some("section1")),
                ("teardown", None),
                // phase 3
                ("setup", None),
                ("section4", Some("section4")),
                ("teardown", None),
            ]
        );
    }

    #[test]
    fn smoke_async() {
        #[crate::test]
        #[rye(crate = crate)]
        async fn test_case(ctx: &mut Context<'_>) {
            use futures_test::future::FutureTestExt as _;

            append_history(ctx, "setup");
            async {}.pending_once().await;

            section!(ctx, "section1", {
                append_history(ctx, "section1:setup");
                async {}.pending_once().await;

                section!(ctx, "section2", {
                    async {}.pending_once().await;
                    append_history(ctx, "section2");
                });

                section!(ctx, "section3", {
                    async {}.pending_once().await;
                    append_history(ctx, "section3");
                });

                async {}.pending_once().await;
                append_history(ctx, "section1:teardown");
            });

            section!(ctx, "section4", {
                async {}.pending_once().await;
                append_history(ctx, "section4");
            });

            async {}.pending_once().await;
            append_history(ctx, "teardown");
        }

        let history = run(test_case);
        assert_eq!(
            history,
            vec![
                // phase 1
                ("setup", None),
                ("section1:setup", Some("section1")),
                ("section2", Some("section2")),
                ("section1:teardown", Some("section1")),
                ("teardown", None),
                // phase 2
                ("setup", None),
                ("section1:setup", Some("section1")),
                ("section3", Some("section3")),
                ("section1:teardown", Some("section1")),
                ("teardown", None),
                // phase 3
                ("setup", None),
                ("section4", Some("section4")),
                ("teardown", None),
            ]
        );
    }
}
