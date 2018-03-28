extern crate clap;
extern crate hex;
extern crate present;

use std::io::{self, BufReader, BufWriter, Read, Write};
use std::fs::File;

use clap::{App, Arg, ArgGroup};
use present::{present128, present80};

const MAX_KEY_LENGTH_HEX: usize = 32;
const MAX_KEY_LENGTH_BINARY: usize = 16;

const BLOCK_SIZE_HEX: usize = 16;
const BLOCK_SIZE_BINARY: usize = 8;

enum Format {
    Binary,
    Hex,
}

enum KeyLength {
    Key80,
    Key128,
    Auto,
}

enum InputSource {
    Stdin,
    File(String),
}

fn main() {
    let matches = App::new("PRESENT.rs")
        .version("0.1.0")
        .author("Jiayu Yi")
        .about("Encrypt or decrypt data with the PRESENT block cipher and print to standard output")
        .arg(Arg::with_name("key")
            .short("k")
            .long("key")
            .value_name("key")
            .takes_value(true)
            .help("Hex encoded encryption key"))
        .arg(Arg::with_name("key file")
            .short("K")
            .long("key-file")
            .value_name("key_file")
            .help("Read encryption key from a file")
            .takes_value(true))
        .group(ArgGroup::with_name("key source")
            .args(&["key", "key file"])
            .required(true))
        .arg(Arg::with_name("key length")
            .short("l")
            .long("key-length")
            .value_name("key_length")
            .help("Specify whether to use an 80-bit or 128-bit key")
            .possible_values(&["auto", "80", "128"])
            .default_value("auto"))
        .arg(Arg::with_name("input format")
            .short("I")
            .long("input-format")
            .possible_values(&["binary", "hex"])
            .default_value("binary")
            .help("Specify input format"))
        .arg(Arg::with_name("key format")
            .short("f")
            .long("key-format")
            .possible_values(&["binary", "hex"])
            .default_value("binary")
            .help("Specify key format from file"))
        .arg(Arg::with_name("output format")
            .short("O")
            .long("output-format")
            .possible_values(&["binary", "hex"])
            .default_value("binary")
            .help("Specify output format"))
        .arg(Arg::with_name("decrypt")
            .short("d")
            .long("decrypt")
            .help("Decrypt data"))
        .arg(Arg::with_name("FILE")
            .help("Encrypt or decrypt the contents of FILE and print to standard output. When no FILE, or when FILE is -, read standard input.")
            .index(1))
        .get_matches();

    let input_format = match matches.value_of("input format").unwrap() {
        "binary" => Format::Binary,
        "hex" => Format::Hex,
        _ => unreachable!(),
    };

    let key_length = match matches.value_of("key length").unwrap() {
        "auto" => KeyLength::Auto,
        "80" => KeyLength::Key80,
        "128" => KeyLength::Key128,
        _ => unreachable!(),
    };

    let output_format = match matches.value_of("output format").unwrap() {
        "binary" => Format::Binary,
        "hex" => Format::Hex,
        _ => unreachable!(),
    };

    let decrypt_mode = matches.is_present("decrypt");

    let key_bytes = match matches.value_of("key") {
        None => {
            let filename = matches.value_of("key file").unwrap();
            let key_format = match matches.value_of("key format").unwrap() {
                "binary" => Format::Binary,
                "hex" => Format::Hex,
                _ => unreachable!(),
            };

            read_key_bytes_from_file(filename, &key_format)
        }
        Some(key_string) => read_key_bytes_from_string(key_string, &Format::Hex),
    };

    let input_source = match matches.value_of("FILE") {
        None => InputSource::Stdin,
        Some(filename) => match filename {
            "-" => InputSource::Stdin,
            _ => InputSource::File(filename.to_string()),
        },
    };

    match input_source {
        InputSource::Stdin => {
            let stdin = io::stdin();
            let mut file = stdin.lock();

            if decrypt_mode {
                decrypt(
                    &mut file,
                    key_length,
                    key_bytes,
                    &input_format,
                    &output_format,
                );
            } else {
                encrypt(
                    &mut file,
                    key_length,
                    key_bytes,
                    &input_format,
                    &output_format,
                );
            }
        }
        InputSource::File(filename) => {
            let mut file = io::BufReader::new(File::open(filename).expect("file not found"));

            if decrypt_mode {
                decrypt(
                    &mut file,
                    key_length,
                    key_bytes,
                    &input_format,
                    &output_format,
                );
            } else {
                encrypt(
                    &mut file,
                    key_length,
                    key_bytes,
                    &input_format,
                    &output_format,
                );
            }
        }
    }
}

fn encrypt<R: io::BufRead>(
    file: &mut R,
    key_length: KeyLength,
    key_bytes: Vec<u8>,
    input_format: &Format,
    output_format: &Format,
) {
    match key_length {
        KeyLength::Key80 => {
            if key_bytes.len() < present80::KEY_LENGTH_IN_BYTES {
                eprintln!("warning: provided key contains less than 80 bytes and will be padded with zeroes")
            } else if key_bytes.len() > present80::KEY_LENGTH_IN_BYTES {
                eprintln!("warning: provided key contains more than 80 bytes and will be truncated")
            }

            let key = present80::Key::new(&key_bytes[..]);
            present80_encrypt(file, key, input_format, output_format);
        }
        KeyLength::Key128 => {
            if key_bytes.len() < present128::KEY_LENGTH_IN_BYTES {
                eprintln!("warning: provided key contains less than 128 bytes and will be padded with zeroes");
            } else if key_bytes.len() > present128::KEY_LENGTH_IN_BYTES {
                eprintln!(
                    "warning: provided key contains more than 128 bytes and will be truncated"
                );
            }

            let key = present128::Key::new(&key_bytes[..]);
            unimplemented!();
        }
        KeyLength::Auto => if key_bytes.len() <= 80 {
            encrypt(
                file,
                KeyLength::Key80,
                key_bytes,
                input_format,
                output_format,
            );
        } else {
            encrypt(
                file,
                KeyLength::Key128,
                key_bytes,
                input_format,
                output_format,
            );
        },
    }
}

fn decrypt<R: io::BufRead>(
    file: &mut R,
    key_length: KeyLength,
    key_bytes: Vec<u8>,
    input_format: &Format,
    output_format: &Format,
) {
    match key_length {
        KeyLength::Key80 => {
            if key_bytes.len() < present80::KEY_LENGTH_IN_BYTES {
                eprintln!("warning: provided key contains less than 80 bytes and will be padded with zeroes")
            } else if key_bytes.len() > present80::KEY_LENGTH_IN_BYTES {
                eprintln!("warning: provided key contains more than 80 bytes and will be truncated")
            }

            let key = present80::Key::new(&key_bytes[..]);
            present80_decrypt(file, key, input_format, output_format);
        }
        KeyLength::Key128 => {
            if key_bytes.len() < present128::KEY_LENGTH_IN_BYTES {
                eprintln!("warning: provided key contains less than 128 bytes and will be padded with zeroes");
            } else if key_bytes.len() > present128::KEY_LENGTH_IN_BYTES {
                eprintln!(
                    "warning: provided key contains more than 128 bytes and will be truncated"
                );
            }

            let key = present128::Key::new(&key_bytes[..]);
            unimplemented!();
        }
        KeyLength::Auto => if key_bytes.len() <= 80 {
            decrypt(
                file,
                KeyLength::Key80,
                key_bytes,
                input_format,
                output_format,
            );
        } else {
            decrypt(
                file,
                KeyLength::Key128,
                key_bytes,
                input_format,
                output_format,
            );
        },
    }
}

fn read_key_bytes_from_file(filename: &str, format: &Format) -> Vec<u8> {
    let mut file = File::open(filename).expect("file not found!");
    match format {
        &Format::Binary => {
            let mut buf = [0u8; MAX_KEY_LENGTH_BINARY];
            let bytes_read = file.read(&mut buf[..]).expect("error reading file");

            let mut key_bytes: Vec<u8> = Vec::with_capacity(bytes_read);
            key_bytes.extend_from_slice(&buf[..bytes_read]);

            key_bytes
        }
        &Format::Hex => {
            let mut buf = [0u8; MAX_KEY_LENGTH_HEX];
            let bytes_read = file.read(&mut buf[..]).expect("error reading file");

            let key_bytes = hex::decode(&buf[..bytes_read]).expect("error converting from hex");

            key_bytes
        }
    }
}

fn read_key_bytes_from_string(s: &str, format: &Format) -> Vec<u8> {
    match format {
        &Format::Binary => {
            let mut key_bytes: Vec<u8> = Vec::with_capacity(s.len());
            key_bytes.extend_from_slice(s.as_bytes());

            key_bytes
        }
        &Format::Hex => hex::decode(s).expect("error converting from hex"),
    }
}

fn read_block_from_file<R: io::BufRead>(file: &mut R, input_format: &Format) -> Option<Vec<u8>> {
    match input_format {
        &Format::Binary => {
            let mut buf = [0u8; BLOCK_SIZE_BINARY];
            let bytes_read = file.read(&mut buf[..]).expect("error reading file");

            if bytes_read == 0 {
                return None;
            }

            let mut block: Vec<u8> = Vec::with_capacity(bytes_read);
            block.extend_from_slice(&buf[..bytes_read]);

            Some(block)
        }
        &Format::Hex => {
            let mut buf = [0u8; BLOCK_SIZE_HEX];
            let bytes_read = file.read(&mut buf[..]).expect("error reading file");

            if bytes_read == 0 {
                return None;
            }

            let block = hex::decode(&buf[..bytes_read]).expect("error converting from hex");

            Some(block)
        }
    }
}

fn present80_encrypt<R: io::BufRead>(
    file: &mut R,
    key: present80::Key,
    input_format: &Format,
    output_format: &Format,
) {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    loop {
        let block = match read_block_from_file(file, input_format) {
            None => return,
            Some(block) => block,
        };
        let encrypted = present80::encrypt_block(&block[..], key);
        match output_format {
            &Format::Binary => out.write_all(&encrypted[..]),
            &Format::Hex => out.write_all(hex::encode(&encrypted[..]).as_bytes()),
        }.expect("error writing to stdout");
    }
}

fn present80_decrypt<R: io::BufRead>(
    file: &mut R,
    key: present80::Key,
    input_format: &Format,
    output_format: &Format,
) {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    loop {
        let block = match read_block_from_file(file, input_format) {
            None => return,
            Some(block) => block,
        };
        let decrypted = present80::decrypt_block(&block[..], key);
        match output_format {
            &Format::Binary => out.write_all(&decrypted[..]),
            &Format::Hex => out.write_all(hex::encode(&decrypted[..]).as_bytes()),
        }.expect("error writing to stdout");
    }
}
