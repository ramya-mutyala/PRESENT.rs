extern crate hex;
extern crate present;

#[test]
fn test_encrypt1() {
    let plaintext = hex::decode("0000000000000000").unwrap();
    let key = hex::decode("00000000000000000000").unwrap();

    let encrypted = present::encrypt(&plaintext[..], &key[..]);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "5579C1387B228445";
    assert_eq!(expected, ciphertext);
}

#[test]
fn test_encrypt2() {
    let plaintext = hex::decode("0000000000000000").unwrap();
    let key = hex::decode("FFFFFFFFFFFFFFFFFFFF").unwrap();

    let encrypted = present::encrypt(&plaintext[..], &key[..]);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "E72C46C0F5945049";
    assert_eq!(expected, ciphertext);
}

#[test]
fn test_encrypt3() {
    let plaintext = hex::decode("FFFFFFFFFFFFFFFF").unwrap();
    let key = hex::decode("00000000000000000000").unwrap();

    let encrypted = present::encrypt(&plaintext[..], &key[..]);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "A112FFC72F68417B";
    assert_eq!(expected, ciphertext);
}

#[test]
fn test_encrypt4() {
    let plaintext = hex::decode("FFFFFFFFFFFFFFFF").unwrap();
    let key = hex::decode("FFFFFFFFFFFFFFFFFFFF").unwrap();

    let encrypted = present::encrypt(&plaintext[..], &key[..]);
    let ciphertext = hex::encode_upper(&encrypted[..]);

    let expected = "3333DCD3213210D2";
    assert_eq!(expected, ciphertext);
}
