mod cert_chain;
mod key;
mod sig;

use bufstream::BufStream;
use dcap_ql::quote::{Qe3CertDataPckCertChain, Quote3SignatureEcdsaP256};
use openssl::x509::*;
use std::{
    borrow::Borrow,
    convert::TryFrom,
    env,
    error::Error,
    fs,
    io::{Read, Write},
    iter::Iterator,
    net::TcpStream,
};

const DAEMON_CONN: &'static str = "localhost:1034";

/// The tenant requests attestation of an enclave from the platform's attestation daemon, and
/// receives a Quote from the daemon. The Quote verifies the enclave's measurement. The tenant
/// verifies:
/// 1. That the Quote's PCK Certificate (embedded in the Cert Data) is valid.
/// 2. That the PCK Certificate's Key signed the platform's Attestation Key.
/// 3. That the Attestation Key signed the Quote.
/// 4. That the hashed material (containing the Attestation Key) signed by the PCK is valid.
///
/// For more information on Intel's PCK and certificate chains, you may refer to:
/// https://download.01.org/intel-sgx/dcap-1.0/docs/SGX_PCK_Certificate_CRL_Spec-1.0.pdf
///
/// For more informtation on Intel's Attestation Key and the Quote, you may refer to:
/// https://download.01.org/intel-sgx/dcap-1.0/docs/SGX_ECDSA_QuoteGenReference_DCAP_API_Linux_1.0.pdf

fn main() -> Result<(), Box<dyn Error>> {
    // The tenant's PCK certificate chain must be loaded to verify the Quote's PCK Leaf
    // Certificate. The root certificate in this chain is trusted, since it is provided by the
    // tenant.
    let cert_chain_file = env::args()
        .nth(1)
        .expect("You must supply the path of a valid PCK certificate chain as the first argument.");
    let cert_chain_file_contents =
        fs::read_to_string(&cert_chain_file[..]).expect("PCK cert chain file path invalid.");

    // The tenant requests attestation from the platform's attestation daemon.
    // The actual signal is arbitrary.
    let daemon_conn = TcpStream::connect(DAEMON_CONN)?;
    let mut daemon_buf = BufStream::new(daemon_conn);
    daemon_buf.write(&b"Request attestation"[..])?;

    // The tenant receives a Quote from the platform's attestation
    // daemon. This Quote verifies the enclave's self-measurement from its Report.
    let mut quote = Vec::new();
    daemon_buf.read_to_end(&mut quote)?;

    // The signed material for the Quoting Enclave's Attestation Key (Quote Header ||
    // ISV Enclave Report) is retrieved.
    let att_key_signed_material = dcap_ql::quote::Quote::raw_header_and_body(&quote)?;

    // The hashed material (containing the Attestation Key) signed by the PCK is retrieved.
    let hashed_reportdata = dcap_ql::quote::Quote::raw_pck_hash(&quote)?;

    // This parses the Quote's signature section.
    let quote = dcap_ql::quote::Quote::parse(&quote)?;
    let q_sig = quote.signature::<Quote3SignatureEcdsaP256>()?;
    let q_enclave_report_sig = q_sig.signature();
    let q_qe_report = q_sig.qe3_report();
    let q_qe_report_sig = q_sig.qe3_signature();
    let q_att_key_pub = q_sig.attestation_public_key();
    let q_auth_data = q_sig.authentication_data();

    // The Quote's Certification Data contains the PCK Cert Chain and PCK Certificate;
    // the embedded PCK signs the Attestation Key.
    let cert_data = q_sig.certification_data::<Qe3CertDataPckCertChain>()?;
    let quote_pck_leaf_cert = cert_data.leaf_cert;

    // The PCK chain is reconstructed with the Quote's leaf cert added to end of tenant's chain.
    let cert_chain = cert_chain::CertChain::new_from_chain(
        X509::stack_from_pem(cert_chain_file_contents.as_bytes())?,
        &quote_pck_leaf_cert,
    );
    cert_chain.len_ok()?;
    println!("Tenant's PCK cert chain loaded...");

    // The PCK certificate chain's issuers and signatures are verified.
    cert_chain.verify_issuers()?;
    cert_chain.verify_sigs()?;
    println!("PCK cert chain verified...");

    // The Attestation Key's signature on the Quote is verified.
    let attestation_key = key::Key::new_from_xy(&q_att_key_pub)?;
    let quote_signature = sig::Signature::try_from(q_enclave_report_sig)?.to_der_vec()?;
    attestation_key.verify_sig(&att_key_signed_material, &quote_signature)?;
    println!("AK signature on Quote header || report body is valid...");

    // The PCK's signature on the Attestation Public Key is verified.
    let pc_key = key::Key::new_from_pubkey(quote_pck_leaf_cert.public_key()?);
    let qe_report_signature = sig::Signature::try_from(q_qe_report_sig)?.to_der_vec()?;
    pc_key
        .borrow()
        .verify_sig(&q_qe_report, &qe_report_signature)?;
    println!("PCK signature on AK is valid...");

    // This verifies that the hashed material signed by the PCK is correct.
    let mut unhashed_data = Vec::new();
    unhashed_data.extend(q_att_key_pub.to_vec());
    unhashed_data.extend(q_auth_data.to_vec());
    pc_key
        .borrow()
        .verify_hash(hashed_reportdata, unhashed_data)?;
    println!("QE Report's hash is valid....");

    println!("\nQuote verified.");
    Ok(())
}
