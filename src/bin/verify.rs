extern crate bn;
extern crate powersoftau;
extern crate rand;
extern crate blake2;
extern crate byteorder;

use powersoftau::*;
use powersoftau::cmd_utils::*;

use std::str;
use std::fs::OpenOptions;
use std::io::{self, BufReader, Write, Read};

fn into_hex(h: &[u8]) -> String {
    let mut f = String::new();

    for byte in &h[..] {
        f += &format!("{:02x}", byte);
    }

    f
}

// Computes the hash of the challenge file for the player,
// given the current state of the accumulator and the last
// response file hash.
fn get_challenge_file_hash(
    acc: &Accumulator,
    last_response_file_hash: &[u8; 64]
) -> [u8; 64]
{
    let sink = io::sink();
    let mut sink = HashWriter::new(sink);

    sink.write_all(last_response_file_hash)
        .unwrap();

    acc.serialize(
        &mut sink,
        UseCompression::No
    ).unwrap();

    let mut tmp = [0; 64];
    tmp.copy_from_slice(sink.into_hash().as_slice());

    tmp
}

// Computes the hash of the response file, given the new
// accumulator, the player's public key, and the challenge
// file's hash.
fn get_response_file_hash(
    acc: &Accumulator,
    pubkey: &PublicKey,
    last_challenge_file_hash: &[u8; 64]
) -> [u8; 64]
{
    let sink = io::sink();
    let mut sink = HashWriter::new(sink);

    sink.write_all(last_challenge_file_hash)
        .unwrap();

    acc.serialize(
        &mut sink,
        UseCompression::Yes
    ).unwrap();

    pubkey.serialize(&mut sink).unwrap();

    let mut tmp = [0; 64];
    tmp.copy_from_slice(sink.into_hash().as_slice());

    tmp
}

fn main() {
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help");
    opts.optopt("n", "", "number of tau powers", "NUM_POWERS");
    opts.optopt("r", "rounds", "number of rounds", "NUM_ROUNDS");
    opts.optopt("d", "digest", "check contribution with given digest", "FILE");
    opts.optflag("s", "skip-lagrange", "skip generation of phase1radix2m files");
    let matches = match_or_fail(&opts);

    let config = configuration::Configuration::new(
        get_opt_default(&matches, "n", configuration::DEFAULT_NUM_POWERS));
    // 89 hard-coded into original code
    let num_rounds = get_opt_default(&matches, "r", 89);
    let skip_lagrange = matches.opt_present("s");
    let digest_file_opt : Option<String> = get_opt(&matches, "d");
    let contrib_digest_opt : Option<[u8;DIGEST_LENGTH]> = digest_file_opt
        .as_ref()
        .map(|digest_file| {
            let mut digest_reader = OpenOptions::new()
                .read(true).open(digest_file).expect("failed to open digest file");
            let mut digest_buffer = [0u8; DIGEST_STRING_LENGTH];
            digest_reader.read_exact(&mut digest_buffer).expect("invalid digest file size");
            let digest_string = String::from(
                str::from_utf8(&digest_buffer).expect("invalid digest data"));
            let digest : [u8; DIGEST_LENGTH] =
                digest_from_string(&digest_string).expect("invalid digest file");
            digest
        });

    // Try to load `./transcript` from disk.
    let reader = OpenOptions::new()
                            .read(true)
                            .open("transcript")
                            .expect("unable open `./transcript` in this directory");

    let mut reader = BufReader::with_capacity(1024 * 1024, reader);

    // Initialize the accumulator
    let mut current_accumulator = Accumulator::new(config);

    // The "last response file hash" is just a blank BLAKE2b hash
    // at the beginning of the hash chain.
    let mut last_response_file_hash = [0; 64];
    last_response_file_hash.copy_from_slice(blank_hash().as_slice());

    // If a digest was specified, check the transcript to ensure it is
    // included.
    let mut found_digest : bool = contrib_digest_opt.is_none();

    for _ in 0..num_rounds {
        // Compute the hash of the challenge file that the player
        // should have received.
        let last_challenge_file_hash = get_challenge_file_hash(
            &current_accumulator,
            &last_response_file_hash
        );

        // Deserialize the accumulator provided by the player in
        // their response file. It's stored in the transcript in
        // uncompressed form so that we can more efficiently
        // deserialize it.
        let response_file_accumulator = Accumulator::deserialize(
            config,
            &mut reader,
            UseCompression::Yes,
            CheckForCorrectness::Yes
        ).expect("unable to read uncompressed accumulator");

        // Deserialize the public key provided by the player.
        let response_file_pubkey = PublicKey::deserialize(&mut reader)
            .expect("wasn't able to deserialize the response file's public key");

        // Compute the hash of the response file. (we had it in uncompressed
        // form in the transcript, but the response file is compressed to save
        // participants bandwidth.)
        last_response_file_hash = get_response_file_hash(
            &response_file_accumulator,
            &response_file_pubkey,
            &last_challenge_file_hash
        );

        if !found_digest {
            found_digest = digest_equal(
                &last_response_file_hash, &contrib_digest_opt.expect(""));
        }

        print!("{}", into_hex(&last_response_file_hash));

        // Verify the transformation from the previous accumulator to the new
        // one. This also verifies the correctness of the accumulators and the
        // public keys, with respect to the transcript so far.
        if !verify_transform(
            &current_accumulator,
            &response_file_accumulator,
            &response_file_pubkey,
            &last_challenge_file_hash
        )
        {
            println!(" ... FAILED");
            panic!("INVALID RESPONSE FILE!");
        } else {
            println!("");
        }

        current_accumulator = response_file_accumulator;
    }

    println!("Transcript OK!");

    if !found_digest {
        println!("Digest not found!");
        std::process::exit(1);
    }

    if skip_lagrange {
        println!("WARNING: --skip-lagrange flag unused");
    }
}
