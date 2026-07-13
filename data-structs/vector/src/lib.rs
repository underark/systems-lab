use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::fmt::Result as FmtResult;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::drop_in_place;

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

    // NOTE: it is possible for this function to fail, but no Result<> type is returned because all
    // failure results in unwinding
    fn allocate_memory(size: usize) -> VectorAlloc<T> {
        let t_size = size_of::<T>();
        let align = align_of::<T>();

        // from_size_align will panic with a debug error message if certain conditions (see docs)
        // are not true. Would rather fail fast if one of these invariants is not true
        let l = Layout::from_size_align(t_size * size, align).unwrap();
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
        let new_size = self.capacity * 2;
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
