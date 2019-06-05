use crate::{error::CapacityError, ArrayVec};

use core::{
    slice,
};

pub struct ArrayString<const N: usize> {
    array: ArrayVec<u8, {N}>
}

impl <const N: usize> Default for ArrayString<{N}> {
    fn default() -> Self {
        Self {
            array: Default::default(),
        }
    }
}

impl <const N: usize> ArrayString<{N}> {
    pub const fn is_full(&self) -> bool {
        self.array.is_full()
    }

    pub const fn len(&self) -> usize {
        self.array.len()
    }

    pub const fn remaining_capacity(&self) -> usize {
        self.array.remaining_capacity()
    }

    pub const fn capacity(&self) -> usize {
        self.array.capacity()
    }

    pub fn push(&mut self, item: char) {
        self.try_push(item).unwrap();
    }

    pub fn try_push(&mut self, item: char) -> Result<(), CapacityError<char>> {
        unimplemented!()
    }
    
    pub unsafe fn as_bytes_mut(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.array.as_mut_ptr(), self.capacity())
    }
}