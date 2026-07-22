use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::fmt::Result as FmtResult;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::{NonNull, drop_in_place};

pub struct Vector<T> {
    start: NonNull<T>,
    capacity: usize,
    length: usize,
    phantom: PhantomData<T>,
}

impl<T> Vector<T> {
    pub fn new() -> Vector<T> {
        let capacity = if size_of::<T>() == 0 { isize::MAX } else { 0 };
        Vector {
            start: NonNull::dangling(),
            capacity: capacity as usize,
            length: 0,
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn is_null(&self) -> bool {
        self.capacity == 0
    }

    // length <= capacity
    // elements 0..length - 1 are initialized
    // elements length..capacity - 1 are uninitialized
    // absolute max number of elements is <= isize::MAX / size_of::<T>()
    pub fn push(&mut self, t: T) {
        if self.length == self.capacity {
            self.grow_vector();
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

    pub fn insert(&mut self, index: usize, e: T) {
        assert!(index <= self.len());

        if self.length == self.capacity {
            self.grow_vector();
        }

        self.shuffle_one_higher(index);
        unsafe { self.start.add(index).write(e) };
        self.length += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len());

        unsafe {
            let t = self.start.add(index).read();
            self.length -= 1;
            for i in 0..(self.length - index) {
                self.start
                    .add(index + i)
                    .write(self.start.add(index + i + 1).read());
            }
            t
        }
    }

    fn grow_vector(&mut self) {
        // ZST should not try to grow the vector
        // growing the vector means that it is necessarily at absolute max capacity for ZST
        assert!(size_of::<T>() != 0);

        let new_size = if self.capacity == 0 {
            1
        } else {
            self.capacity
                .checked_mul(2)
                .expect("overflow usize on requested buffer size")
        };

        let new_allocation = Vector::<T>::allocate_memory(new_size);
        if !self.is_empty() {
            self.move_elements_to(new_allocation);
            // SAFETY: current 'capacity' value was successfully used to obtain Layout of current cap
            // should be sound to obtain Layout with same values
            unsafe {
                let layout =
                    Layout::from_size_align(self.capacity * size_of::<T>(), align_of::<T>())
                        .unwrap();
                dealloc(self.start.as_ptr().cast(), layout);
            }
        }
        self.update_vec_metadata(new_size, new_allocation);
    }

    fn move_elements_to(&mut self, new_ptr: *mut u8) {
        // SAFETY: elements 0 to n-1 are valid Ts and read() will move ownership to the new pointer
        unsafe {
            let t_ptr: *mut T = new_ptr.cast();
            for i in 0..self.length {
                t_ptr.add(i).write(self.start.add(i).read());
            }
        }
    }

    fn shuffle_one_higher(&mut self, low_index: usize) {
        for i in 0..(self.length - low_index) {
            unsafe {
                self.start
                    .add(self.length - i)
                    .write(self.start.add(self.length - i - 1).read());
            }
        }
    }

    fn allocate_memory(elements: usize) -> *mut u8 {
        let size = elements
            .checked_mul(size_of::<T>())
            .expect("overflow usize on requested buffer size");

        let layout = Layout::from_size_align(size, align_of::<T>())
            .expect("unable to provide valid layout for alloc");
        // SAFETY: from_size_align will effectively filter out improper layouts
        // ZST will never request memory due to assert! in grow_vector (allocation for 0-size is UB)
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                handle_alloc_error(layout);
            }
            ptr
        }
    }

    fn update_vec_metadata(&mut self, new_cap: usize, new_alloc: *mut u8) {
        self.capacity = new_cap;
        self.start = NonNull::new(new_alloc.cast()).unwrap();
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

            if !self.is_null() {
                let layout =
                    Layout::from_size_align(self.capacity * size_of::<T>(), align_of::<T>())
                        .unwrap();
                dealloc(self.start.as_ptr().cast(), layout);
            }
        }
    }
}

pub struct VecIntoIter<T> {
    buffer: *mut T,
    current: usize,
    size: usize,
    capacity: usize,
}

impl<T> Iterator for VecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.size {
            None
        } else {
            unsafe {
                let item = self.buffer.add(self.current).read();
                self.current += 1;
                Some(item)
            }
        }
    }
}

impl<T> Drop for VecIntoIter<T> {
    fn drop(&mut self) {
        unsafe {
            for i in self.current..self.size {
                drop_in_place(self.buffer.add(i));
            }

            if self.capacity > 0 {
                let layout =
                    Layout::from_size_align(self.capacity * size_of::<T>(), align_of::<T>())
                        .expect("invalid layout request in VecIntoIter drop");
                dealloc(self.buffer.cast(), layout);
            }
        }
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = VecIntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        let i = VecIntoIter::<T> {
            buffer: self.start.as_ptr(),
            current: 0,
            size: self.len(),
            capacity: self.capacity(),
        };
        self.length = 0;
        self.capacity = 0;
        i
    }
}

// at the end of into_inter, the vector itself will be dropped
// 1) dropping the vector means droping the elements in the vector - which the vector no longer owns
// 2) drop will also free the underlying memory - which the vector no longer owns
