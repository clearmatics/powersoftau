extern crate powersoftau;
extern crate rand;
extern crate rustc_serialize;
extern crate bincode;
extern crate hex;
extern crate bn;
extern crate getopts;

use rand::{SeedableRng, Rng};
use rand::chacha::ChaChaRng;

use bn::*;
use std::ops::*;
use std::string::String;
use std::env;
use rustc_serialize::{Encodable};
use getopts::Options;

fn to_hex<E : Encodable>(e : E) -> String
{
    let bin = bincode::encode(&e, bincode::SizeLimit::Infinite)
        .unwrap();
    hex::encode(bin)
}

fn test_serialization()
{
    // Write out some test values to help with binary compatibility
    // with other libraries.

    let f_1 = bn::Fr::one();
    let f_2 = f_1 + f_1;
    let f_4 = f_2 + f_2;
    let f_7 = f_4 + f_2 + f_1;
    let f_7_inv = f_7.inverse().unwrap();
    let g1_7_inv = bn::G1::one().mul(f_7_inv);
    let g2_7_inv = bn::G2::one().mul(f_7_inv);

    println!("fr_0_bin='{}'", to_hex(&bn::Fr::zero()));
    println!("fr_1_bin='{}'", to_hex(&f_1));
    println!("fr_7_inv_bin='{}'", to_hex(&f_7_inv));

    println!("g1_0_bin='{}'", to_hex(&G1::zero()));
    println!("g1_1_bin='{}'", to_hex(&G1::one()));
    println!("g1_7_inv_bin='{}'", to_hex(&g1_7_inv));

    println!("g2_0_bin='{}'", to_hex(&G2::zero()));
    println!("g2_1_bin='{}'", to_hex(&G2::one()));
    println!("g2_7_inv_bin='{}'", to_hex(&g2_7_inv));
}

fn test_chacha()
{
    let mut seed : [u8;32] = [0; 32];
    seed.copy_from_slice(
        &hex::decode(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .unwrap());
    println!("seed: {}", hex::encode(&seed));

    {
        let mut rng : ChaChaRng = ChaChaRng::from_seed(seed);

        let mut output : [u8; 64] = [0; 64];

        rng.fill(& mut output);
        println!("output: {}", hex::encode(&output[..]));

        rng.fill(& mut output);
        println!("output: {}", hex::encode(&output[..]));
    }

    {
        let r_fr = arith::U512::random(&mut ChaChaRng::from_seed(seed));
        let rem = r_fr.divrem(&fields::Fr::modulus());
        println!("r_fr='{}'", to_hex(&r_fr));
        println!("rem='{}'", to_hex(&rem));
    }

    {
        let r_fr = Fr::random(&mut ChaChaRng::from_seed(seed));
        println!("r_fr='{}'", to_hex(&r_fr));
    }

    {
        let r_g2 = G2::random(&mut ChaChaRng::from_seed(seed));
        println!("r_g2='{}'", to_hex(&r_g2));
    }
}

fn test_operators()
{
    let v : usize = 1 << 21;
    let y = !(v-1);
    println!("v = {:X}, y = {:X}, v & y = {:X}", v, y, v & y);
}

fn test_getopts()
{
    let args : Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("n", "", "number of tau powers", "NUM_POWERS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(e) => { panic!(e.to_string()) }
    };

    if matches.opt_present("n") {
        let n : usize = matches.opt_get::<usize>("n").unwrap().unwrap();
        println!("GOT ARG: n = {:#X} ({})", n, n);
    }
}

fn main()
{
    test_getopts();
    test_operators();
    test_serialization();
    test_chacha();
}
