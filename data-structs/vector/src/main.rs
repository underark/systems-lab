use vector::Vector;

fn main() {
    let mut v: Vector<Tattler> = Vector::new();
    v.push(Tattler(" Hello"));
    v.push(Tattler(" world"));
}

#[derive(Debug)]
struct Tattler(&'static str);

impl Drop for Tattler {
    fn drop(&mut self) {
        println!("Dropping {}", self.0);
    }
}
