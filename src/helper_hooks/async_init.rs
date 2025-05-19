use dioxus::prelude::*;

use std::{
    future::Future,
    ops::{Deref, DerefMut},
};

/// This is similar to a [Resource] but with some major differences:
/// 1. The future does not re-run reactively (or at all), it is only intended to initialize the inner Signal value
/// 2. The inner value is accessible mutably
/// 3. Implements deref and deref_mut on the inner Signal<Option<T>>
///
/// The use-case here is to retrieve asynchronously btx-heritage-wallet types from the Database and then be able to call methods on them
pub fn use_async_init<T, F>(mut future: impl FnMut() -> F + 'static) -> AsyncSignal<T>
where
    T: 'static,
    F: Future<Output = T> + 'static,
{
    let mut value = use_signal(|| None);

    let use_future = use_future(move || {
        let fut = future();
        async move {
            let t = fut.await;
            value.set(Some(t));
        }
    });

    AsyncSignal { value, use_future }
}

pub struct AsyncSignal<T: 'static> {
    value: Signal<Option<T>>,
    use_future: UseFuture,
}
impl<T> PartialEq for AsyncSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.use_future == other.use_future
    }
}
impl<T> Clone for AsyncSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

trait AsyncSignalCallback<'a, T: 'a, R: 'static>: FnOnce(&'a mut T) -> Self::Fut {
    type Fut: Future<Output = R>;
}
impl<'a, T: 'a, R: 'static, Out: Future<Output = R>, F: FnOnce(&'a mut T) -> Out>
    AsyncSignalCallback<'a, T, R> for F
{
    type Fut = Out;
}

impl<T> Copy for AsyncSignal<T> {}
impl<T: 'static> AsyncSignal<T> {
    pub fn finished(&self) -> bool {
        self.use_future.finished()
    }

    pub async fn with_mut<'a, R: 'static, F>(&'a mut self, f: F) -> R
    where
        F: for<'b> AsyncSignalCallback<'b, T, R>,
    {
        while !self.use_future.finished() {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        let mut write_ref = self.value.write();

        let mut_t = write_ref
            .as_mut()
            .expect("wait on use_future ensure it is some");
        f(mut_t).await
    }
}

impl<T> From<AsyncSignal<T>> for ReadOnlySignal<Option<T>> {
    fn from(val: AsyncSignal<T>) -> Self {
        val.value.into()
    }
}

impl<T> Readable for AsyncSignal<T> {
    type Target = Option<T>;
    type Storage = UnsyncStorage;

    fn try_read_unchecked(&self) -> Result<ReadableRef<'static, Self>, BorrowError> {
        self.value.try_read_unchecked()
    }

    fn try_peek_unchecked(&self) -> Result<ReadableRef<'static, Self>, BorrowError> {
        self.value.try_peek_unchecked()
    }
}

impl<T> Deref for AsyncSignal<T> {
    type Target = Signal<Option<T>>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T> DerefMut for AsyncSignal<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
