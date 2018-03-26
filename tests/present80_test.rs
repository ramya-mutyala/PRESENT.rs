extern crate hex;
extern crate present;

use present::present80;

#[test]
fn test_encrypt_block1() {
    let plaintext = hex::decode("0000000000000000").unwrap();
    let key_bytes = hex::decode("00000000000000000000").unwrap();
    let key = present80::Key::new(&key_bytes[..]);

    let encrypted = present80::encrypt_block(&plaintext[..], key);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "5579C1387B228445";
    assert_eq!(expected, ciphertext);
}

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
fn test_encrypt_block3() {
    let plaintext = hex::decode("FFFFFFFFFFFFFFFF").unwrap();
    let key_bytes = hex::decode("00000000000000000000").unwrap();
    let key = present80::Key::new(&key_bytes[..]);

    let encrypted = present80::ecb_encrypt(&plaintext[..], key);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "A112FFC72F68417B";
    assert_eq!(expected, ciphertext);
}

#[test]
fn test_encrypt_block4() {
    let plaintext = hex::decode("FFFFFFFFFFFFFFFF").unwrap();
    let key_bytes = hex::decode("FFFFFFFFFFFFFFFFFFFF").unwrap();
    let key = present80::Key::new(&key_bytes[..]);


    let encrypted = present80::encrypt_block(&plaintext[..], key);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "3333DCD3213210D2";
    assert_eq!(expected, ciphertext);
}

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
fn test_ecb_encrypt2() {
    let plaintext = hex::decode("0000000000000000FFFFFFFFFFFFFFFF").unwrap();
    let key_bytes = hex::decode("FFFFFFFFFFFFFFFFFFFF").unwrap();
    let key = present80::Key::new(&key_bytes[..]);


    let encrypted = present80::ecb_encrypt(&plaintext[..], key);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "E72C46C0F59450493333DCD3213210D2";
    assert_eq!(expected, ciphertext);
}
