use parity_byte::checksum;
use std::fs::read;

fn main() {
    let data = read("./pg2600.txt").unwrap();
    let compare = read("./pg2600.txt").unwrap();
    let result = checksum(&data, &compare);
    println!("{:?}", result);
}
