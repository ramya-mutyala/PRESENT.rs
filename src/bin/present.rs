extern crate clap;
extern crate hex;

use std::io;
use std::fs::File;

use clap::{App, Arg, ArgGroup};

fn main() {
    let matches = App::new("PRESENT.rs")
        .version("0.1.0")
        .author("Jiayu Yi")
        .about("Encrypt/decrypt data with the PRESENT block cipher and print to standard output")
        .arg(Arg::with_name("key")
            .short("k")
            .long("key")
            .value_name("key")
            .takes_value(true)
            .help("Hex or base64 encoded encryption key"))
        .arg(Arg::with_name("key file")
            .short("K")
            .long("key-file")
            .value_name("key_file")
            .help("Read encryption key from a file")
            .takes_value(true))
        .group(ArgGroup::with_name("key source")
            .args(&["key", "key file"])
            .required(true))
        .arg(Arg::with_name("input format")
            .short("I")
            .long("input-format")
            .possible_values(&["binary", "base64", "hex"])
            .default_value("hex")
            .help("Specify input format"))
        .arg(Arg::with_name("output format")
            .short("O")
            .long("output-format")
            .possible_values(&["binary", "base64", "hex"])
            .default_value("hex")
            .help("Specify output format"))
        .arg(Arg::with_name("FILE")
            .help("Encrypt the contents of FILE and print to standard output. When no FILE, or when FILE is -, read standard input.")
            .index(1))
        .get_matches();

    let mut data: Box<io::Read> = match matches.value_of("DATA") {
        None => Box::new(io::stdin()),
        Some(filename) => match filename {
            "-" => Box::new(io::stdin()),
            _ => Box::new(File::open(filename).expect("file not found!"))
        }
    };

    let mut buf = [0; 8];
    data.read(&mut buf).expect("read error!");
    println!("{:?}", &buf[..]);
}
