extern crate hex;

const KEY_LENGTH: usize = 80;
const KEY_UPDATE_LEFT_ROTATION: usize = 61;
const BLOCK_SIZE: usize = 64;
const NUM_ROUNDS: usize = 31;
const S_BOX: [u8; 16] = [0xC, 5, 6, 0xB, 9, 0, 0xA, 0xD, 3, 0xE, 0xF, 8, 4, 7, 1, 2];
const P: [u8; 64] = [
    0, 16, 32, 48, 1, 17, 33, 49, 2, 18, 34, 50, 3, 19, 35, 51, 4, 20, 36, 52, 5, 21, 37, 53, 6,
    22, 38, 54, 7, 23, 39, 55, 8, 24, 40, 56, 9, 25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59, 12,
    28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63,
];

fn s_box(b: u8) -> u8 {
    S_BOX[b as usize]
}

fn rotate(key: &mut [u8; KEY_LENGTH]) {
    let temp = key.clone();
    let cut = KEY_LENGTH - KEY_UPDATE_LEFT_ROTATION;

    for i in 0..cut {
        key[i] = temp[i + KEY_UPDATE_LEFT_ROTATION];
    }

    for i in cut..KEY_LENGTH {
        key[i] = temp[i - cut as usize];
    }
}

fn key_update2(key: &mut [u8; KEY_LENGTH]) {
    let left4 = (key[0] << 3) + (key[1] << 2) + (key[2] << 1) + key[3];
    let sb_left4 = s_box(left4);

    key[0] = (sb_left4 >> 3) & 1;
    key[1] = (sb_left4 >> 2) & 1;
    key[2] = (sb_left4 >> 1) & 1;
    key[3] = sb_left4 & 1;
}

fn key_update3(key: &mut [u8; KEY_LENGTH], round_counter: u8) {
    let bits19_15 = (key[60] << 4) + (key[61] << 3) + (key[62] << 2) + (key[63] << 1) + key[64];
    let xor_bits19_15 = bits19_15 ^ round_counter;

    key[60] = (xor_bits19_15 >> 4) & 1;
    key[61] = (xor_bits19_15 >> 3) & 1;
    key[62] = (xor_bits19_15 >> 2) & 1;
    key[63] = (xor_bits19_15 >> 1) & 1;
    key[64] = xor_bits19_15 & 1;
}

fn update_key(key: &mut [u8; KEY_LENGTH], round_counter: u8) {
    rotate(key);
    key_update2(key);
    key_update3(key, round_counter);
}

fn s_box_layer(state: &mut [u8; BLOCK_SIZE]) {
    for i in 0..16 {
        let b = (state[4 * i] << 3) + (state[4 * i + 1] << 2) + (state[4 * i + 2] << 1)
            + state[4 * i + 3];
        let sb_b = s_box(b);

        state[4 * i] = (sb_b >> 3) & 1;
        state[4 * i + 1] = (sb_b >> 2) & 1;
        state[4 * i + 2] = (sb_b >> 1) & 1;
        state[4 * i + 3] = sb_b & 1;
    }
}

fn p_layer(state: &mut [u8; BLOCK_SIZE]) {
    let temp = state.clone();
    for (i, pi) in P.iter().enumerate() {
        state[*pi as usize] = temp[i];
    }
}

fn add_round_key(state: &mut [u8; BLOCK_SIZE], key: &[u8; KEY_LENGTH]) {
    for i in 0..BLOCK_SIZE {
        state[i] = state[i] ^ key[i];
    }
}

fn bytes_to_state(bytes: &[u8]) -> [u8; BLOCK_SIZE] {
    let mut state = [0; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE / 8 {
        for j in 0..8 {
            let bit = (bytes[i] >> (7 - j)) & 1;
            state[8 * i + j] = bit;
        }
    }
    state
}

fn bytes_to_key(bytes: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0; KEY_LENGTH];
    for i in 0..KEY_LENGTH / 8 {
        for j in 0..8 {
            let bit = (bytes[i] >> (7 - j)) & 1;
            key[8 * i + j] = bit;
        }
    }
    key
}

fn state_to_bytes(state: [u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE / 8] {
    let mut bytes = [0; BLOCK_SIZE / 8];
    for i in 0..BLOCK_SIZE / 8 {
        let mut byte: u8 = 0;
        for j in 0..8 {
            byte += state[i * 8 + j] << (7 - j);
        }
        bytes[i] = byte;
    }
    bytes
}

fn encrypt(state: &mut [u8; BLOCK_SIZE], key_register: &mut [u8; KEY_LENGTH]) {
    for i in 0..NUM_ROUNDS as u8 {
        add_round_key(state, key_register);
        s_box_layer(state);
        p_layer(state);

        update_key(key_register, i + 1);
    }

    add_round_key(state, &key_register);
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

pub fn ecb_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    let padded = pad(data);
    let length = padded.len();
    let mut encrypted: Vec<u8> = Vec::with_capacity(length);

    for i in 0..length / 8 {
        let encrypted_block = _encrypt_block(&padded[8 * i..8 * (i + 1)], key);
        encrypted.extend(encrypted_block.iter());
    }

    encrypted
}

fn _encrypt_block(data: &[u8], key: &[u8]) -> [u8; BLOCK_SIZE / 8] {
    debug_assert_eq!(data.len(), BLOCK_SIZE / 8);

    let mut state = bytes_to_state(data);
    let mut key_register = bytes_to_key(key);

    encrypt(&mut state, &mut key_register);

    state_to_bytes(state)
}

pub fn encrypt_block(data: &[u8], key: &[u8]) -> [u8; BLOCK_SIZE / 8] {
    if data.len() < BLOCK_SIZE / 8 {
        let data = pad(data);
        _encrypt_block(&data[..], key)
    } else {
        _encrypt_block(data, key)
    }
}

fn generate_round_keys(key: &[u8; KEY_LENGTH]) -> [[u8; BLOCK_SIZE]; NUM_ROUNDS + 1] {
    let mut key_register = key.clone();
    let mut round_keys = [[0; BLOCK_SIZE]; NUM_ROUNDS + 1];
    for i in 0..NUM_ROUNDS {
        round_keys[i].copy_from_slice(&key_register[..BLOCK_SIZE]);
        update_key(&mut key_register, (i + 1) as u8);
    }
    round_keys[NUM_ROUNDS].copy_from_slice(&key_register[..BLOCK_SIZE]);
    round_keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_rotate() {
        let mut key = [
            79, 78, 77, 76, 75, 74, 73, 72, 71, 70, 69, 68, 67, 66, 65, 64, 63, 62, 61, 60, 59, 58,
            57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45, 44, 43, 42, 41, 40, 39, 38, 37, 36,
            35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14,
            13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
        ];
        rotate(&mut key);
        let expected = [
            18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 79, 78, 77, 76, 75,
            74, 73, 72, 71, 70, 69, 68, 67, 66, 65, 64, 63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53,
            52, 51, 50, 49, 48, 47, 46, 45, 44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31,
            30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19,
        ];
        assert_eq!(expected[..], key[..]);
    }

    #[test]
    fn test_key_update2() {
        let mut key = [0; KEY_LENGTH];
        key[1] = 1;

        key_update2(&mut key);

        let expected = [
            1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(expected[..], key[..]);
    }

    #[test]
    fn test_key_update3() {
        let mut key = [0; KEY_LENGTH];
        key[60] = 1;

        key_update3(&mut key, 1);

        let expected = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(expected[..], key[..]);
    }

    #[test]
    fn test_update_key() {
        let mut key = [0; KEY_LENGTH];

        update_key(&mut key, 1);

        let expected = [
            1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(expected[..], key[..]);
    }

    #[test]
    fn test_s_box_layer() {
        let mut state = [
            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0,
            1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 1,
            1, 0, 1, 1, 1, 1,
        ];

        s_box_layer(&mut state);

        let expected = [
            1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 1,
            1, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0,
            0, 1, 0, 0, 1, 0,
        ];
        assert_eq!(expected[..], state[..]);
    }

    #[test]
    fn test_p_layer() {
        let mut state = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
        ];

        p_layer(&mut state);

        let expected = [
            0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 1, 5, 9, 13, 17, 21, 25,
            29, 33, 37, 41, 45, 49, 53, 57, 61, 2, 6, 10, 14, 18, 22, 26, 30, 34, 38, 42, 46, 50,
            54, 58, 62, 3, 7, 11, 15, 19, 23, 27, 31, 35, 39, 43, 47, 51, 55, 59, 63,
        ];
        assert_eq!(expected[..], state[..]);
    }

    #[test]
    fn test_add_round_key() {
        let mut state = [
            0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0,
            0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 1, 1,
        ];
        let key = [
            0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
            1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
            0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        add_round_key(&mut state, &key);

        let expected = [
            0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0,
            1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1,
            1, 0, 0, 1, 1, 0,
        ];
        assert_eq!(expected[..], state[..]);
    }

    #[test]
    fn test_bytes_to_state() {
        let bytes = hex::decode("3000000000000000").unwrap();
        let state = bytes_to_state(&bytes[..]);

        let expected = [
            0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(expected[..], state[..]);
    }

    #[test]
    fn test_bytes_to_key() {
        let bytes = hex::decode("30000000000000000000").unwrap();
        let state = bytes_to_key(&bytes[..]);

        let expected = [
            0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(expected[..], state[..]);
    }

    #[test]
    fn test_state_to_bytes() {
        let state = [
            0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ];
        let bytes = state_to_bytes(state);

        let expected = [48, 65, 0, 0, 0, 0, 0, 0];
        assert_eq!(expected[..], bytes[..]);
    }

    #[test]
    fn test_generate_round_keys() {
        let key = [
            0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
            1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1,
            0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let round_keys = generate_round_keys(&key);

        let k32 = [
            1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0,
            1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0,
            1, 1, 1, 0, 1, 1,
        ];
        assert_eq!(key[..BLOCK_SIZE], round_keys[0][..]);
        assert_eq!(k32[..], round_keys[NUM_ROUNDS][..]);
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
}
