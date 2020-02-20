use crate::test::Test;

pub trait Registration {
    fn register(&self, registry: &mut dyn Registry) -> Result<(), RegistryError>;
}

pub trait Registry {
    fn add_test(&mut self, test: Test) -> Result<(), RegistryError>;
}

#[derive(Debug)]
pub struct RegistryError(());

impl RegistryError {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(())
    }
}
