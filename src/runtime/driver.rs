// //! Abstracts out the entire chain of runtime sub-drivers into common types.
use crate::park::thread::ParkThread;

type IoStack = ParkThread;


type TimeDriver = IoStack;

// // ===== runtime driver =====

#[derive(Debug)]
pub struct Driver {
    pub inner: TimeDriver,
}

impl Driver {
    pub fn shutdown(&mut self) {
        self.inner.shutdown()
    }
}
