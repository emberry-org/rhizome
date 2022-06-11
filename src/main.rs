use clap::Parser;
use rhizome::Args;

fn main() {
    let args = Args::parse();

    #[cfg(feature = "certgen")]
    if args.cert_gen {
        rhizome::regenerate_certs(args.cert, args.key).unwrap();
        return;
    }

    if let Err(e) = rhizome::run(args) {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
}
