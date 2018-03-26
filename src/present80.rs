use std::fmt;

const KEY_LENGTH_IN_BYTES: usize = 10;

#[derive(Clone, Copy)]
pub struct Key {
    bytes: [u8; KEY_LENGTH_IN_BYTES],
}

struct KeyRegister {
    a: u64,
    b: u64,
}

impl Key {
    pub fn new(bytes: &[u8]) -> Key {
        let mut b = [0u8; KEY_LENGTH_IN_BYTES];
        match bytes.len() {
            0 => {}
            1...KEY_LENGTH_IN_BYTES => b[..bytes.len()].copy_from_slice(bytes),
            _ => b.copy_from_slice(&bytes[..KEY_LENGTH_IN_BYTES]),
        }

        Key { bytes: b }
    }
}

impl KeyRegister {
    fn rotate(&mut self) {
        let w = self.a & 0b1111111111111111111111111111111111111111111110000000000000000000;
        let x = self.a & 0b0000000000000000000000000000000000000000000001111111111111111000;
        let y = self.a & 0b0000000000000000000000000000000000000000000000000000000000000111;
        let z = self.b & 0b1111111111111111000000000000000000000000000000000000000000000000;

        let a = (y << 61) + (z >> 3) + (w >> 19);
        let b = x << 45;

        self.a = a;
        self.b = b;
    }

    fn update2(&mut self) {
        let w = (self.a >> 60) & 0xf;
        let x = super::S_BOX[w as usize];
        let y = (x as u64) << 60;
        let z = self.a & 0x0fffffffffffffff;

        let a = y + z;
        let b = self.b;

        self.a = a;
        self.b = b;
    }

    fn update3(&mut self, round_counter: u64) {
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

        self.a = a;
        self.b = b;
    }

    fn update(&mut self, round_counter: u64) {
        self.rotate();
        self.update2();
        self.update3(round_counter);
    }
}

impl From<Key> for KeyRegister {
    fn from(key: Key) -> Self {
        let (mut a, mut b) = (0u64, 0u64);
        for (i, x) in key.bytes.iter().enumerate() {
            if i > KEY_LENGTH_IN_BYTES - 1 {
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

        KeyRegister { a, b }
    }
}

impl fmt::Debug for KeyRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KeyRegister {{ a: {:b}, b: {:b} }}", self.a, self.b)
    }
}

fn encrypt(state: u64, key: Key) -> u64 {
    let mut state = state;
    let mut key_register = KeyRegister::from(key);

    for i in 0..super::NUM_ROUNDS {
        let round_key = key_register.a;
        state = super::add_round_key(state, round_key);
        state = super::s_box_layer(state);
        state = super::p_layer(state);

        key_register.update((i + 1) as u64);
    }

    let round_key = key_register.a;
    state = super::add_round_key(state, round_key);

    state
}

pub fn ecb_encrypt(data: &[u8], key: Key) -> Vec<u8> {
    let padded = super::pad(data);
    let num_blocks = padded.len();

    let mut encrypted: Vec<u8> = Vec::with_capacity(num_blocks);

    for i in 0..num_blocks / 8 {
        let encrypted_block = _encrypt_block(&padded[8 * i..8 * (i + 1)], key);
        encrypted.extend(encrypted_block.iter());
    }

    encrypted
}

fn _encrypt_block(data: &[u8], key: Key) -> [u8; super::BLOCK_SIZE_IN_BITS / 8] {
    let state = super::bytes_to_state(data);
    let encrypted = encrypt(state, key);

    super::state_to_bytes(encrypted)
}

pub fn encrypt_block(data: &[u8], key: Key) -> [u8; super::BLOCK_SIZE_IN_BITS / 8] {
    if data.len() < super::BLOCK_SIZE_IN_BYTES {
        let mut padded = [0u8; super::BLOCK_SIZE_IN_BYTES];
        &padded[..data.len()].copy_from_slice(data);
        _encrypt_block(&padded[..], key)
    } else {
        _encrypt_block(&data[..super::BLOCK_SIZE_IN_BYTES], key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_register_from_key1() {
        let key = Key::new(&[0, 0, 0, 0, 0, 1]);
        let key_register = KeyRegister::from(key);
        assert_eq!(key_register.a, 1u64 << 16);
        assert_eq!(key_register.b, 0u64);
    }

    #[test]
    fn test_key_register_from_key2() {
        let key = Key::new(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 0]);
        let key_register = KeyRegister::from(key);
        assert_eq!(key_register.a, 1u64);
        assert_eq!(key_register.b, 0u64);
    }

    #[test]
    fn test_key_register_from_key3() {
        let key = Key::new(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 1]);
        let key_register = KeyRegister::from(key);
        assert_eq!(key_register.a, 1u64);
        assert_eq!(key_register.b, 1u64 << 48);
    }

    #[test]
    fn test_key_register_from_key4() {
        let key = Key::new(&[0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1]);
        let key_register = KeyRegister::from(key);
        assert_eq!(key_register.a, 1u64);
        assert_eq!(key_register.b, 1u64 << 48);
    }

    #[test]
    fn test_key_register_rotate() {
        let mut key_register = KeyRegister { a: 0b1100, b: 0 };
        key_register.rotate();

        let a: u64 = 1 << 63;
        let b: u64 = 1 << 48;

        assert_eq!(a, key_register.a);
        assert_eq!(b, key_register.b);
    }

    #[test]
    fn test_key_register_update1() {
        let mut key_register = KeyRegister { a: 0, b: 0 };
        key_register.update(1);

        let a: u64 = 0b11 << 62;
        let b: u64 = 1 << 63;

        assert_eq!(a, key_register.a);
        assert_eq!(b, key_register.b);
    }
}
