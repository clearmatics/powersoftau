extern crate getopts;

use configuration::*;
use std::str::FromStr;
use std::env;

pub fn digest_from_string(s: &String) -> Result<[u8; 64], String>
{
    let mut out = [0u8;64];

    let mut idx = 0;
    for line in out.chunks_mut(16) {
        for word in line.chunks_mut(4) {
            word.copy_from_slice(hex::decode(&s[idx..idx+8]).unwrap().as_slice());
            idx = idx + 9;
        }
    }

    return Ok(out);
}

// fn digest_to_string(digest: &[u8; 64]) -> String {
pub fn digest_to_string(digest: &[u8]) -> String
{
    use std::fmt::Write;

    // 4 lines, each with 4 x 8-char hex string + whitespace)
    const SIZE : usize = 4 * (4 * 8 + 4);
    let mut out = String::with_capacity(SIZE);
    for line in digest.chunks(16) {
        let mut chunks = line.chunks(4);
        for b in chunks.next().unwrap() { out.write_fmt(format_args!("{:02x}", b)).unwrap(); }
        out.push(' ');
        for b in chunks.next().unwrap() { out.write_fmt(format_args!("{:02x}", b)).unwrap(); }
        out.push(' ');
        for b in chunks.next().unwrap() { out.write_fmt(format_args!("{:02x}", b)).unwrap(); }
        out.push(' ');
        for b in chunks.next().unwrap() { out.write_fmt(format_args!("{:02x}", b)).unwrap(); }
        out.push('\n');
    }

    return out;
}

pub fn match_or_fail(opts : &getopts::Options) -> getopts::Matches
{
    let mut args_iter = env::args();
    let program = args_iter.next();
    let args = args_iter.collect::<Vec<String>>();

    let matches = match opts.parse(args) {
        Ok(m) => { m }
        Err(e) => { panic!(e.to_string()) }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage:  {} [options]", program.unwrap());
        print!("{}", opts.usage(&brief));
        std::process::exit(0);
    }

    matches
}

pub fn get_opt<T>(matches: &getopts::Matches, name: &str) -> Option<T>
    where T: FromStr,
          T::Err: ToString
{
    match matches.opt_get::<T>(name) {
        Ok(v_opt) => { v_opt }
        Err(e) => { panic!("arg {}: {}", name, e.to_string()) }
    }
}

pub fn get_opt_default<T>(matches: &getopts::Matches, name: &str, default : T) -> T
    where T: FromStr,
          T::Err: ToString
{
    match matches.opt_get_default(name, default) {
        Ok(v) => { v }
        Err(e) => { panic!("arg {}: {}", name, e.to_string()) }
    }
}

/// Handle the common case of commands that only take '-n NUM_POWERS' option.
pub fn parse_simple_options() -> Configuration
{
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help");
    opts.optopt("n", "", "number of tau powers", "NUM_POWERS");
    let matches = match_or_fail(&opts);
    return Configuration::new(get_opt_default(&matches, "n", DEFAULT_NUM_POWERS));
}

#[test]
fn test_digest_strings()
{
    let s = concat!(
        "45252182 de43a613 24d87136 3535dfb8\n",
        "829a35e3 09a06fe1 7ef54b76 60ddc31d\n",
        "69511dd4 f0f474c1 475e9cc6 fcd0d261\n",
        "7501d8b3 617ecc47 8d0ace6a c735c83b\n");

    let digest = digest_from_string(&String::from_str(s).unwrap()).unwrap();
    let digest_string = digest_to_string(&digest[..]);

    assert_eq!(0x45u8, digest[0]);
    assert_eq!(0x3bu8, digest[63]);
    assert_eq!(s, digest_string);
}
