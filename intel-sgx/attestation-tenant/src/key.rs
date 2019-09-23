use openssl::{
    bn::BigNum,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Public},
    sha,
    sign::Verifier,
};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// This is the error returned when the PCK hash is not valid.
#[derive(Debug, Clone)]
pub struct HashError;

impl Error for HashError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl Display for HashError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "PCK hash could not be validated")
    }
}

/// This is a wrapper for an openssl::PKey<Public> value that adds methods to create
/// the key from raw x and y coordinates and verify a signature and SHA256 hash.
pub struct Key {
    pkey: PKey<Public>,
}

impl Key {
    /// This creates a new Key from raw x and y coordinates for the SECP256R1 curve.
    pub fn new_from_xy(xy_coords: &[u8]) -> Result<Self, Box<dyn Error>> {
        let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        let mut x: [u8; 32] = Default::default();
        let mut y: [u8; 32] = Default::default();
        x.copy_from_slice(&xy_coords[0..32]);
        y.copy_from_slice(&xy_coords[32..64]);
        let xbn = BigNum::from_slice(&x)?;
        let ybn = BigNum::from_slice(&y)?;
        let ec_key = EcKey::from_public_key_affine_coordinates(&group, &xbn, &ybn)?;
        let pkey = PKey::from_ec_key(ec_key)?;

        Ok(Key { pkey: pkey })
    }

    /// This creates a new Key from existing PKey value.
    pub fn new_from_pubkey(pkey: PKey<Public>) -> Self {
        Key { pkey: pkey }
    }

    /// Given a signature and material that was signed with the Key's PKey value, this
    /// verifies the given signature.
    pub fn verify_sig(&self, signed: &[u8], sig: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), &self.pkey)?;
        verifier.update(signed)?;
        verifier.verify(sig)?;
        Ok(())
    }

    /// This is meant to verify the SHA-256 hash of the Attestation Public Key || QEAuthData
    /// (embedded in Quote, signed by PCK).
    // TODO: I don't like that this method doesn't use the pubkey's value, but I attached it to
    // the Key struct because that's where it makes the most sense conceptually.
    pub fn verify_hash(&self, hashed_data: &[u8], unhashed_data: Vec<u8>) -> Result<(), HashError> {
        let mut hasher = sha::Sha256::new();
        hasher.update(&unhashed_data);
        let hash = hasher.finish();
        if hash != hashed_data {
            Err(HashError)
        } else {
            Ok(())
        }
    }
}
