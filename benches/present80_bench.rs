#![feature(test)]

extern crate test;

extern crate hex;
extern crate present;

use test::Bencher;

use present::present80;

macro_rules! bench_encrypt_or_decrypt {
    ($name:ident, $f:ident, $n:expr, $k:expr, $i:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let input: Vec<u8> = hex::decode($i)
                .unwrap()
                .iter()
                .cycle()
                .take($n)
                .map(|&x| x)
                .collect();
            let key_bytes = hex::decode($k).unwrap();
            let key = present80::Key::new(&key_bytes[..]);

            b.iter(|| present80::$f(&input[..], key));
        }
    };
}

bench_encrypt_or_decrypt!(bench_ecb_encrypt_4kb, ecb_encrypt, 4096, "FFFFFFFFFFFFFFFFFFFF", "0000000000000000FFFFFFFFFFFFFFFF");

bench_encrypt_or_decrypt!(bench_par_ecb_encrypt_4kb, par_ecb_encrypt, 4096, "FFFFFFFFFFFFFFFFFFFF", "0000000000000000FFFFFFFFFFFFFFFF");

bench_encrypt_or_decrypt!(bench_ecb_encrypt_16kb, ecb_encrypt, 16384, "FFFFFFFFFFFFFFFFFFFF", "0000000000000000FFFFFFFFFFFFFFFF");

bench_encrypt_or_decrypt!(bench_par_ecb_encrypt_16kb, par_ecb_encrypt, 16384, "FFFFFFFFFFFFFFFFFFFF", "0000000000000000FFFFFFFFFFFFFFFF");
