use std::sync::Condvar;
use std::sync::Arc;
pub mod util;
pub mod runtime;
use runtime::scheduler::multi_thread::park::{
    Parker,
    Inner,
    Shared,
};

fn main() {
    let arc_inner = Arc::new(Condvar::new());
    let shared = Arc::new(
        Shared {
            driver: util::TryLock::new(arc_inner)
        }
    );
    Parker {
        inner: Arc::new (Inner {
            shared: shared.clone(),
        })
    }.shutdown();
}