extern crate hex;

const KEY_LENGTH: usize = 80;
const BLOCK_SIZE: usize = 64;
const NUM_ROUNDS: usize = 31;
const S_BOX: [u8; 16] = [0xC, 5, 6, 0xB, 9, 0, 0xA, 0xD, 3, 0xE, 0xF, 8, 4, 7, 1, 2];
const P: [u8; 64] = [
    0, 16, 32, 48, 1, 17, 33, 49, 2, 18, 34, 50, 3, 19, 35, 51, 4, 20, 36, 52, 5, 21, 37, 53, 6,
    22, 38, 54, 7, 23, 39, 55, 8, 24, 40, 56, 9, 25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59, 12,
    28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63,
];

pub struct Key80 {
    a: u64,
    b: u64,
}

pub struct Key128 {
    a: u64,
    b: u64,
}

pub trait Key {
    fn get_round_key(&self) -> u64;
    fn next(&self, round_counter: u64) -> Self;
}

impl Key80 {
    pub fn new(bytes: &[u8]) -> Key80 {
        let (mut a, mut b) = (0u64, 0u64);
        for (i, x) in bytes.iter().enumerate() {
            if i > 10 {
                break;
            }

            let byte = *x as u64;

            if i < 8 {
                let shift = 56 - i * 8;
                a |= byte << shift;
            } else {
                let shift = 120 - i * 8;
                b |= byte << shift;
            }
        }

        Key80 { a, b }
    }

    fn rotate(&self) -> Key80 {
        let w = self.a & 0b1111111111111111111111111111111111111111111110000000000000000000;
        let x = self.a & 0b0000000000000000000000000000000000000000000001111111111111111000;
        let y = self.a & 0b0000000000000000000000000000000000000000000000000000000000000111;
        let z = self.b & 0b1111111111111111000000000000000000000000000000000000000000000000;

        let a = (y << 61) + (z >> 3) + (w >> 19);
        let b = x >> 3;

        Key80 { a, b }
    }

    fn update2(&self) -> Key80 {
        let w = (self.a >> 60) & 0xf;
        let x = S_BOX[w as usize];
        let y = (x as u64) << 60;
        let z = self.a & 0x0fffffffffffffff;

        let a = y + z;
        let b = self.b;

        Key80 { a, b }
    }

    fn update3(&self, round_counter: u64) -> Key80 {
        let w = (self.a & 0xf) << 1;
        let x = (self.b >> 63) & 1;
        let y = w + x;
        let z = y ^ round_counter;

        let p = (z & 0b11110) >> 1;
        let q = (z & 0b00001) << 63;
        let r = self.a & 0xfffffffffffffff0;
        let s = self.b & 0x7fffffffffffffff;

        let a = p + r;
        let b = q + s;

        Key80 { a, b }
    }
}

impl Key for Key80 {
    fn get_round_key(&self) -> u64 {
        self.a
    }

    fn next(&self, round_counter: u64) -> Key80 {
        self.rotate()
            .update2()
            .update3(round_counter)
    }
}

impl Key128 {
    pub fn new(bytes: &[u8]) -> Key128 {
        let (mut a, mut b) = (0u64, 0u64);
        for (i, x) in bytes.iter().enumerate() {
            if i > 16 {
                break;
            }

            let byte = *x as u64;

            if i < 8 {
                let shift = 56 - i * 8;
                a |= byte << shift;
            } else {
                let shift = 120 - i * 8;
                b |= byte << shift;
            }
        }

        Key128 { a, b }
    }
}

fn s_box_layer(state: u64) -> u64 {
    let mut new_state = 0u64;
    for i in 0..16 {
        let shift = i * 4;
        let mask = 0xf << shift;
        let x = (state & mask) >> shift;
        let y = S_BOX[x as usize] as u64;
        let z = y << shift;
        new_state |= z;
    }

    new_state
}

fn p_layer(state: u64) -> u64 {
    let mut new_state = 0u64;

    for (i, pi) in P.iter().enumerate() {
        let mask = 1 << i;
        let x = (state & mask) >> i;
        let y = x << *pi;
        new_state |= y;
    }

    new_state
}

fn add_round_key<T: Key>(state: u64, key: &T) -> u64 {
    let round_key = key.get_round_key();
    state ^ round_key
}

fn bytes_to_state(bytes: &[u8]) -> u64 {
    let mut state = 0u64;
    for i in 0..BLOCK_SIZE / 8 {
        let x = ((bytes[i] as u64) << (7 - i) * 8);
        state |= x as u64;
    }
    state
}

fn state_to_bytes(state: u64) -> [u8; BLOCK_SIZE / 8] {
    let mut bytes = [0u8; BLOCK_SIZE / 8];
    for i in 0..BLOCK_SIZE / 8 {
        let x = (state >> (7 - i) * 8) & 0xff;
        bytes[i] = x as u8;
    }
    bytes
}

fn encrypt<T: Key>(state: u64, key: &T) -> u64 {
    let mut state = state;
    let mut key = key;
    for i in 0..NUM_ROUNDS {
        state = add_round_key(state, key);
        state = s_box_layer(state);
        state = p_layer(state);

//        key = &key.next((i + 1) as u64);
    }

    state
}

fn pad(data: &[u8]) -> Vec<u8> {
    let num_blocks = match (data.len() / 8, data.len() % 8) {
        (quo, 0) => quo,
        (quo, _) => quo + 1,
    };

    let mut padded: Vec<u8> = Vec::with_capacity(num_blocks * 8);
    padded.extend(data.iter());
    padded.resize(num_blocks * 8, 0);
    padded
}

pub fn ecb_encrypt<T: Key>(data: &[u8], key: &T) -> Vec<u8> {
    let padded = pad(data);
    let length = padded.len();
    let mut encrypted: Vec<u8> = Vec::with_capacity(length);

    for i in 0..length / 8 {
        let encrypted_block = _encrypt_block(&padded[8 * i..8 * (i + 1)], key);
        encrypted.extend(encrypted_block.iter());
    }

    encrypted
}

fn _encrypt_block<T: Key>(data: &[u8], key: &T) -> [u8; BLOCK_SIZE / 8] {
    let state = bytes_to_state(data);
    let encrypted = encrypt(state, key);

    state_to_bytes(state)
}

pub fn encrypt_block<T: Key>(data: &[u8], key: &T) -> [u8; BLOCK_SIZE / 8] {
    if data.len() < BLOCK_SIZE / 8 {
        let data = pad(data);
        _encrypt_block(&data[..], key)
    } else {
        _encrypt_block(data, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s_box_layer() {
        let state = 0b0000000100100011010001010110011110001001101010111100110111101111u64;
        let new_state = s_box_layer(state);

        let expected = 0b1100010101101011100100001010110100111110111110000100011100010010u64;
        assert_eq!(expected, new_state);
    }

    #[test]
    fn test_p_layer() {
        let state = 0b1010101010101010101010101010101010101010101010101010101010101010u64;
        let new_state = p_layer(state);

        let expected = 0b1111111111111111000000000000000011111111111111110000000000000000u64;
        assert_eq!(expected, new_state);
    }

    #[test]
    fn test_add_round_key() {
        let state = 0b0011001100110011001100110011001100110011001100110011001100110011u64;
        let key = Key80 { a: 0b0101010101010101010101010101010101010101010101010101010101010101u64, b: 0u64 };

        let new_state = add_round_key(state, &key);

        let expected = 0b0110011001100110011001100110011001100110011001100110011001100110u64;
        assert_eq!(expected, new_state);
    }

    #[test]
    fn test_bytes_to_state() {
        let bytes = hex::decode("3000000000000000").unwrap();
        let state = bytes_to_state(&bytes[..]);

        let expected = 0b0011000000000000000000000000000000000000000000000000000000000000u64;
        assert_eq!(expected, state);
    }

    #[test]
    fn test_state_to_bytes() {
        let state = 0b0011000001000001000000000000000000000000000000000000000000000000u64;
        let bytes = state_to_bytes(state);

        let expected = [48, 65, 0, 0, 0, 0, 0, 0];
        assert_eq!(expected[..], bytes[..]);
    }

    #[test]
    fn test_pad1() {
        let data = [117, 121, 97, 105, 106];
        let padded = pad(&data);

        let expected = [117, 121, 97, 105, 106, 0, 0, 0];
        assert_eq!(expected[..], padded[..]);
    }

    #[test]
    fn test_pad2() {
        let data = [103, 110, 105, 121, 103, 110, 111, 114];
        let padded = pad(&data);

        let expected = [103, 110, 105, 121, 103, 110, 111, 114];
        assert_eq!(expected[..], padded[..]);
    }

    #[test]
    fn test_pad3() {
        let data = [
            121, 114, 114, 101, 104, 115, 105, 121, 110, 101, 119, 121, 97, 116
        ];
        let padded = pad(&data);

        let expected = [
            121, 114, 114, 101, 104, 115, 105, 121, 110, 101, 119, 121, 97, 116, 0, 0
        ];
        assert_eq!(expected[..], padded[..]);
    }

    #[test]
    fn test_key80_new1() {
        let key = Key80::new(&[0, 0, 0, 0, 0, 1]);
        assert_eq!(key.a, 1u64 << 16);
        assert_eq!(key.b, 0u64);
    }

    #[test]
    fn test_key80_new2() {
        let key = Key80::new(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 0]);
        assert_eq!(key.a, 1u64);
        assert_eq!(key.b, 0u64);
    }

    #[test]
    fn test_key80_new3() {
        let key = Key80::new(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 1]);
        assert_eq!(key.a, 1u64);
        assert_eq!(key.b, 1u64 << 48);
    }

    #[test]
    fn test_key80_key_register_update() {
        let mut key = Key80 { a: 0, b: 0 };
        key = key.next(1);

        let a: u64 = 0b11 << 62;
        let b: u64 = 1 << 63;

        assert_eq!(a, key.a);
        assert_eq!(b, key.b);
    }

    #[test]
    fn test_key128_new1() {
        let key = Key128::new(&[0, 0, 0, 0, 0, 1]);
        assert_eq!(key.a, 1u64 << 16);
        assert_eq!(key.b, 0u64);
    }

    #[test]
    fn test_key128_new2() {
        let key = Key128::new(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 0]);
        assert_eq!(key.a, 1u64);
        assert_eq!(key.b, 0u64);
    }

    #[test]
    fn test_key128_new3() {
        let key = Key128::new(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(key.a, 1u64);
        assert_eq!(key.b, 1u64);
    }
}
