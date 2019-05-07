use ketuvim::{Kvm, VirtualMachine, VirtualCpu, MemoryFlags, Reason, ReasonIo, arch, util::map};
use std::convert::TryFrom;
use std::fs::File;
use codicon::*;

fn fetch_chain(fw: &sev::firmware::Firmware) -> sev::certs::Chain {
    const CEK_SVC: &str = "https://kdsintf.amd.com/cek/id";
    const NAPLES: &str = "https://developer.amd.com/wp-content/resources/ask_ark_naples.cert";

    let mut chain = fw.pdh_cert_export()
        .expect("unable to export SEV certificates");

    let id = fw.get_identifer().expect("error fetching identifier");
    let url = format!("{}/{}", CEK_SVC, id);

    let mut rsp = reqwest::get(&url)
        .expect(&format!("unable to contact server"));
    assert!(rsp.status().is_success());

    chain.cek = sev::certs::sev::Certificate::decode(&mut rsp, ())
        .expect("Invalid CEK!");

    let mut rsp = reqwest::get(NAPLES)
        .expect(&format!("unable to contact server"));
    assert!(rsp.status().is_success());

    sev::certs::Chain {
        ca: sev::certs::ca::Chain::decode(&mut rsp, ())
            .expect("Invalid CA chain!"),
        sev: chain,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: demo NUMBER NUMBER");
        std::process::exit(1);
    }

    let a = args[1].trim().parse::<u8>().expect("Must be a number!");
    if a > 4 { panic!("Number must be between 0 and 4, inclusive!"); }

    let b = args[2].trim().parse::<u8>().expect("Must be a number!");
    if b > 4 { panic!("Number must be between 0 and 4, inclusive!"); }

    let code = [
        0xba, 0xf8, 0x03, // mov $0x3f8, %dx
        0xb0, a,          // mov a, %al
        0xb3, b,          // mov b, %bl

        0x00, 0xd8,       // add %bl, %al
        0x04, b'0',       // add $'0', %al
        0xee,             // out %al, (%dx)

        0xb0, b'\n',      // mov $'\n', %al
        0xee,             // out %al, (%dx)

        0xf4,             // hlt
    ];

    // Server delivers chain and build to client...
    let fw = sev::firmware::Firmware::open().unwrap();
    let build = fw.platform_status().unwrap().build;
    let chain = if let Ok(mut file) = File::open("/tmp/demo.chain") {
        sev::certs::Chain::decode(&mut file, ()).unwrap()
    } else {
        println!("         SERVER: Fetch Certificate Chain");
        let chain = fetch_chain(&fw);
        let mut file = File::create("/tmp/demo.chain").unwrap();
        chain.encode(&mut file, ()).unwrap();
        chain
    };
    println!("CLIENT < SERVER: Certificate Chain");

    // Client creates session and starts the launch.
    let policy = sev::launch::Policy::default();
    let session = sev::session::Session::try_from(policy).unwrap();
    let start = session.start(chain).unwrap();
    println!("CLIENT         : Chain OK");
    println!("CLIENT > SERVER: Policy, Session Keys");

    // Server spins up the VM.
    let kvm = Kvm::open().unwrap();
    let mut vm = VirtualMachine::new(&kvm).unwrap();
    let mem = map::Map::<()>::build(map::Access::Shared)
        .protection(map::Protection::READ | map::Protection::WRITE)
        .flags(map::Flags::ANONYMOUS)
        .extra(0x1000)
        .done().unwrap();
    let addr = &*mem as *const () as u64;
    vm.add_region(0, MemoryFlags::default(), 0x1000, mem).unwrap();

    // Server takes a measurement and sends it to the client.
    let launch = ketuvim::sev::Launch::new(vm).unwrap();
    let launch = launch.start(start).unwrap();
    let launch = launch.measure().unwrap();
    let measurement = launch.measurement();
    println!("CLIENT < SERVER: VM Measurement");

    // Client verifies measurement and delivers encrypted code to server.
    let session = session.measure().unwrap();
    let session = session.verify(build, measurement).unwrap();
    println!("CLIENT         : Measurement OK");
    let secret = session.secret(sev::launch::HeaderFlags::default(), &code).unwrap();
    println!("CLIENT > SERVER: Encrypted Code/Data");

    // Server injects the encrypted code into the VM.
    print!("         SERVER: Inject Encrypted Code/Data: ");
    for b in secret.ciphertext.iter() { print!("{:02X}", *b) }
    println!("");
    let len = secret.ciphertext.len() as u32;
    launch.inject(secret, addr, len).unwrap();
    let (_, vm) = launch.finish().unwrap();
    println!("         SERVER: Guest Launched");

    // Setup special registers.
    let mut cpu = VirtualCpu::new(&vm).unwrap();
    let mut sregs = cpu.special_registers().unwrap();
    sregs.cs.base = 0;
    sregs.cs.selector = 0;
    cpu.set_special_registers(sregs).unwrap();

    // Setup registers.
    cpu.set_registers(arch::Registers {
        rip: 0x1000,
        rflags: 0x2,
        ..Default::default()
    }).unwrap();

    loop {
        match cpu.run().unwrap() {
            Reason::Halt => break,

            Reason::Io(io) => match io {
                ReasonIo::Out { port, data } => match port {
                    0x03f8 => for b in data {
                        unsafe { libc::putchar(*b as i32); }
                    },

                    _ => panic!("Unexpected IO port!"),
                },

                _ => panic!("Unexpected IO!"),
            },
        }
    }
}
