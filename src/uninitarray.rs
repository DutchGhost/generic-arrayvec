use crate::array::{Array, UninitArrayExt};

use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    slice,
};

pub(crate) struct UninitArray<A>
where
    A: Array
{
    array: A::UninitArray,
}

impl<A> UninitArray<A>
where
    A: Array
{
    pub fn new() -> Self {
        Self {
            array: unsafe { A::UninitArray::uninit() },
        }
    }
}

impl<A> Deref for UninitArray<A>
where
    A: Array
{
    type Target = [MaybeUninit<A::Item>];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.array.as_ptr(), A::capacity()) }
    }
}

impl<A> DerefMut for UninitArray<A>
where
    A: Array
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.array.as_mut_ptr(), A::capacity()) }
    }
}
