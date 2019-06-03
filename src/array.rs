use core::mem::MaybeUninit;

pub trait Array {
    /// The array's element type.
    type Item;

    type UninitArray: UninitArrayExt<UninitItem = <Self as Array>::Item>;

    fn as_ptr(&self) -> *const Self::Item;

    fn as_mut_ptr(&mut self) -> *mut Self::Item;

    fn capacity() -> usize;
}

impl<T, const N: usize> Array for [T; N] {
    type Item = T;

    type UninitArray = [MaybeUninit<T>; N];

    fn as_ptr(&self) -> *const Self::Item {
        self as *const Self as *const Self::Item
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self as *mut Self as *mut Self::Item
    }

    fn capacity() -> usize {
        N
    }
}

pub trait UninitArrayExt: Array<Item = MaybeUninit<<Self as UninitArrayExt>::UninitItem>> {
    type UninitItem;
    fn uninit() -> Self;
}

impl<T, const N: usize> UninitArrayExt for [MaybeUninit<T>; N] {
    type UninitItem = T;
    fn uninit() -> Self {
        unsafe { MaybeUninit::<Self>::uninit().assume_init() }
    }
}
