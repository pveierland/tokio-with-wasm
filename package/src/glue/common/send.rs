use std::fmt::{Debug, Display};

// SendWrapper adopted from:
// https://docs.rs/worker/0.6.6/src/worker/send.rs.html

pub struct SendWrapper<T>(pub T);

unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}

impl<T> SendWrapper<T> {
  pub fn new(inner: T) -> Self {
    Self(inner)
  }
}

impl<T> std::ops::Deref for SendWrapper<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> std::ops::DerefMut for SendWrapper<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T: Debug> Debug for SendWrapper<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "SendWrapper({:?})", self.0)
  }
}

impl<T: Clone> Clone for SendWrapper<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl<T: Default> Default for SendWrapper<T> {
  fn default() -> Self {
    Self(T::default())
  }
}

impl<T: Display> Display for SendWrapper<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "SendWrapper({})", self.0)
  }
}
