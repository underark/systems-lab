pub fn xor_all(b: &[u8]) -> u8 {
    let mut current = 0b00000000;
    for byte in b.iter() {
        current ^= byte;
    }
    current
}
