pub mod present80;
pub mod present128;

pub const BLOCK_SIZE_IN_BYTES: usize = 8;
pub(crate) const NUM_ROUNDS: usize = 31;
pub(crate) const S_BOX: [u8; 16] = [0xC, 5, 6, 0xB, 9, 0, 0xA, 0xD, 3, 0xE, 0xF, 8, 4, 7, 1, 2];
pub(crate) const P: [u8; 64] = [
    0, 16, 32, 48, 1, 17, 33, 49, 2, 18, 34, 50, 3, 19, 35, 51, 4, 20, 36, 52, 5, 21, 37, 53, 6,
    22, 38, 54, 7, 23, 39, 55, 8, 24, 40, 56, 9, 25, 41, 57, 10, 26, 42, 58, 11, 27, 43, 59, 12,
    28, 44, 60, 13, 29, 45, 61, 14, 30, 46, 62, 15, 31, 47, 63,
];

pub(crate) fn pad(data: &[u8]) -> Vec<u8> {
    let num_blocks = match (data.len() / 8, data.len() % 8) {
        (quo, 0) => quo,
        (quo, _) => quo + 1,
    };

    let mut padded: Vec<u8> = Vec::with_capacity(num_blocks * 8);
    padded.extend(data.iter());
    padded.resize(num_blocks * 8, 0);
    padded
}

pub(crate) fn s_box_layer(state: u64) -> u64 {
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

pub(crate) fn p_layer(state: u64) -> u64 {
    let mut new_state = 0u64;

    for (i, pi) in P.iter().enumerate() {
        let mask = 1 << i;
        let x = (state & mask) >> i;
        let y = x << *pi;
        new_state |= y;
    }

    new_state
}

pub(crate) fn add_round_key(state: u64, round_key: u64) -> u64 {
    state ^ round_key
}

pub(crate) fn bytes_to_state(bytes: &[u8]) -> u64 {
    let mut state = 0u64;
    for i in 0..BLOCK_SIZE_IN_BYTES {
        let x = (bytes[i] as u64) << (7 - i) * 8;
        state |= x as u64;
    }
    state
}

pub(crate) fn state_to_bytes(state: u64) -> [u8; BLOCK_SIZE_IN_BYTES] {
    let mut bytes = [0u8; BLOCK_SIZE_IN_BYTES];
    for i in 0..BLOCK_SIZE_IN_BYTES {
        let x = (state >> (7 - i) * 8) & 0xff;
        bytes[i] = x as u8;
    }
    bytes
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
        let round_key = 0b0101010101010101010101010101010101010101010101010101010101010101u64;

        let new_state = add_round_key(state, round_key);

        let expected = 0b0110011001100110011001100110011001100110011001100110011001100110u64;
        assert_eq!(expected, new_state);
    }

    #[test]
    fn test_bytes_to_state() {
        let bytes = [0x30u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
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
}
