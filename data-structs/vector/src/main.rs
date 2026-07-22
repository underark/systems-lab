use vector::Vector;

fn main() {
    let mut v = Vector::new();
    v.push(1);
    v.push(2);
    for x in &mut v {
        *x += 1;
    }
    println!("{:?}", v);
}
