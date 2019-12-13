extern crate powersoftau;
extern crate rand;
extern crate blake2;
extern crate byteorder;
extern crate getopts;

use powersoftau::*;
use powersoftau::cmd_utils::*;
use std::fs::OpenOptions;
use std::io::{self, Read, BufReader, Write, BufWriter};

fn main() {
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help");
    opts.optopt("n", "", "number of tau powers", "NUM_POWERS");
    opts.optopt("d", "digest", "file to write digest to", "FILE");
    let matches = match_or_fail(&opts);

    let num_powers : usize =
        get_opt_default(&matches, "n", configuration::DEFAULT_NUM_POWERS);
    let config = configuration::Configuration::new(num_powers);
    let digest_file_opt : Option<String> = get_opt(&matches, "d");

    // Create an RNG based on a mixture of system randomness and user provided randomness
    let mut rng = {
        use byteorder::{ReadBytesExt, BigEndian};
        use blake2::{Blake2b, Digest};
        use rand::{SeedableRng, Rng, OsRng};
        use rand::chacha::ChaChaRng;

        let h = {
            let mut system_rng = OsRng::new().unwrap();
            let mut h = Blake2b::default();

            // Gather 1024 bytes of entropy from the system
            for _ in 0..1024 {
                let r: u8 = system_rng.gen();
                h.input(&[r]);
            }

            // Ask the user to provide some information for additional entropy
            let mut user_input = String::new();
            println!("Type some random text and press [ENTER] to provide additional entropy...");
            io::stdin().read_line(&mut user_input).expect("expected to read some random text from the user");

            // Hash it all up to make a seed
            h.input(&user_input.as_bytes());
            h.result()
        };

        let mut digest = &h[..];

        let mut seed : [u8;32] = [0;32];
        for i in 0..8 {
            let bytes = digest.read_u32::<BigEndian>().unwrap().to_be_bytes();
            seed[(4 * i) .. ((4 * i) + 4)].copy_from_slice(&bytes);
        }

        ChaChaRng::from_seed(seed)
    };

    // Try to load `./challenge` from disk.
    let reader = OpenOptions::new()
                            .read(true)
                            .open("challenge").expect("unable open `./challenge` in this directory");

    {
        let metadata = reader.metadata().expect("unable to get filesystem metadata for `./challenge`");
        if metadata.len() != (config.accumulator_size_bytes as u64) {
            panic!(
                "The size of `./challenge` should be {}, but it's {}, so something isn't right.",
                config.accumulator_size_bytes,
                metadata.len());
        }
    }

    let reader = BufReader::new(reader);
    let mut reader = HashReader::new(reader);

    // Create `./response` in this directory
    let writer = OpenOptions::new()
                            .read(false)
                            .write(true)
                            .create_new(true)
                            .open("response").expect("unable to create `./response` in this directory");

    let writer = BufWriter::new(writer);
    let mut writer = HashWriter::new(writer);

    println!("Reading `./challenge` into memory...");

    // Read the BLAKE2b hash of the previous contribution
    {
        // We don't need to do anything with it, but it's important for
        // the hash chain.
        let mut tmp = [0; 64];
        reader.read_exact(&mut tmp).expect("unable to read BLAKE2b hash of previous contribution");
    }

    // Load the current accumulator into memory
    let mut current_accumulator = Accumulator::deserialize(
        config,
        &mut reader,
        UseCompression::No,
        CheckForCorrectness::No)
        .expect("unable to read uncompressed accumulator");

    // Get the hash of the current accumulator
    let current_accumulator_hash = reader.into_hash();

    // Construct our keypair using the RNG we created above
    let (pubkey, privkey) = keypair(&mut rng, current_accumulator_hash.as_ref());

    // Perform the transformation
    println!("Computing, this could take a while...");
    current_accumulator.transform(&privkey);
    println!("Writing your contribution to `./response`...");

    // Write the hash of the input accumulator
    writer.write_all(&current_accumulator_hash.as_ref()).expect("unable to write BLAKE2b hash of input accumulator");

    // Write the transformed accumulator (in compressed form, to save upload bandwidth for disadvantaged
    // players.)
    current_accumulator.serialize(&mut writer, UseCompression::Yes).expect("unable to write transformed accumulator");

    // Write the public key
    pubkey.serialize(&mut writer).expect("unable to write public key");

    // Get the hash of the contribution, so the user can compare later
    let contribution_hash = writer.into_hash();

    print!("Done!\n\n\
              Your contribution has been written to `./response`\n\n\
              The BLAKE2b hash of `./response` is:\n");

    let hash_str = digest_to_string(contribution_hash.as_slice());
    print!("{}", hash_str);

    match digest_file_opt {
        Some(digest_file) => {
            let mut digest_writer = OpenOptions::new()
                .read(false)
                .write(true)
                .create(true)
                .open(&digest_file).expect("unable to create digest file");
            digest_writer.write(hash_str.as_bytes()).expect("digest write failed");
            println!("\nDigest written to `{}'", &digest_file);
        }
        None => {}
    }
}
