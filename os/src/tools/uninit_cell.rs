use core::ops::{Deref, DerefMut};

pub struct UninitCell<T>(Option<T>);

impl<T> UninitCell<T> {
    pub const fn uninit() -> Self {
        Self(None)
    }

    pub fn init(inner: T) -> Self {
        Self(Some(inner))
    }
}

impl<T> Deref for UninitCell<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("Not init yet")
    }
}

impl<T> DerefMut for UninitCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("Not init yet")
    }
}
