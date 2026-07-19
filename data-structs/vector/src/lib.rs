use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fmt::{Display, Result as FmtResult};
use std::marker::PhantomData;
use std::panic;
use std::ptr::{NonNull, drop_in_place};

pub struct Vector<T> {
    start: NonNull<T>,
    capacity: usize,
    length: usize,
    phantom: PhantomData<T>,
}

struct VecAlloc {
    ptr: *mut u8,
    layout: Layout,
}

#[derive(Debug)]
struct OverflowError {}

impl Display for OverflowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "size calculation overflowed")
    }
}

impl Error for OverflowError {}

impl<T> Vector<T> {
    pub fn new() -> Vector<T> {
        if size_of::<T>() == 0 {
            panic!("vector cannot be initialized with zero-size type");
        }

        Vector {
            start: NonNull::dangling(),
            capacity: 0,
            length: 0,
            phantom: PhantomData,
        }
    }

    // length <= capacity
    // elements 0..length - 1 are initialized
    // elements length..capacity - 1 are uninitialized
    // absolute max number of elements is <= isize::MAX / size_of::<T>()
    pub fn push(&mut self, t: T) {
        if self.length == self.capacity {
            self.migrate_vector().unwrap();
        }

        unsafe {
            self.start.add(self.length).write(t);
            self.length += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            unsafe { Some(self.start.add(self.length).read()) }
        }
    }

    // question: how am i meeting allocator contracts?
    fn allocate_memory(elements: usize) -> Result<VecAlloc, Box<dyn Error>> {
        let size = elements
            .checked_mul(size_of::<T>())
            .ok_or(OverflowError {})?;
        let layout = Layout::from_size_align(size, align_of::<T>())?;
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                handle_alloc_error(layout);
            }
            Ok(VecAlloc { ptr, layout })
        }
    }

    fn migrate_vector(&mut self) -> Result<(), Box<dyn Error>> {
        let new_size = self.capacity.checked_mul(2).ok_or(OverflowError {})?;
        let new_allocation = Vector::<T>::allocate_memory(new_size)?;
        self.move_elements_to(new_allocation.ptr);
        // SAFETY: current 'capacity' value was successfully used to obtain Layout of current cap
        // should be sound to obtain Layout with same values
        unsafe {
            let layout = Layout::from_size_align(self.capacity, align_of::<T>()).unwrap();
            dealloc(self.start.as_ptr().cast(), layout);
        }
        self.update_vec_metadata(new_size, new_allocation);
        Ok(())
    }

    fn move_elements_to(&mut self, new_ptr: *mut u8) {
        unsafe {
            let t_ptr: *mut T = new_ptr.cast();
            for i in 0..self.length {
                t_ptr.add(i).write(self.start.add(i).read());
            }
        }
    }

    fn update_vec_metadata(&mut self, new_cap: usize, new_alloc: VecAlloc) {
        self.capacity = new_cap;
        self.start = NonNull::new(new_alloc.ptr.cast()).unwrap();
    }
}

impl<T: Debug> Debug for Vector<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_list()
            // SAFETY:
            .entries((0..self.length).map(|i| unsafe { &*self.start.add(i).as_ptr() }))
            .finish()
    }
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.length {
                drop_in_place(self.start.add(i).as_ptr());
            }
            let layout = Layout::from_size_align(self.capacity, align_of::<T>()).unwrap();
            dealloc(self.start.as_ptr().cast(), layout);
        }
    }
}
