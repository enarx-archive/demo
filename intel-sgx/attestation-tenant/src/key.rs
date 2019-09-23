use openssl::{
    bn::BigNum,
    derive::Deriver,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private, Public},
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

/// This Key is a wrapper for an openssl::PKey<Public> and openssl::EcKey<Private> key pair
/// with extra functionality, ex. the PKey can be created from raw x and y coordinates and verify
/// a signature and SHA256 hash. The curve for all keys is SECP256R1 (known as PRIME256V1).
pub struct Key {
    curve: EcGroup,
    pubkey: PKey<Public>,
    privkey: Option<EcKey<Private>>,
}

impl Key {
    /// This creates a new public PKey from raw x and y coordinates for the SECP256R1 curve.
    /// The private key is not known or needed.
    pub fn new_from_xy(xy_coords: &[u8]) -> Result<Self, Box<dyn Error>> {
        // TODO: Is it possible to give the Key a reference to this curve without instantiating it in each
        // Key instance? Rust doesn't do runtime-generated global variables.
        let curve = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        let mut x: [u8; 32] = Default::default();
        let mut y: [u8; 32] = Default::default();
        x.copy_from_slice(&xy_coords[0..32]);
        y.copy_from_slice(&xy_coords[32..64]);
        let xbn = BigNum::from_slice(&x)?;
        let ybn = BigNum::from_slice(&y)?;
        let ec_key = EcKey::from_public_key_affine_coordinates(&curve, &xbn, &ybn)?;
        let pkey = PKey::from_ec_key(ec_key)?;

        Ok(Key {
            curve: curve,
            pubkey: pkey,
            privkey: None,
        })
    }

    /// This creates a new public PKey from bytes. This can reconstruct a public key sent
    /// from an enclave, which uses mbedtls rather than openssl.
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut ctx = openssl::bn::BigNumContext::new()?;
        let curve = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        let pub_ecpoint = openssl::ec::EcPoint::from_bytes(curve.as_ref(), &bytes, &mut *ctx)?;
        let pub_eckey = openssl::ec::EcKey::from_public_key(curve.as_ref(), pub_ecpoint.as_ref())?;
        let pub_pkey = openssl::pkey::PKey::from_ec_key(pub_eckey)?;

        Ok(Key {
            curve: curve,
            pubkey: pub_pkey,
            privkey: None,
        })
    }

    /// This creates a new Key from existing PKey value.
    pub fn new_from_pubkey(pkey: PKey<Public>) -> Self {
        Key {
            curve: EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap(),
            pubkey: pkey,
            privkey: None,
        }
    }

    /// This creates a new elliptic curve key pair for the SECP256R1 curve with no other inputs.
    /// These are then converted to PKeys, which can be used for a DH key exchange according to
    /// https://github.com/sfackler/rust-openssl/blob/master/openssl/src/pkey.rs#L16. The EcKey type
    /// as the private key allows the public key to be returned as bytes in return_pubkey_bytes().
    // TODO: Is this a good curve to use for ECDH keys?
    pub fn new_pair_secp256r1() -> Result<Self, Box<dyn Error>> {
        let curve = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
        let eckey_priv = EcKey::generate(&curve)?;
        eckey_priv.check_key()?;
        let eckey_pub = EcKey::from_public_key(&curve, eckey_priv.as_ref().public_key())?;
        let pkey_pub = PKey::from_ec_key(eckey_pub)?;
        Ok(Key {
            curve: curve,
            pubkey: pkey_pub,
            privkey: Some(eckey_priv),
        })
    }

    /// Returns the Key's public key as a PKey<Public>
    pub fn return_pubkey(&self) -> &PKey<Public> {
        &self.pubkey
    }

    /// Returns the Key's public key as bytes. This is useful for transmitting to the enclave, which
    /// can reconstruct the key with mbedtls.
    pub fn return_pubkey_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut new_ctx = openssl::bn::BigNumContext::new()?;
        let priv_key = self.privkey.as_ref().unwrap();

        let tenant_pubkey_bytes = priv_key.public_key().to_bytes(
            &self.curve,
            openssl::ec::PointConversionForm::UNCOMPRESSED,
            &mut *new_ctx,
        )?;

        Ok(tenant_pubkey_bytes)
    }

    /// DHKE deriving shared secret between self's private key and peer's public key.
    pub fn derive_shared_secret(&self, peer_key: &PKey<Public>) -> Result<Vec<u8>, Box<dyn Error>> {
        let ec_priv_key = self.privkey.as_ref().unwrap();
        let pkey_priv_key = PKey::from_ec_key(ec_priv_key.clone())?;
        let mut deriver = Deriver::new(pkey_priv_key.as_ref())?;
        deriver.set_peer(peer_key)?;
        Ok(deriver.derive_to_vec()?)
    }

    /// Given a signature and material that was signed with the Key's PKey value, this
    /// verifies the given signature.
    pub fn verify_sig(&self, signed: &[u8], sig: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), &self.pubkey)?;
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
