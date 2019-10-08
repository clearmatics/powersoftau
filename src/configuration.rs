extern crate getopts;

// This ceremony is based on the BN256 elliptic curve construction.
const G1_UNCOMPRESSED_BYTE_SIZE: usize = 1 + 32 + 32;
const G2_UNCOMPRESSED_BYTE_SIZE: usize = 1 + 64 + 64;
const G1_COMPRESSED_BYTE_SIZE: usize = 1 + 32;
const G2_COMPRESSED_BYTE_SIZE: usize = 1 + 64;

// Default value for num_powers
pub const DEFAULT_NUM_POWERS: usize = (1 << 21);

/// The "public key" is used to verify a contribution was correctly
/// computed.
pub const PUBLIC_KEY_SIZE: usize =
    3 * G2_UNCOMPRESSED_BYTE_SIZE + // tau, alpha, and beta in g2
    6 * G1_UNCOMPRESSED_BYTE_SIZE; // (s1, s1*tau), (s2, s2*alpha), (s3, s3*beta) in g1

fn is_pow2(v: usize) -> bool {
    0 == (v & (v-1))
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Configuration {
    /// The maximum number of gates to be supported by circuits
    pub num_powers: usize,

    /// More tau powers are needed in G1 because the Groth16 H query
    /// includes terms of the form tau^i * (tau^m - 1) = tau^(i+m) - tau^i
    /// where the largest i = m - 2, requiring the computation of tau^(2m - 2)
    /// and thus giving us a vector length of 2^22 - 1.
    pub num_powers_g1: usize,

    /// The size of the accumulator on disk.
    pub accumulator_size_bytes: usize,

    /// The size of the contribution on disk.
    pub contribution_size_bytes: usize,
}

impl Configuration {
    pub fn new(num_powers: usize) -> Self
    {
        assert!(is_pow2(num_powers));
        let num_powers_g1 = (num_powers << 1) - 1;
        let accumulator_size =
            (num_powers_g1 * G1_UNCOMPRESSED_BYTE_SIZE) + // g1 tau powers
            (num_powers * G2_UNCOMPRESSED_BYTE_SIZE) + // g2 tau powers
            (num_powers * G1_UNCOMPRESSED_BYTE_SIZE) + // alpha tau powers
            (num_powers * G1_UNCOMPRESSED_BYTE_SIZE) // beta tau powers
            + G2_UNCOMPRESSED_BYTE_SIZE // beta in g2
            + 64; // blake2b hash of previous contribution
        let contribution_size =
            (num_powers_g1 * G1_COMPRESSED_BYTE_SIZE) + // g1 tau powers
            (num_powers * G2_COMPRESSED_BYTE_SIZE) + // g2 tau powers
            (num_powers * G1_COMPRESSED_BYTE_SIZE) + // alpha tau powers
            (num_powers * G1_COMPRESSED_BYTE_SIZE) // beta tau powers
            + G2_COMPRESSED_BYTE_SIZE // beta in g2
            + 64 // blake2b hash of input accumulator
            + PUBLIC_KEY_SIZE; // public key
        Configuration {
            num_powers: num_powers,
            num_powers_g1: num_powers_g1,
            accumulator_size_bytes: accumulator_size,
            contribution_size_bytes: contribution_size
        }
    }

    pub fn default() -> Self
    {
        Self::new(DEFAULT_NUM_POWERS)
    }
}
