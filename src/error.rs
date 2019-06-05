
use core::{
    fmt::{self, Debug},
};

pub struct CapacityError<T>(T);

impl <T> CapacityError<T> {
    #[inline]
    pub const fn new(item: T) -> Self {
        Self(item)
    }
}

impl <T> Debug for CapacityError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("CapacityError { .. }")   
    }
}

