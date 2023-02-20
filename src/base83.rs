pub const ALPHABET: [u8; 83] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
    b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V',
    b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l',
    b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'#', b'$',
    b'%', b'*', b'+', b',', b'-', b'.', b':', b';', b'=', b'?', b'@', b'[', b']', b'^', b'_', b'{',
    b'|', b'}', b'~',
];
pub const ALPHABET_SIZE: usize = 83;

pub fn encode(value: usize, length: usize) -> String {
    (1..=length).fold(String::with_capacity(length), |mut acc, i| {
        let index = (value / usize::pow(ALPHABET_SIZE, (length - i) as u32)) % 83;
        acc.push(ALPHABET[index] as char);
        acc
    })
}

pub fn encode_into(dest: &mut String, value: usize, length: usize) {
    (1..=length).for_each(|i| {
        let index = (value / usize::pow(ALPHABET_SIZE, (length - i) as u32)) % 83;
        dest.push(ALPHABET[index] as char)
    })
}

pub fn encode_char(value: usize) -> char {
    let index = (value / ALPHABET_SIZE) % 83;
    ALPHABET[index] as char
}
