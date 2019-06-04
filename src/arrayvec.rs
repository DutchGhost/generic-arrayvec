use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr, slice,
};

use crate::{array::Array, uninitarray::UninitArray};

pub struct ArrayVec<A: Array> {
    array: UninitArray<A>,
    len: usize,
}

impl<A: Array> ArrayVec<A> {
    pub fn new() -> Self {
        Self {
            array: UninitArray::new(),
            len: 0,
        }
    }

    pub fn is_empty(&mut self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }

    fn push(&mut self, item: A::Item) {
        self.try_push(item).unwrap()
    }

    fn try_push(&mut self, item: A::Item) -> Result<(), ()> {
        if self.len < A::capacity() {
            unsafe { self.push_unchecked(item) }
            Ok(())
        } else {
            Err(())
        }
    }

    pub unsafe fn push_unchecked(&mut self, item: A::Item) {
        let len = self.len();
        debug_assert!(len < A::capacity());
        let item = MaybeUninit::new(item);
        ptr::write(self.array.get_unchecked_mut(len), item);
        self.set_len(len + 1);
    }

    pub fn pop(&mut self) -> Option<A::Item> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                let new_len = self.len() - 1;
                self.set_len(new_len);
                let element = self.array.get_unchecked_mut(new_len);
                let element = ptr::replace(element, MaybeUninit::uninit());
                Some(element.assume_init())
            }
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        unsafe {
            if new_len < self.len() {
                let ptr: *mut [A::Item] = &mut self[new_len..];
                self.len = new_len;
                ptr::drop_in_place(ptr);
            }
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }
}

impl<A: Array> Deref for ArrayVec<A> {
    type Target = [A::Item];

    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr: *const MaybeUninit<A::Item> = self.array.as_ptr();
            let ptr: *const A::Item = ptr as *const _;
            slice::from_raw_parts(ptr, self.len())
        }
    }
}

impl<A: Array> DerefMut for ArrayVec<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let ptr: *mut MaybeUninit<A::Item> = self.array.as_mut_ptr();
            let ptr: *mut A::Item = ptr as *mut _;
            slice::from_raw_parts_mut(ptr, self.len())
        }
    }
}

impl<A: Array> Drop for ArrayVec<A> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl<A: Array> IntoIterator for ArrayVec<A> {
    type Item = A::Item;
    type IntoIter = IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            array: self,
            index: 0,
        }
    }
}

pub struct IntoIter<A: Array> {
    array: ArrayVec<A>,
    index: usize,
}

impl<A: Array> Iterator for IntoIter<A> {
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.array.len {
            None
        } else {
            unsafe {
                let elem: *mut MaybeUninit<A::Item> =
                    self.array.array.get_unchecked_mut(self.index);
                let elem = ptr::replace(elem, MaybeUninit::uninit()).assume_init();
                self.index += 1;
                Some(elem)
            }
        }
    }
}

impl<A: Array> DoubleEndedIterator for IntoIter<A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index == self.array.len {
            None
        } else {
            unsafe {
                let new_len = self.array.len() - 1;
                self.array.set_len(new_len);
                let elem = self.array.array.get_unchecked_mut(new_len);
                let elem = ptr::replace(elem, MaybeUninit::uninit()).assume_init();
                Some(elem)
            }
        }
    }
}

impl<A: Array> Drop for IntoIter<A> {
    fn drop(&mut self) {
        let index = self.index;
        let len = self.array.len;

        unsafe {
            self.array.set_len(0);

            let elements =
                slice::from_raw_parts_mut(self.array.get_unchecked_mut(index), len - index);

            ptr::drop_in_place(elements)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut array = ArrayVec::<[Vec<usize>; 50]>::new();

        array.push(vec![1, 2]);
        array.push(vec![2, 3]);

        assert_eq!(array.len(), 2);

        assert_eq!(array.pop(), Some(vec![2, 3]));
        assert_eq!(array.pop(), Some(vec![1, 2]));

        assert!(array.is_empty());
    }

    #[test]
    fn unit_type_test() {
        let mut array = ArrayVec::<[(); std::usize::MAX]>::new();

        assert_eq!(std::mem::size_of::<ArrayVec<[(); std::usize::MAX]>>(), 8)
    }

    #[test]
    fn test_into_iter() {
        let mut array = ArrayVec::<[usize; 10]>::new();

        array.push(20);
        array.push(30);
        array.push(50);

        let mut iter = array.into_iter();

        assert_eq!(iter.next(), Some(20));
        assert_eq!(iter.next_back(), Some(50));
        assert_eq!(iter.next_back(), Some(30));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None)
    }

}
