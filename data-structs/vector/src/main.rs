use vector::Vector;

fn main() {
    let mut v: Vector<PanicDrop> = Vector::new(5);
    let first = PanicDrop { i: 1 };
    let second = PanicDrop { i: 2 };
    let third = PanicDrop { i: 3 };
    v.push(first);
    v.push(second);
    v.push(third);
    drop(v);
}

struct PanicDrop {
    i: u8,
}

impl Drop for PanicDrop {
    fn drop(&mut self) {
        panic!("{:?} panicking!", self.i);
    }
}
