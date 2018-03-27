#![feature(test)]

extern crate test;

extern crate hex;
extern crate present;

use test::Bencher;

use present::present80;

#[bench]
fn bench_ecb_encrypt(b: &mut Bencher) {
    let plaintext: Vec<u8> = hex::decode("0000000000000000FFFFFFFFFFFFFFFF")
        .unwrap()
        .iter()
        .cycle()
        .take(16384)
        .map(|&x| x)
        .collect();
    let key_bytes = hex::decode("FFFFFFFFFFFFFFFFFFFF").unwrap();
    let key = present80::Key::new(&key_bytes[..]);

    b.iter(|| present80::ecb_encrypt(&plaintext[..], key));
}

#[bench]
fn bench_par_ecb_encrypt(b: &mut Bencher) {
    let plaintext: Vec<u8> = hex::decode("0000000000000000FFFFFFFFFFFFFFFF")
        .unwrap()
        .iter()
        .cycle()
        .take(16384)
        .map(|&x| x)
        .collect();
    let key_bytes = hex::decode("FFFFFFFFFFFFFFFFFFFF").unwrap();
    let key = present80::Key::new(&key_bytes[..]);

    b.iter(|| present80::par_ecb_encrypt(&plaintext[..], key));
}
