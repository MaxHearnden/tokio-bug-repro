use std::sync::Arc;
use std::fmt::Debug;
use std::time::Duration;
pub mod thread;

pub trait Park {
  /// Unpark handle type for the `Park` implementation.
  type Unpark: Unpark;

  /// Error returned by `park`.
  type Error: Debug;

  /// Gets a new `Unpark` handle associated with this `Park` instance.
  fn unpark(&self) -> Self::Unpark;

  /// Blocks the current thread unless or until the token is available.
  ///
  /// A call to `park` does not guarantee that the thread will remain blocked
  /// forever, and callers should be prepared for this possibility. This
  /// function may wakeup spuriously for any reason.
  ///
  /// # Panics
  ///
  /// This function **should** not panic, but ultimately, panics are left as
  /// an implementation detail. Refer to the documentation for the specific
  /// `Park` implementation.
  fn park(&mut self) -> Result<(), Self::Error>;

  /// Parks the current thread for at most `duration`.
  ///
  /// This function is the same as `park` but allows specifying a maximum time
  /// to block the thread for.
  ///
  /// Same as `park`, there is no guarantee that the thread will remain
  /// blocked for any amount of time. Spurious wakeups are permitted for any
  /// reason.
  ///
  /// # Panics
  ///
  /// This function **should** not panic, but ultimately, panics are left as
  /// an implementation detail. Refer to the documentation for the specific
  /// `Park` implementation.
  fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error>;

  /// Releases all resources held by the parker for proper leak-free shutdown.
  fn shutdown(&mut self);
}

pub trait Unpark: Sync + Send + 'static {
  /// Unblocks a thread that is blocked by the associated `Park` handle.
  ///
  /// Calling `unpark` atomically makes available the unpark token, if it is
  /// not already available.
  ///
  /// # Panics
  ///
  /// This function **should** not panic, but ultimately, panics are left as
  /// an implementation detail. Refer to the documentation for the specific
  /// `Unpark` implementation.
  fn unpark(&self);
}

impl Unpark for Box<dyn Unpark> {
    fn unpark(&self) {
        (**self).unpark()
    }
}

impl Unpark for Arc<dyn Unpark> {
    fn unpark(&self) {
        (**self).unpark()
    }
}
