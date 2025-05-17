mod from;
mod from_dioxus;
mod ref_from;

pub trait FromRef<R> {
    fn from_ref(value: &R) -> Self;
}
impl<T: Clone> FromRef<T> for T {
    #[inline(always)]
    fn from_ref(value: &T) -> Self {
        value.clone()
    }
}

pub trait RefInto<T> {
    fn ref_into(&self) -> T;
}

impl<R, T: FromRef<R>> RefInto<T> for R {
    #[inline(always)]
    fn ref_into(&self) -> T {
        T::from_ref(self)
    }
}
