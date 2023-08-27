use std::sync::{Arc, Condvar};
use crate::util::TryLock;

pub struct Parker {
    pub inner: Arc<Inner>,
}

pub struct Inner {
    pub shared: Arc<Shared>,
}

pub struct Shared {
    pub driver: TryLock<Arc<Condvar>>,
}

impl Parker {
    #[no_mangle]
    #[inline(never)]
    pub fn shutdown(&mut self) {
        self.inner.shutdown();
    }
}

impl Inner {
    #[inline(never)]
    fn shutdown(&self) {
        if let Some(driver) = self.shared.driver.try_lock() {
            driver.notify_all();
        }

    }
}
