// //! Abstracts out the entire chain of runtime sub-drivers into common types.
use crate::park::thread::ParkThread;
use crate::park::Park;

use std::time::Duration;

type IoStack = ParkThread;


type TimeDriver = IoStack;

// // ===== runtime driver =====

#[derive(Debug)]
pub struct Driver {
    inner: TimeDriver,
}

impl Park for Driver {
    type Unpark = <TimeDriver as Park>::Unpark;
    type Error = <TimeDriver as Park>::Error;

    fn unpark(&self) -> Self::Unpark {
        self.inner.unpark()
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        self.inner.park()
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        self.inner.park_timeout(duration)
    }

    fn shutdown(&mut self) {
        self.inner.shutdown()
    }
}
