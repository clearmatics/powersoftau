extern crate getopts;

use configuration::*;
use std::str::FromStr;
use std::env;

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
