/// Arbitrary return values in test cases.
pub trait Termination: sealed::Sealed {
    #[doc(hidden)] // TODO: dox
    fn exit() -> Self;

    #[doc(hidden)] // TODO: dox
    fn into_result(self) -> anyhow::Result<()>;
}

impl Termination for () {
    #[doc(hidden)] // TODO: dox
    fn exit() -> Self {}

    #[doc(hidden)] // TODO: dox
    #[inline]
    fn into_result(self) -> anyhow::Result<()> {
        Ok(())
    }
}

impl<E> Termination for Result<(), E>
where
    E: Into<anyhow::Error>,
{
    #[doc(hidden)] // TODO: dox
    fn exit() -> Self {
        Ok(())
    }

    #[doc(hidden)] // TODO: dox
    #[inline]
    fn into_result(self) -> anyhow::Result<()> {
        self.map_err(Into::into)
    }
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for () {}

    impl<E> Sealed for Result<(), E> where E: Into<anyhow::Error> {}
}

#[allow(missing_docs)]
pub fn exit(t: impl Termination) -> ! {
    let code = match Termination::into_result(t) {
        Ok(()) => 0,
        Err(_) => 101,
    };
    std::process::exit(code);
}
