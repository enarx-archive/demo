use sgx_isa::Report;
use std::error::Error;
use std::net::TcpListener;

const LISTENER_ADDR: &'static str = "localhost:1032";

fn main() -> Result<(), Box<dyn Error>> {
    println!("\nListening on {}....\n", LISTENER_ADDR);

    // The enclave handles each incoming connection from attestation daemon.
    for stream in TcpListener::bind(LISTENER_ADDR).unwrap().incoming() {
        let mut stream = stream?;

        // The enclave receives the identity of the Quoting Enclave from the
        // attestation daemon, in the form of a (serialized) TargetInfo
        // structure. The TargetInfo contains the measurement and attribute flags
        // of the Quoting Enclave.
        let qe_id: sgx_isa::Targetinfo = serde_json::from_reader(&mut stream)?;

        // The enclave creates a Report attesting its identity, with the Quoting
        // Enclave (whose identity was just received) as the Report's target. The
        // blank ReportData field must be passed in as a &[u8; 64].
        let report = {
            let report_data = [0u8; 64];
            Report::for_target(&qe_id, &report_data)
        };

        // The enclave sends its attestation Report back to the attestation daemon.
        serde_json::to_writer(&mut stream, &report)?;

        println!("Successfully sent report to daemon.");
    }

    Ok(())
}
