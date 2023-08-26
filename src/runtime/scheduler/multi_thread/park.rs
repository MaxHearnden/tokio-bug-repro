use std::sync::{Arc, Condvar};
use crate::util::TryLock;
use crate::park::Park;
use crate::runtime::driver::Driver;

pub struct Parker {
    inner: Arc<Inner>,
}

struct Inner {
    condvar: Condvar,
    shared: Arc<Shared>,
}

struct Shared {
    driver: TryLock<Driver>,
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
