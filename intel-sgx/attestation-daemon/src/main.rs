use std::error::Error;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

const LISTENER_CONN: &'static str = "localhost:1034";
const ENCLAVE_CONN: &'static str = "localhost:1032";

fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "Daemon listening for attestation request on {}... ",
        LISTENER_CONN
    );

    // The attestation daemon handles each incoming connection from a tenant. The tenant, by
    // connecting, is requesting an attestation of the enclave.
    for incoming_tenant_stream in TcpListener::bind(LISTENER_CONN)?.incoming() {
        // The attestation daemon retrieves the Quoting Enclave's Target Info from the CPU and
        // sends the Quoting Enclave's Target Info to the enclave. This Target Info will be
        // used as the target for the enclave's attestation Report.
        let qe_ti = dcap_ql::target_info().expect("Could not retrieve QE target info.");

        // Serialize the Target Info onto the stream to the enclave
        let mut enclave_stream = TcpStream::connect(ENCLAVE_CONN)?;
        serde_json::to_writer(&mut enclave_stream, &qe_ti)?;
        enclave_stream.shutdown(std::net::Shutdown::Write)?;

        // The attestation daemon receives the Report back from the attesting enclave.
        let report: sgx_isa::Report = serde_json::from_reader(&mut enclave_stream)?;

        // The attestation daemon gets a Quote from the Quoting Enclave for the Report.
        // The Quoting Enclave verifies the Report's MAC as a prerequisite for generating
        // the Quote. The Quote is signed with the Quoting Enclave's Attestation Key.
        let quote = dcap_ql::quote(&report).expect("Could not generate quote.");

        // The attestation daemon sends the Quote to the tenant.
        let mut tenant_stream = incoming_tenant_stream?;
        tenant_stream.write(&quote)?;

        println!("\nQuote successfully generated and sent to tenant...");
    }
    Ok(())
}
