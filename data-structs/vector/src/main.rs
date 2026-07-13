use vector::Vector;
fn main() {
    let mut v = Vector::new();
    v.push(());
    let e = v.pop().unwrap();
    println!("{:?}", e);
}
