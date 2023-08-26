use std::{
    sync::{
        Arc,
        Condvar,
    },
};

#[derive(Debug)]
pub struct ParkThread {
    pub inner: Arc<Inner>,
}

pub type ParkError = ();

#[derive(Debug)]
pub struct Inner {
    // pub state: AtomicUsize,
    // pub mutex: Mutex<()>,
    pub condvar: Condvar,
}

impl ParkThread {
    pub fn shutdown(&mut self) {
        self.inner.shutdown();
    }
}

impl Inner {
    fn shutdown(&self) {
        self.condvar.notify_all();
    }
}
