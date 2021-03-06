use futures::future::{BoxFuture, Future, LocalBoxFuture};
use rye_runtime as rt;
use std::rc::Rc;
use tokio::{
    runtime::{Handle, Runtime},
    task::LocalSet,
};

pub fn runtime() -> impl rt::Runtime {
    // TODO: configure
    let rt = Runtime::new().expect("failed to start Tokio runtime");
    let locals = Rc::new(LocalSet::new());
    TokioRuntime { rt, locals }
}

struct TokioRuntime {
    rt: Runtime,
    locals: Rc<LocalSet>,
}

impl rt::Runtime for TokioRuntime {
    type Spawner = TokioSpawner;

    fn spawner(&self) -> Self::Spawner {
        TokioSpawner {
            handle: self.rt.handle().clone(),
            locals: self.locals.clone(),
        }
    }

    fn block_on<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        self.locals.block_on(&mut self.rt, fut)
    }
}

struct TokioSpawner {
    handle: Handle,
    locals: Rc<LocalSet>,
}

impl rt::Spawner for TokioSpawner {
    fn spawn(&mut self, fut: BoxFuture<'static, ()>) -> anyhow::Result<()> {
        self.handle.spawn(fut);
        Ok(())
    }

    fn spawn_local(&mut self, fut: LocalBoxFuture<'static, ()>) -> anyhow::Result<()> {
        self.locals.spawn_local(fut);
        Ok(())
    }

    fn spawn_blocking(&mut self, f: Box<dyn FnOnce() + Send + 'static>) -> anyhow::Result<()> {
        self.handle
            .spawn(async move { tokio::task::block_in_place(f) });
        Ok(())
    }
}
