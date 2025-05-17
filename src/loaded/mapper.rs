use super::{component::LoadedComponentInput, element::LoadedElement};

pub trait LoadedComponentInputMapper {
    fn map<T: LoadedElement>(&self, t: T) -> LoadedComponentInput<T>;
}

pub struct Loading;
impl LoadedComponentInputMapper for Loading {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, _t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::Loading
    }
}

pub struct LoadedError(pub String);
impl LoadedComponentInputMapper for LoadedError {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, _t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::LoadedError(self.0.clone())
    }
}

pub struct LoadedSuccess;
impl LoadedComponentInputMapper for LoadedSuccess {
    #[inline(always)]
    fn map<T: LoadedElement>(&self, t: T) -> LoadedComponentInput<T> {
        LoadedComponentInput::LoadedSuccess(t)
    }
}
