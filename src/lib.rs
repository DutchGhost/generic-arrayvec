//#![no_std]
#![feature(const_generics)]

pub mod error;
use error::CapacityError;

//mod string;
//pub use string::ArrayString;
mod macros;

use core::{
    mem::{self, MaybeUninit},
    ptr,
    slice,
    ops::{Deref, DerefMut},
    iter::{FusedIterator, Extend, FromIterator},
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

impl <T, const N: usize> ArrayVec<T, {N}> {
    /// Takes the value at `index`,
    /// and returns it.
    /// Marked unsafe, because it indexes into the array without
    /// bounds checks, and it assumes the element at `index` has been initialized.
    unsafe fn take(&mut self, index: usize) -> T {
        debug_assert!(index < self.len());
        let element = self.array.get_unchecked_mut(index);
        ptr::read(element.as_ptr())
    }
}

impl<T, const N: usize> ArrayVec<T, { N }> {
    /// Returns whether the `ArrayVec` is empty.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    constify! {
        if #[cfg(not(miri))]
        pub fn is_full(self: &Self) -> bool {
            self.len() == self.capacity()
        }    
    }

    /// Returns the number of elements in the `ArrayVec`.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    constify! {
        if #[cfg(not(miri))]
        pub fn capacity(self: &Self) -> usize {
            #[cfg(not(miri))]
            {
                N
            }

            #[cfg(miri)]
            {
                self.array.len()
            }
        }
    }

    constify! {
        if #[cfg(not(miri))]
        pub fn remaining_capacity(self: &Self) -> usize {
            self.capacity() - self.len()
        }
    }

    /// Sets the length of the `ArrayVec` to `length`,
    /// without dropping or moving elements.
    ///
    /// # Unsafe
    /// This function is marked unsafe, because it changes
    /// the number of `valid` (e.g written-to) elements.
    #[inline(always)]
    pub unsafe fn set_len(&mut self, length: usize) {
        debug_assert!(length <= self.capacity());
        self.len = length;
    }

    /// Push `item` onto the `ArrayVec`.
    #[inline]
    pub fn push(&mut self, item: T) {
        self.try_push(item).unwrap()
    }
    
    /// Tries to push `item` onto the `ArrayVec`.
    /// Returns `Ok(())` if the ArrayVec had
    /// enough free space for another item.
    /// A [`CapacityError`] is returned when there wasn't enough
    /// free space for another item.
    #[inline]
    pub fn try_push(&mut self, item: T) -> Result<(), CapacityError<T>> {
        if self.len() < self.capacity() {
            unsafe { self.push_unchecked(item) }
            Ok(())
        } else {
            Err(CapacityError::new(item))
        }
    }

    #[inline]
    pub unsafe fn push_unchecked(&mut self, item: T) {
        let len = self.len();
        debug_assert!(len < self.capacity());
        ptr::write(self.array.get_unchecked_mut(len).as_mut_ptr(), item);
        self.set_len(len + 1);
    }

    #[inline]
    pub fn insert(&mut self, index: usize, item: T) {
        self.try_insert(index, item).unwrap()
    }

    #[inline]
    pub fn try_insert(&mut self, index: usize, item: T) -> Result<(), CapacityError<T>> {
        if index > self.len() {
            panic!()
        }

        if self.is_full() {
            return Err(CapacityError::new(item));
        }

        let len = self.len();
        unsafe {
            let place_to_insert: *mut MaybeUninit<T> = self.array.get_unchecked_mut(index);
            ptr::copy(place_to_insert, place_to_insert.offset(1), len - index);
            ptr::write(place_to_insert, MaybeUninit::new(item));
            self.set_len(len + 1);
        }
        Ok(())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let new_len = self.len() - 1;
                let popped = self.take(new_len);
                self.set_len(new_len);
                Some(popped)
            }
        }
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.swap_pop(index).unwrap()
    }

    #[inline]
    pub fn swap_pop(&mut self, index: usize) -> Option<T> {
        let len = self.len();
        if index >= len {
            None
        } else {
            self.swap(index, len - 1);
            self.pop()
        }
    }

    #[inline]
    pub fn truncate(&mut self, new_len: usize) {
        unsafe {
            if new_len < self.len() {
                let old_len = self.len();

                // panic safety,
                // dont double drop when destructors panic.
                self.set_len(new_len);
                let truncated: *mut [T] = self.get_unchecked_mut(new_len..old_len);
                // truncated is self[new_len..old_len] *before* we did set the len
                // to the new len, which we had to do for panic safety.
                ptr::drop_in_place(truncated);
            }
        }
    }

    pub fn try_extend_from_slice(&mut self, slice: &[T]) -> Result<(), CapacityError>
    where
        T: Copy
    {
        if self.remaining_capacity() < slice.len() {
            return Err(CapacityError::new(()));
        } else {
            let self_len = self.len();
            let slice_len = slice.len();

            unsafe {
                let dst = self.array.get_unchecked_mut(0).as_mut_ptr().offset(self_len as isize);
                ptr::copy_nonoverlapping(slice.as_ptr(), dst, slice_len);
                self.set_len(self_len + slice_len);
            }
            Ok(())
        }
    }


    #[inline(always)]
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    #[inline(always)]
    pub fn iter(&self) -> slice::Iter<T> {
        let slice: &[T] = self;
        slice.iter()
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> slice::IterMut<T> {
        let slice: &mut [T] = self;
        slice.iter_mut()
    }

    #[inline]
    pub fn into_inner(self) -> Result<[T; N], Self> {
        if !self.is_full() {
            Err(self)
        } else {
            unsafe {
                let array = self.array.as_ptr() as *const [T; N];
                let array = ptr::read(array);
                mem::forget(self);
                Ok(array)
            }
        }
    }
}

impl <T, const N: usize> Drop for ArrayVec<T, {N}> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl <T, const N: usize> Deref for ArrayVec<T, {N}> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            let first_ptr: *const MaybeUninit<T> = self.array.as_ptr();
            let first_ptr: *const T = first_ptr as *const T;

            slice::from_raw_parts(first_ptr, self.len())
        }
    }
}

impl <T, const N: usize> DerefMut for ArrayVec<T, {N}> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let first_ptr: *mut MaybeUninit<T> = self.array.as_mut_ptr();
            let first_ptr: *mut T = first_ptr as *mut T;

            slice::from_raw_parts_mut(first_ptr, self.len())
        }
    }
}

impl <T, const N: usize> Extend<T> for ArrayVec<T, {N}> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>
    {
        let take = self.remaining_capacity();
        
        let mut iter = iter.into_iter();

        let array = &mut self.array;
        let len = &mut self.len;
        
        unsafe {
            for dst in array.get_unchecked_mut(*len..*len + take) {
                match iter.next() {
                    Some(item) => {
                        ptr::write(dst.as_mut_ptr(), item);
                        *len += 1;
                    }
                    None => break,
                }
            }
        }
    }
}

impl <T, const N: usize> FromIterator<T> for ArrayVec<T, {N}> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>
    {
        let mut array = ArrayVec::<T, {N}>::default();
        array.extend(iter);
        array
    }
}

impl <'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, {N}> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl <'a, T, const N: usize> IntoIterator for &'a mut ArrayVec<T, {N}> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl <T, const N: usize> IntoIterator for ArrayVec<T, {N}> {
    type Item = T;
    type IntoIter = IntoIter<T, {N}>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::<T, {N}> {
            array: self,
            index: 0,
        }
    }
}

pub struct IntoIter<T, const N: usize> {
    array: ArrayVec<T, {N}>,
    index: usize,
}

impl <T, const N: usize> Drop for IntoIter<T, {N}> {
    fn drop(&mut self) {
        let len = self.array.len();
        let index = self.index;

        unsafe {
            // Drop the elements between index..len.
            
            self.array.set_len(0);
            let elements: *mut [T] = self.array.get_unchecked_mut(index..len);
            ptr::drop_in_place(elements);
        }
    }
}

impl <T, const N: usize> Iterator for IntoIter<T, {N}> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.array.len() {
            None
        } else {
            unsafe {
                let elem = self.array.take(self.index);
                self.index += 1;
                Some(elem)
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.array.len() - self.index;
        (len, Some(len))
    }
}

impl <T, const N: usize> DoubleEndedIterator for IntoIter<T, {N}> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == self.array.len() {
            None
        } else {
            unsafe {
                let new_len = self.array.len() - 1;
                let elem = self.array.take(new_len);
                self.array.set_len(new_len);
                Some(elem)
            }
        }
    }
}

impl <T, const N: usize> ExactSizeIterator for IntoIter<T, {N}> {}
impl <T, const N: usize> FusedIterator for IntoIter<T, {N}> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn length_erasure<T, const N: usize>(_: &ArrayVec<T, {N}>) {

    }

    #[test]
    fn test_to_ensure_it_doesnt_ice() {

        let mut v = ArrayVec::<u8, { 399 }>::default();

        length_erasure(&v);

        assert!(v.is_empty());
        assert!(!v.is_full());
        assert_eq!(v.capacity(), 399);

        v.push(20);
        v.push(30);

        assert_eq!(v.len(), 2);

        assert_eq!(v.pop(), Some(30));
        assert_eq!(v.len(), 1);

        v.extend(0..10);
        
        assert_eq!(&*v, &[20, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(v.len(), 11);

        v.clear();

        assert!(v.is_empty());

        assert!(v.into_inner().is_err());

    }

    #[test]
    fn into_iter() {
        let mut v = ArrayVec::<usize, {10}>::default();

        for n in 0..v.capacity() {
            v.push(n);
        }

        let mut iter = v.into_iter();

        assert_eq!(iter.next_back(), Some(9));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(8));
        assert_eq!(iter.next_back(), Some(7));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(6));
        assert_eq!(iter.next_back(), Some(5));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);

        let mut v: ArrayVec<i32, {10}> = (0..10).collect();

        let mut iter = v.into_iter();
    }

    #[test]
    fn empty_array_vec() {
        let mut v: ArrayVec<String, {0}> = std::iter::repeat(String::from("WORLD")).take(100).collect();
        mem::forget(v);
    }

    #[test]
    fn test_collect() {
        let mut v: ArrayVec<String, {10}> = std::iter::repeat(String::from("Helloooooo there")).take(100).collect();
        assert_eq!(v.len(), 10);
        let array = v.into_inner();
    }

    #[test]
    //#[should_panic]
    fn panic_while_truncate() {
        struct DropPanic(Box<usize>);

        let mut v: ArrayVec<DropPanic, {20}> = (0..10).map(Box::new).map(DropPanic).collect();

        v.truncate(5);
    } 
}
