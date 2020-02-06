use crate::section::Sections;
use futures_core::future::Future;

/// Run a test case asynchronously.
pub async fn test_case_async<'a, F, Fut>(f: F)
where
    F: Fn() -> Fut + 'a,
    Fut: Future<Output = ()> + 'a,
{
    crate::tls::futures::with_tls(async move {
        let sections = Sections::new();
        while !sections.completed() {
            let section = sections.root();
            let _guard = crate::tls::set(section);
            f().await;
        }
    })
    .await
}
