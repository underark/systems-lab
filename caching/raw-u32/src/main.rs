use std::hint::black_box;
const SIZE: usize = 1000000;

fn main() {
    let mut v: Vec<u32> = black_box(vec![0; SIZE]);
    inc_one(&mut v);
    black_box(&v);
}

#[inline(never)]
fn inc_one(vec: &mut Vec<u32>) {
    for n in vec {
        *n += 1;
    }
}
