use dioxus::prelude::*;

use std::future::Future;

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
impl<T: 'static> Clone for AsyncSignal<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: 'static> Copy for AsyncSignal<T> {}

impl<T: 'static> AsyncSignal<T> {
    pub fn finished(&self) -> bool {
        self.use_future.finished()
    }

    async fn load_barrier(&self) {
        while !self.finished() {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }

    pub async fn with<CB, R>(&self, cb: CB) -> R
    where
        CB: AsyncFnOnce(&T) -> R,
        R: 'static,
    {
        self.load_barrier().await;
        let read_ref = self.value.read();
        let t = read_ref.as_ref().expect("load_barrier ensure it is some");
        cb(t).await
    }

    pub async fn with_peek<CB, R>(&self, cb: CB) -> R
    where
        CB: AsyncFnOnce(&T) -> R,
        R: 'static,
    {
        self.load_barrier().await;
        let read_ref = self.value.peek();
        let t = read_ref.as_ref().expect("load_barrier ensure it is some");
        cb(t).await
    }

    pub async fn with_mut<CB, R>(&mut self, cb: CB) -> R
    where
        CB: AsyncFnOnce(&mut T) -> R,
        R: 'static,
    {
        self.load_barrier().await;
        let mut write_ref = self.value.write();
        let mut_t = write_ref.as_mut().expect("load_barrier ensure it is some");
        cb(mut_t).await
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

impl<T> core::ops::Deref for AsyncSignal<T> {
    type Target = Signal<Option<T>>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T> core::ops::DerefMut for AsyncSignal<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
