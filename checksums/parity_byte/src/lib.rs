pub fn checksum(original: &[u8], compare: &[u8]) -> bool {
    if original.len() != compare.len() {
        return false;
    }

    xor_all(original) ^ xor_all(compare) == 0x00
}

pub fn xor_all(b: &[u8]) -> u8 {
    let mut current = 0x00;
    for byte in b.iter() {
        current ^= byte;
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len_one() {
        assert_eq!(xor_all(&[0x01]), 0x01);
    }

    #[test]
    fn test_len_two() {
        assert_eq!(xor_all(&[0x01, 0x01]), 0x00)
    }

    #[test]
    fn test_len_ten() {
        assert_eq!(xor_all(&[0x01; 10]), 0x00);
    }

    #[test]
    fn test_various_bits() {
        assert_eq!(xor_all(&[0x0F, 0x0E]), 0x01);
    }
}
