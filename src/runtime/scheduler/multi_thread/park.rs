use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Condvar, Mutex};
use crate::util::TryLock;
use crate::park::{Park, Unpark};
use crate::runtime::driver::Driver;

pub struct Parker {
    inner: Arc<Inner>,
}

pub struct Unparker {
    inner: Arc<Inner>,
}

struct Inner {
    state: AtomicUsize,
    mutex: Mutex<()>,
    condvar: Condvar,
    shared: Arc<Shared>,
}

struct Shared {
    driver: TryLock<Driver>,
    handle: <Driver as Park>::Unpark,
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
