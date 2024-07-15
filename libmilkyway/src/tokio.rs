/** Tokio utilities **/
use std::future::Future;
use std::sync::Mutex;
use std::time::Duration;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

thread_local! {
    pub static RUNTIME: Lazy<Mutex<Option<Runtime>>> = Lazy::new(Default::default);
}

/// Creates tokio runtime
#[inline]
fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

/// Creates a thread-local tokio runtime
#[inline]
pub fn init_tokio() {
    RUNTIME.with(|rt| { *rt.lock().unwrap() = Some(create_runtime()) });
}

/// Block execution on specific future
#[inline]
pub fn tokio_block_on<F: Future>(f: F) -> F::Output {
    RUNTIME.with(|rt| { rt.try_lock().unwrap().as_mut().unwrap().block_on(f) })
}

/// Spawn a future without blocking on it
#[inline]
pub fn tokio_spawn<F: Future + std::marker::Send + 'static>(f: F)
    where
        <F as futures::Future>::Output: std::marker::Send,
{
    RUNTIME.with(|rt| {
        rt.try_lock().unwrap().as_mut().unwrap().spawn(f)
    });
}

pub async fn tokio_timeout<'a, F: Future + Send + 'a>(milliseconds: Option<u64>, f: F) -> Option<<F as futures::Future>::Output>
    where <F as futures::Future>::Output: std::marker::Send,
{
    if milliseconds.is_none(){
        Some(f.await)
    } else {
        let duration = Duration::from_millis(milliseconds.unwrap());
        let result = tokio::time::timeout(duration, f).await;
        if result.is_err(){
            return None;
        }
        Some(result.unwrap())
    }
}