/// Wrapper to align a type to a 16-byte boundary
#[repr(align(16))]
#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Hash,Default)]
pub struct Align16<T>(pub T);

impl<T> From<T> for Align16<T> {
    fn from(v: T) -> Self {
        Align16(v)
    }
}

impl<T> core::ops::Deref for Align16<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> core::ops::DerefMut for Align16<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
