use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::fmt::Result as FmtResult;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::drop_in_place;

// Must always be true:
// length is the number of elements stored in the vector
// 0 to length - 1 elements are initialized
// length to capacity - 1 elements are uninitialized
// capacity * size_of::<T> is the total number of allocated bytes
// capacity * size_of::<T> cannot exceed isize::MAX

pub struct Vector<T> {
    start: *mut T,
    current_layout: Layout, // dealloc() expects to be called with the same layout that was used to
    // alloc the memory
    capacity: usize,
    length: usize,
    phantom: PhantomData<T>,
}

struct VectorAlloc<T> {
    start: *mut T,
    layout: Layout,
}

impl<T> Vector<T> {
    pub fn new() -> Vector<T> {
        let alloc: VectorAlloc<T> = Vector::allocate_memory(1);
        Vector {
            start: alloc.start,
            current_layout: alloc.layout,
            capacity: 1,
            length: 0,
            phantom: PhantomData,
        }
    }

    pub fn push(&mut self, t: T) {
        // returning early prevents allocations for zero-sized types
        if size_of::<T>() == 0 {
            self.length += 1;
            return;
        }

        if self.length >= self.capacity {
            self.migrate_vector();
        }

        // SAFETY: be careful of off by one errors. write to self.length; read from self.length - 1
        unsafe {
            self.start.add(self.length).write(t);
        }

        self.length += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            return None;
        }

        // SAFETY: be careful of off by one errors. write to self.length; read from self.length - 1
        // SAFETY: T returned from pop is moved out of allocation in vector but still exists in
        // memory; self.length + 1 must be treated as uninitialized
        unsafe {
            self.length -= 1;
            let p = self.start.add(self.length).read();
            Some(p)
        }
    }

    // NOTE: it is possible for this function to fail, but no Result<> type is returned because all
    // failure results in unwinding
    fn allocate_memory(size: usize) -> VectorAlloc<T> {
        // SAFETY: clamping at 1 ensures that zero-sized types never request 0 bytes, which is UB
        let t_size = size_of::<T>().clamp(1, isize::MAX as usize);
        let align = align_of::<T>();

        // from_size_align will panic at unwrap() if certain conditions are untrue (see docs)
        // this is only the case for values passed into from_size_align i.e. prior calculations must
        // not overflow, or else this will not be caught

        // maximum allocation size is isize::MAX because offset in methods like add() must be
        // addressable within an isize
        let l = Layout::from_size_align((t_size * size).max(isize::MAX as usize), align).unwrap();
        // SAFETY: pointer returned from alloc may be null. Calling handle_alloc_error for failed states
        unsafe {
            let p = alloc(l);
            if p.is_null() {
                handle_alloc_error(l);
            } else {
                VectorAlloc {
                    start: p.cast(),
                    layout: l,
                }
            }
        }
    }

    fn migrate_vector(&mut self) {
        // maximum allocation size is isize::MAX because offset in methods like add() must be
        // addressable within an isize
        let new_size = (self.length * 2).max(isize::MAX as usize);
        let alloc: VectorAlloc<T> = Vector::allocate_memory(new_size);

        // SAFETY: iterating from 0..length - 1 because these elements should be treated as
        // initialized - n >= length elements may contain moved out data
        // SAFETY: using previously stored layout ensures matching layout for a given alloc is used
        unsafe {
            for i in 0..self.length {
                alloc.start.add(i).write(self.start.add(i).read());
            }
            dealloc(self.start.cast(), self.current_layout);
        }
        self.start = alloc.start;
        self.current_layout = alloc.layout;
        self.capacity = new_size;
    }
}

impl<T: Debug> Debug for Vector<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        unsafe {
            f.debug_list()
                // NOTE: a zero-sized type will never actually increment the pointer (count *
                // size_of::<T>()) but print the first and only element n times
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
        unsafe { dealloc(self.start.cast(), self.current_layout) };
    }
}
