use core::{
    mem::{self, MaybeUninit},
    ptr,
    slice,
    ops::{Deref, DerefMut},
};

pub struct ArrayVec<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> Default for ArrayVec<T, { N }> {
    fn default() -> Self {
        Self {
            array: unsafe { MaybeUninit::<_>::uninit().assume_init() },
            len: 0,
        }
    }
}

impl<T, const N: usize> ArrayVec<T, { N }> {
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }

    pub fn push(&mut self, item: T) {
        self.try_push(item).unwrap()
    }

    pub fn try_push(&mut self, item: T) -> Result<(), ()> {
        if self.len() < self.capacity() {
            unsafe { self.push_unchecked(item) }
            Ok(())
        } else {
            Err(())
        }
    }

    pub unsafe fn push_unchecked(&mut self, item: T) {
        let len = self.len();
        debug_assert!(len < N);
        ptr::write(self.array.get_unchecked_mut(len).as_mut_ptr(), item);
        self.set_len(len + 1);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let new_len = self.len() - 1;
                self.set_len(new_len);
                let to_be_popped = self.array.get_unchecked_mut(new_len);
                let popped = ptr::replace(to_be_popped, MaybeUninit::uninit());
                Some(popped.assume_init())
            }
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        unsafe {
            if new_len < self.len() {
                let truncated: *mut [T] = self.get_unchecked_mut(new_len..);
                self.set_len(new_len);
                ptr::drop_in_place(truncated);
            }
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }
}

impl <T, const N: usize> Drop for ArrayVec<T, {N}> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl <T, const N: usize> Deref for ArrayVec<T, {N}> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe {
            let first_ptr: *const MaybeUninit<T> = self.array.as_ptr();
            let first_ptr: *const T = first_ptr as *const T;

            slice::from_raw_parts(first_ptr, self.len())
        }
    }
}

impl <T, const N: usize> DerefMut for ArrayVec<T, {N}> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let first_ptr: *mut MaybeUninit<T> = self.array.as_mut_ptr();
            let first_ptr: *mut T = first_ptr as *mut T;

            slice::from_raw_parts_mut(first_ptr, self.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_ensure_it_doesnt_ice() {
        let mut v = ArrayVec::<u8, { 399 }>::default();

        assert!(v.is_empty());
        assert!(!v.is_full());
        assert_eq!(v.capacity(), 399);

        v.push(20);
        v.push(30);

        assert_eq!(v.len(), 2);

        assert_eq!(v.pop(), Some(30));
        assert_eq!(v.len(), 1);

        v.clear();

        assert!(v.is_empty());
    }
}
