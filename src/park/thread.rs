use crate::park::{Park, Unpark};

use std::{
    sync::{
        Arc,
        atomic::{
            AtomicUsize,
            Ordering::SeqCst,
        },
        Condvar,
        Mutex,
    },
    time::Duration,
};

#[derive(Debug)]
pub struct ParkThread {
    inner: Arc<Inner>,
}

pub type ParkError = ();

#[derive(Clone, Debug)]
pub struct UnparkThread {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    state: AtomicUsize,
    mutex: Mutex<()>,
    condvar: Condvar,
}

const EMPTY: usize = 0;
const PARKED: usize = 1;
const NOTIFIED: usize = 2;

impl Park for ParkThread {
    type Unpark = UnparkThread;
    type Error = ParkError;

    fn unpark(&self) -> Self::Unpark {
        let inner = self.inner.clone();
        UnparkThread { inner }
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        self.inner.park();
        Ok(())
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        // Wasm doesn't have threads, so just sleep.
        self.inner.park_timeout(duration);
        Ok(())
    }

    fn shutdown(&mut self) {
        self.inner.shutdown();
    }
}

impl Unpark for UnparkThread {
    fn unpark(&self) {
        self.inner.unpark();
    }
}

impl Inner {
    /// Parks the current thread for at most `dur`.
    fn park(&self) {
        // If we were previously notified then we consume this notification and
        // return quickly.
        if self
            .state
            .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
            .is_ok()
        {
            return;
        }

        // Otherwise we need to coordinate going to sleep
        let mut m = self.mutex.lock().unwrap();

        match self.state.compare_exchange(EMPTY, PARKED, SeqCst, SeqCst) {
            Ok(_) => {}
            Err(NOTIFIED) => {
                // We must read here, even though we know it will be `NOTIFIED`.
                // This is because `unpark` may have been called again since we read
                // `NOTIFIED` in the `compare_exchange` above. We must perform an
                // acquire operation that synchronizes with that `unpark` to observe
                // any writes it made before the call to unpark. To do that we must
                // read from the write it made to `state`.
                let old = self.state.swap(EMPTY, SeqCst);
                debug_assert_eq!(old, NOTIFIED, "park state changed unexpectedly");

                return;
            }
            Err(actual) => panic!("inconsistent park state; actual = {}", actual),
        }

        loop {
            m = self.condvar.wait(m).unwrap();

            if self
                .state
                .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
                .is_ok()
            {
                // got a notification
                return;
            }

            // spurious wakeup, go back to sleep
        }
    }

    fn park_timeout(&self, dur: Duration) {
        // Like `park` above we have a fast path for an already-notified thread,
        // and afterwards we start coordinating for a sleep. Return quickly.
        if self
            .state
            .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
            .is_ok()
        {
            return;
        }

        if dur == Duration::from_millis(0) {
            return;
        }

        let m = self.mutex.lock().unwrap();

        match self.state.compare_exchange(EMPTY, PARKED, SeqCst, SeqCst) {
            Ok(_) => {}
            Err(NOTIFIED) => {
                // We must read again here, see `park`.
                let old = self.state.swap(EMPTY, SeqCst);
                debug_assert_eq!(old, NOTIFIED, "park state changed unexpectedly");

                return;
            }
            Err(actual) => panic!("inconsistent park_timeout state; actual = {}", actual),
        }

        // Wait with a timeout, and if we spuriously wake up or otherwise wake up
        // from a notification, we just want to unconditionally set the state back to
        // empty, either consuming a notification or un-flagging ourselves as
        // parked.
        let (_m, _result) = self.condvar.wait_timeout(m, dur).unwrap();

        match self.state.swap(EMPTY, SeqCst) {
            NOTIFIED => {} // got a notification, hurray!
            PARKED => {}   // no notification, alas
            n => panic!("inconsistent park_timeout state: {}", n),
        }
    }

    fn unpark(&self) {
        // To ensure the unparked thread will observe any writes we made before
        // this call, we must perform a release operation that `park` can
        // synchronize with. To do that we must write `NOTIFIED` even if `state`
        // is already `NOTIFIED`. That is why this must be a swap rather than a
        // compare-and-swap that returns if it reads `NOTIFIED` on failure.
        match self.state.swap(NOTIFIED, SeqCst) {
            EMPTY => return,    // no one was waiting
            NOTIFIED => return, // already unparked
            PARKED => {}        // gotta go wake someone up
            _ => panic!("inconsistent state in unpark"),
        }

        // There is a period between when the parked thread sets `state` to
        // `PARKED` (or last checked `state` in the case of a spurious wake
        // up) and when it actually waits on `cvar`. If we were to notify
        // during this period it would be ignored and then when the parked
        // thread went to sleep it would never wake up. Fortunately, it has
        // `lock` locked at this stage so we can acquire `lock` to wait until
        // it is ready to receive the notification.
        //
        // Releasing `lock` before the call to `notify_one` means that when the
        // parked thread wakes it doesn't get woken only to have to wait for us
        // to release `lock`.
        drop(self.mutex.lock());

        self.condvar.notify_one()
    }

    fn shutdown(&self) {
        self.condvar.notify_all();
    }
}