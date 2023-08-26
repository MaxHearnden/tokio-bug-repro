use std::sync::{Arc, Condvar};
use crate::util::TryLock;
use crate::runtime::driver::Driver;

pub struct Parker {
    pub inner: Arc<Inner>,
}

pub struct Inner {
    pub condvar: Condvar,
    pub shared: Arc<Shared>,
}

pub struct Shared {
    pub driver: TryLock<Driver>,
}

impl Parker {
    #[no_mangle]
    pub fn shutdown(&mut self) {
        self.inner.shutdown();
    }
}

impl Inner {
    fn shutdown(&self) {
        if let Some(mut driver) = self.shared.driver.try_lock() {
            driver.shutdown();
        }

        self.condvar.notify_all();
    }
}
