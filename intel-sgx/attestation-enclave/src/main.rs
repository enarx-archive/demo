use byteorder::{ByteOrder, NativeEndian, ReadBytesExt};
use mbedtls::{
    cipher::*,
    ecp::{EcGroup, EcPoint},
    hash::{Md, Type::Sha256},
    pk::{EcGroupId, Pk},
    rng::{CtrDrbg, Random, Rdseed},
};
use serde_json::{from_reader, to_writer, Deserializer};
use sgx_isa::Report;
use std::{error::Error, io::Cursor, net::TcpListener};

const DAEMON_LISTENER_ADDR: &'static str = "localhost:1032";
const TENANT_LISTENER_ADDR: &'static str = "localhost:1066";

// This copies the enclave key to the report data
fn from_slice(bytes: &[u8]) -> [u8; 64] {
    let mut array = [0; 64];
    let bytes = &bytes[..array.len()]; // panics if not enough data
    array.copy_from_slice(bytes);
    array
}

// Creates an AES 256 GCM cipher instance with the symmetric key and initialization vector
// set for each decryption operation.
fn new_aes256gcm_decrypt_cipher(
    symm_key: &[u8],
    iv: &[u8],
) -> Result<Cipher<Decryption, Authenticated, AdditionalData>, Box<dyn Error>> {
    let c = Cipher::<_, Authenticated, _>::new(
        raw::CipherId::Aes,
        raw::CipherMode::GCM,
        (symm_key.len() * 8) as _,
    )?;

    Ok(c.set_key_iv(&symm_key, &iv)?)
}

// Creates an AES 256 GCM cipher instance with the symmetric key and initialization vector
// set for each encryption operation.
// TODO: This is redundant, but I can't return a Cipher<_, Authenticated, AdditionalData>, so I need two separate
// functions. How to fix?
fn new_aes256gcm_encrypt_cipher(
    symm_key: &[u8],
    iv: &[u8],
) -> Result<Cipher<Encryption, Authenticated, AdditionalData>, Box<dyn Error>> {
    let c = Cipher::<_, Authenticated, _>::new(
        raw::CipherId::Aes,
        raw::CipherMode::GCM,
        (symm_key.len() * 8) as _,
    )?;

    Ok(c.set_key_iv(&symm_key, &iv)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "\nListening on {} and {}....\n",
        DAEMON_LISTENER_ADDR, TENANT_LISTENER_ADDR
    );

    let daemon_streams = TcpListener::bind(DAEMON_LISTENER_ADDR)?;
    let tenant_streams = TcpListener::bind(TENANT_LISTENER_ADDR)?;

    let curve = EcGroup::new(EcGroupId::SecP256R1)?;

    // The enclave generates an EC key pair. The public key will be inserted into the ReportData
    // field of the enclave's attestation Report, which will be transmitted to the tenant.
    let mut entropy = Rdseed;
    let mut rng = CtrDrbg::new(&mut entropy, None)?;
    let mut ec_key = Pk::generate_ec(&mut rng, curve.clone())?;
    if !Pk::check_pair(&ec_key, &ec_key) {
        panic!("Error generating EC key")
    };
    let ec_pub = ec_key.ec_public()?;

    // The enclave handles incoming connections from attestation daemon.
    //let mut stream = daemon_streams
    //  .incoming()
    //  .next()
    //  .unwrap_or(Err(std::io::ErrorKind::ConnectionReset.into()))?;

    for stream in daemon_streams.incoming() {
        let mut stream = stream?;

        // The enclave receives the identity of the Quoting Enclave from the
        // attestation daemon, in the form of a serialized TargetInfo
        // structure. The TargetInfo contains the measurement and attribute flags
        // of the Quoting Enclave.
        let qe_id: sgx_isa::Targetinfo = from_reader(&mut stream)?;

        // The enclave's public key will be transmitted to the tenant in the ReportData field
        // of the enclave's attesation Report. It must be a &[u8; 64].
        // The compressed public key is 33 bytes long and must be extended by 31 bytes.
        let mut report_data = ec_pub.to_binary(&curve, true)?;
        report_data.extend(&[0u8; 31]);
        let report_data = from_slice(&report_data);

        // The enclave creates a Report attesting its identity, with the Quoting
        // Enclave (whose identity was just received) as the Report's target. The
        // ReportData field contains the enclave's public key.
        let report = Report::for_target(&qe_id, &report_data);

        // The enclave sends its attestation Report back to the attestation daemon.
        to_writer(&mut stream, &report)?;

        println!("Successfully sent report to daemon.");

        break;
    }

    // The enclave handles each incoming connection from the tenant. These channels between the tenant
    // and enclave are established after attestation is verified and all data exchanged between the tenant
    // and enclave after public keys are exchanged is encrypted with a shared symmetric key.
    for stream in tenant_streams.incoming() {
        let mut stream = stream?;

        // The enclave receives and deserializes tenant pub key, ivs and tags for ciphertext values, ciphertext.
        let deserializer = Deserializer::from_reader(stream.try_clone().unwrap());
        let mut iterator = deserializer.into_iter::<Vec<u8>>();

        let tenant_key = iterator.next().unwrap()?;
        let hash_v1 = iterator.next().unwrap()?;
        let hash_v2 = iterator.next().unwrap()?;
        let iv1 = iterator.next().unwrap()?;
        let iv2 = iterator.next().unwrap()?;
        let tag1 = iterator.next().unwrap()?;
        let tag2 = iterator.next().unwrap()?;
        let ciphertext1 = iterator.next().unwrap()?;
        let ciphertext2 = iterator.next().unwrap()?;

        // The enclave generates a shared secret with the tenant. A SHA256 hash of this shared secret
        // is used as the symmetric key for encryption and decryption of data.
        let tenant_pubkey_ecpoint = EcPoint::from_binary(&curve, &tenant_key)?;
        let tenant_pubkey = Pk::public_from_ec_components(curve.clone(), tenant_pubkey_ecpoint)?;

        // TODO: Should this use the same rng as before or create a new one?
        let mut shared_secret = [0u8; 32]; // 256 / 8
        ec_key.agree(&tenant_pubkey, &mut shared_secret, &mut rng)?;
        let mut symm_key = [0u8; 32];
        Md::hash(Sha256, &shared_secret, &mut symm_key)?;

        // These cipher instances are used for decryption operations and one encryption operation.
        // TODO: Can the same cipher instance be used for these? Cipher doesn't implement clone().
        let decrypt_cipher_1 = new_aes256gcm_decrypt_cipher(&symm_key, &iv1)?;
        let decrypt_cipher_2 = new_aes256gcm_decrypt_cipher(&symm_key, &iv2)?;

        let mut entropy = Rdseed;
        let mut rng = CtrDrbg::new(&mut entropy, None)?;
        let mut iv = [0u8; 16];
        rng.random(&mut iv)?;
        let encrypt_cipher = new_aes256gcm_encrypt_cipher(&symm_key, &iv)?;

        // The values received from the tenant are decrypted.
        let mut plaintext1 = [0u8; 32];
        let mut plaintext2 = [0u8; 32];
        let _ = decrypt_cipher_1.decrypt_auth(&hash_v1, &ciphertext1, &mut plaintext1, &tag1)?;
        let _ = decrypt_cipher_2.decrypt_auth(&hash_v2, &ciphertext2, &mut plaintext2, &tag2)?;

        // The values received from the tenant are converted back to 32-bit unsigned ints.
        let num1 = Cursor::new(plaintext1).read_u32::<NativeEndian>()?;
        let num2 = Cursor::new(plaintext2).read_u32::<NativeEndian>()?;

        // The sum of the two plaintext values is calculated.
        let sum: u32 = num1 + num2;
        println!("\n{} + {} = {}", num1, num2, sum);

        // The sum is converted from u32 to bytes to serve as input for the encryption function.
        // The extra 5th byte is in case of overflow.
        let mut sum_as_bytes = [0u8; 5];
        NativeEndian::write_u32(&mut sum_as_bytes, sum);

        // The sum is encrypted.
        let mut ciphersum = [0u8; 5];
        let mut tag = [0u8; 16];
        let mut hash_of_sum = [0u8; 32];
        Md::hash(Sha256, &sum_as_bytes, &mut hash_of_sum)?;
        let _ =
            encrypt_cipher.encrypt_auth(&hash_of_sum, &sum_as_bytes, &mut ciphersum, &mut tag)?;

        // The tag, iv, additional data, and encrypted sum are sent back to the tenant.
        to_writer(&mut stream, &tag)?;
        to_writer(&mut stream, &iv)?;
        to_writer(&mut stream, &hash_of_sum)?;
        to_writer(&mut stream, &ciphersum)?;

        // TODO: This line exits the program after one run. Otherwise, it appears as though the tenant can be run
        // again, but instead the program just hangs the second time. Why?
        break;
    }

    Ok(())
}
