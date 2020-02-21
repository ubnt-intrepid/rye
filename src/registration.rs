//! Registration of test cases.

use crate::test::Test;
use std::{error, fmt};

/// The registration of one or more test cases.
pub trait Registration {
    /// Register a collection of test cases in the registry.
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError>;
}

impl<R: ?Sized> Registration for &R
where
    R: Registration,
{
    #[inline]
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError> {
        (**self).register(registry)
    }
}

impl<R: ?Sized> Registration for Box<R>
where
    R: Registration,
{
    #[inline]
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError> {
        (**self).register(registry)
    }
}

/// The registry of test cases.
pub trait Registry {
    /// Register a test case.
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError>;
}

impl<R: ?Sized> Registry for &mut R
where
    R: Registry,
{
    #[inline]
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
        (**self).add_test(test)
    }
}

impl<R: ?Sized> Registry for Box<R>
where
    R: Registry,
{
    #[inline]
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError> {
        (**self).add_test(test)
    }
}

#[derive(Debug)]
pub struct RegistryError(Box<dyn error::Error + Send + Sync>);

impl RegistryError {
    pub fn new(cause: impl Into<Box<dyn error::Error + Send + Sync>>) -> Self {
        Self(cause.into())
    }
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&*self.0, f)
    }
}

impl error::Error for RegistryError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&*self.0)
    }
}
