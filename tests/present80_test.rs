extern crate hex;
extern crate present;

use present::present80;

macro_rules! test_encrypt_or_decrypt {
    ($name:ident, $f:ident, $i:expr, $k: expr, $e:expr) => {
        #[test]
        fn $name() {
            let input = hex::decode($i).unwrap();
            let key_bytes = hex::decode($k).unwrap();
            let key = present80::Key::new(&key_bytes[..]);

            let crypted = present80::$f(&input[..], key);
            let output = hex::encode_upper(&crypted[..]);

            let expected = $e;
            assert_eq!(expected, output);
        }
    }
}

macro_rules! test_block {
    ($enc:ident, $dec:ident, $k:expr, $p:expr, $c:expr) => {
        test_encrypt_or_decrypt!($enc, encrypt_block, $p, $k, $c);

        test_encrypt_or_decrypt!($dec, decrypt_block, $c, $k, $p);
    };
}

macro_rules! test_ecb {
    ($enc:ident, $dec:ident, $k:expr, $p:expr, $c:expr) => {
        test_encrypt_or_decrypt!($enc, ecb_encrypt, $p, $k, $c);

        test_encrypt_or_decrypt!($dec, ecb_decrypt, $c, $k, $p);
    };
}

macro_rules! test_par_ecb {
    ($enc:ident, $dec:ident, $k:expr, $p:expr, $c:expr) => {
        test_encrypt_or_decrypt!($enc, par_ecb_encrypt, $p, $k, $c);

        test_encrypt_or_decrypt!($dec, par_ecb_decrypt, $c, $k, $p);
    };
}

test_block!(
    test_encrypt_block1,
    test_decrypt_block1,
    "00000000000000000000",
    "0000000000000000",
    "5579C1387B228445"
);

#[test]
fn test_encrypt_block2() {
    let plaintext = hex::decode("0000000000000000").unwrap();
    let key_bytes = hex::decode("FFFFFFFFFFFFFFFFFFFF").unwrap();
    let key = present80::Key::new(&key_bytes[..]);

    let encrypted = present80::encrypt_block(&plaintext[..], key);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "E72C46C0F5945049";
    assert_eq!(expected, ciphertext);
}

#[test]
fn test_decrypt_block2() {
    let ciphertext = hex::decode("E72C46C0F5945049").unwrap();
    let key_bytes = hex::decode("FFFFFFFFFFFFFFFFFFFF").unwrap();
    let key = present80::Key::new(&key_bytes[..]);

    let decrypted = present80::decrypt_block(&ciphertext[..], key);
    let plaintext = hex::encode_upper(&decrypted[..]);

    let expected = "0000000000000000";
    assert_eq!(expected, plaintext);
}

test_block!(
    test_encrypt_block3,
    test_decrypt_block3,
    "00000000000000000000",
    "FFFFFFFFFFFFFFFF",
    "A112FFC72F68417B"
);

test_block!(
    test_encrypt_block4,
    test_decrypt_block4,
    "FFFFFFFFFFFFFFFFFFFF",
    "FFFFFFFFFFFFFFFF",
    "3333DCD3213210D2"
);

#[test]
fn test_ecb_encrypt1() {
    let plaintext = hex::decode("0000000000000000FFFFFFFFFFFFFFFF").unwrap();
    let key_bytes = hex::decode("00000000000000000000").unwrap();
    let key = present80::Key::new(&key_bytes[..]);

    let encrypted = present80::ecb_encrypt(&plaintext[..], key);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "5579C1387B228445A112FFC72F68417B";
    assert_eq!(expected, ciphertext);
}

#[test]
fn test_ecb_decrypt1() {
    let plaintext = hex::decode("5579C1387B228445A112FFC72F68417B").unwrap();
    let key_bytes = hex::decode("00000000000000000000").unwrap();
    let key = present80::Key::new(&key_bytes[..]);

    let encrypted = present80::ecb_decrypt(&plaintext[..], key);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "0000000000000000FFFFFFFFFFFFFFFF";
    assert_eq!(expected, ciphertext);
}

test_ecb!(
    test_ecb_encrypt2,
    test_ecb_decrypt2,
    "FFFFFFFFFFFFFFFFFFFF",
    "0000000000000000FFFFFFFFFFFFFFFF",
    "E72C46C0F59450493333DCD3213210D2"
);

test_par_ecb!(
    test_par_ecb_encrypt1,
    test_par_ecb_decrypt1,
    "00000000000000000000",
    "0000000000000000FFFFFFFFFFFFFFFF",
    "5579C1387B228445A112FFC72F68417B"
);

test_par_ecb!(
    test_par_ecb_encrypt2,
    test_par_ecb_decrypt2,
    "FFFFFFFFFFFFFFFFFFFF",
    "0000000000000000FFFFFFFFFFFFFFFF",
    "E72C46C0F59450493333DCD3213210D2"
);
