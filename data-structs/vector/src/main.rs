use vector::Vector;

fn main() {
    let mut v: Vector<u8> = Vector::new();
    v.push(0);
    v.push(1);
    v.push(2);
    v.push(3);
    v.insert(1, 4);
    println!("{:?}", v);
}
