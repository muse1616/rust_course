use std::alloc::{self};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::vec::IntoIter;
use std::{alloc::Layout, mem, ptr::NonNull};
pub struct Vec<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}
unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Vec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "ZSTS");
        Vec {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
        }
    }
    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.cap;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };
        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );
        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout), // Abort on memory allocation error or failure
        };
        self.cap = new_cap;
    }
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }
        unsafe {
            // write value at offset
            ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }
        self.len += 1;
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.cap == self.len {
            self.grow();
        }
        unsafe {
            ptr::copy(
                self.ptr.as_ptr().add(index),
                self.ptr.as_ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr.as_ptr().add(index), elem);
        }
        self.len += 1;
    }
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            None
        } else {
            self.len -= 1;
            unsafe {
                let result = { ptr::read(self.ptr.as_ptr().add(index)) };
                ptr::copy(
                    self.ptr.as_ptr().add(index + 1),
                    self.ptr.as_ptr().add(index),
                    self.len - index,
                );
                Some(result)
            }
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            while let Some(_) = self.pop() {}
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}
impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let mut a: Vec<i32> = Vec::new();
        // let a :Vec<()> = Vec::new();
        a.push(123);
        assert_eq!(a[0], 123);
        assert_eq!(a.pop(), Some(123));
    }
}
