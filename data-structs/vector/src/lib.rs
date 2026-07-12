use std::alloc::{Layout, alloc, dealloc};
use std::fmt::{self};
use std::marker::PhantomData;
use std::ptr::drop_in_place;

pub struct Vector<T> {
    start: *mut T,
    capacity: usize,
    length: usize,
    phantom: PhantomData<T>,
}

impl<T> Vector<T> {
    pub fn new() -> Vector<T> {
        let p: *mut T = Vector::allocate_memory(1);
        Vector {
            start: p,
            capacity: 1,
            length: 0,
            phantom: PhantomData,
        }
    }

    pub fn push(&mut self, t: T) {
        if self.length >= self.capacity {
            self.migrate_vector();
        }

        unsafe {
            self.start.add(self.length).write(t);
        }

        self.length += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            return None;
        }

        unsafe {
            let p = self.start.add(self.length).read();
            self.length -= 1;
            Some(p)
        }
    }

    pub fn allocate_memory(size: usize) -> *mut T {
        let l = Layout::from_size_align(size_of::<T>() * size, align_of::<T>()).unwrap();
        unsafe { alloc(l).cast() }
    }

    pub fn migrate_vector(&mut self) {
        let layout =
            Layout::from_size_align(size_of::<T>() * self.capacity, align_of::<T>()).unwrap();
        let new_size = self.capacity * 2;
        let new_p: *mut T = Vector::allocate_memory(new_size);
        // This assumes that the vector is full before reassigning - consider changing this
        // assumption at some point so that it reallocates earlier
        unsafe {
            for i in 0..self.capacity {
                new_p.add(i).write(self.start.add(i).read());
            }
            dealloc(self.start.cast(), layout);
        }
        self.start = new_p;
        self.capacity = new_size;
    }
}

impl<T: fmt::Debug> fmt::Debug for Vector<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            f.debug_list()
                .entries((0..self.length).map(|i| &*self.start.add(i)))
                .finish()
        }
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        for i in 0..self.length {
            unsafe {
                drop_in_place(self.start.add(i));
            }
        }
    }
}
