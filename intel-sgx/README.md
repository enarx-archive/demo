# SGX Remote Attestation
This is a demo of the SGX remote attestation process using Intel's
[DCAP](https://download.01.org/intel-sgx/dcap-1.0/docs/SGX_ECDSA_QuoteGenReference_DCAP_API_Linux_1.0.pdf) on a CPU that
supports [SGX2](https://github.com/ayeks/SGX-hardware#hardware-with-sgx2-support).

It contains 3 parts: the [attesting enclave](https://github.com/enarx/demo/tree/master/intel-sgx/attestation-enclave);
the [tenant](https://github.com/enarx/demo/tree/master/intel-sgx/attestation-tenant) requesting attestation; and the
[daemon](https://github.com/enarx/demo/tree/master/intel-sgx/attestation-daemon) to communicate between the first two
until attestation is complete. These can all run on the same machine for the purpose of the demonstration.
However, in a production scenario the enclave and daemon would run in a different location, and therefore on a separate machine, than the tenant.

To run this code, you will need to provide the PCK certificate chain you would like to use to
validate the enclave's attestation. See the section below on how to obtain this certificate chain.

### Checking for SGX2 support
CPUs that support SGX may not support SGX2 with Flexible Launch Control, which is required to run this demo. The known good hardware that
supports SGX2 is the Intel NUC Kit model NUC7CJYH, though there may be others. To check if your hardware supports SGX2, you can use
the test provided by [Fortanix](https://github.com/fortanix/rust-sgx). You will need to have Rust Nightly and the Fortanix EDP components
installed before you can run their test with `sgx-detect` as shown on their page. The result of this test should how a green check mark for 
`SGX Features: SGX2` to indicate that your system supports SGX2.

![PACSPull Plugin](/intel-sgx/images/sgx-detect-sgx2.png)

### Retrieving PCK certificate chain for Intel SGX Remote Attestation
The PCK certificate chain is needed to validate the Quote that contains the enclave's attestation. It is meant to be
retrieved separately by the user or tenant (the party requesting attestation) and is assumed to be trusted by the user.
It contains the root and any intermediate certificates from Intel; the Quote will contain the leaf certificate, known
as the PCK Cert.

The root and intermediate certificates can be retrieved as a chain from
[Intel's API](https://api.portal.trustedservices.intel.com/documentation#pcs-certificate) without registering for an API key.
They can be retrieved by using the following command,
which parses the response from Intel and places it in a file called `pck_chain.pem`:
```console
curl -v "https://api.trustedservices.intel.com/sgx/certification/v1/pckcrl?ca={processor}"  
2>&1 | awk -F"SGX-PCK-CRL-Issuer-Chain: " '{print $2}' | sed -e :a -e  
's@%@\\x@g;/./,$!d;/^\n*$/{$d;N;};/\n$/ba' | xargs -0 printf "%b" > pck_chain.pem
```
The output file, `pck_chain.pem`, will include the Intermediate and Root PCK certificates from Intel. This chain is used to
verify the Quote's PCK Cert, the leaf certificate corresponding to this same certificate chain. There is no need to manually
add the root cert to the system's trusted root certs, as the code does not rely on these.

### Installing Intel's DCAP driver and components
The specific Intel components needed to run this demo are:
- Intel SGX [DCAP driver](https://download.01.org/intel-sgx/dcap-1.0/sgx_linux_x64_driver_license_updated_dcap_a06cb75.bin). After downloading, this can be installed with `sudo bash <file>.bin`.
- Intel SGX [DCAP Quoting Library](https://download.01.org/intel-sgx/dcap-1.0/DCAP_installers/ubuntu18.04/) (the library, dbg, and dev are all needed).
- Intel SGX [Enclave Common](https://download.01.org/intel-sgx/dcap-1.0/SGX_installers/ubuntu18.04/) (as well as dbg).

The Intel SGX SDK is **not** necessary to run the demo.


### How to Run the SGX Demo
1. Make sure your CPU is [SGX2-capable](https://github.com/ayeks/SGX-hardware#hardware-with-sgx2-support) and supports
Flexible Launch Control (see [section](https://github.com/lkatalin/demo/blob/official-sgx-demo-attestation-only/intel-sgx/README.md#checking-for-sgx2-support) above). You should be running Ubuntu 18.04 or Ubuntu 16.04. 

2. Install Intel's DCAP driver and other components
from this [page](https://download.01.org/intel-sgx/dcap-1.0/) (see [section](https://github.com/lkatalin/demo/blob/official-sgx-demo-attestation-only/intel-sgx/README.md#installing-intels-dcap-driver-and-components) above). Note that the Intel's DCAP
driver is **different** from Intel's default SGX driver. The default Intel SGX driver for attestation with
the Intel Attestation Service will not work and documentation online suggesting installation of this driver should be ignored. 

3. Install [Rust](https://www.rust-lang.org/tools/install) Nightly. After installing Rust, you can use `rustup default nightly` to use Nightly Rust.

4. Install the Fortanix EDP, following the steps on this [page](https://github.com/fortanix/rust-sgx). These steps will install the `x86-64-fortanix-unknown-sgx` compilation target. 

5. Install the Fortanix DCAP Quote Provider. Either obtain the crate from this [link](https://crates.io/api/v1/crates/dcap-provider/0.2.0/download) or clone the Fortanix `rust-sgx` 
[repo](https://github.com/fortanix/rust-sgx). In either case, navigate to the `dcap-provider` crate and build it with `cargo build --release`. Find the `libdcap-quoteprov.so` file inside
`dcap-provider/target/release` and move it to `/usr/local/lib`.

6. Retrieve Intel's PCK certificate chain as described in the [section](https://github.com/lkatalin/demo/blob/official-sgx-demo-attestation-only/intel-sgx/README.md#retrieving-pck-certificate-chain-for-intel-sgx-remote-attestation) above.

7. After cloning this repo, run the `attestation-enclave` with `cargo run --target x86_64-fortanix-unknown-sgx` and leave it running. Run the `attestation-daemon` with `cargo run` and leave it running. These both must be running before the tenant requests attestation (step 8).

8. Run the `attestation-tenant` with `cargo run <filepath>`, where filepath is the path to the PCK certificate chain from Step 6.
