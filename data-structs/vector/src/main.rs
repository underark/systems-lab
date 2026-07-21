use vector::Vector;

fn main() {
    let mut v: Vector<u8> = Vector::new();
    v.push(1);
    v.insert(0, 4);
    println!("{:?}", v);
}
