use clap::Parser;

#[cfg(feature = "certgen")] 
#[derive(Parser)]
pub struct Args {
    /// Path to the server certificate file
    #[clap(short, long, parse(from_os_str), default_value = "server.crt")]
    pub cert: std::path::PathBuf,

    /// Path to the server private key file
    #[clap(short, long, parse(from_os_str), default_value = "server.key")]
    pub key: std::path::PathBuf,

    /// Port for the tls control connection
    #[clap(short, long, default_value_t = 9999)]
    pub tls_port: u16,

    /// Deletes existing certificates, generates new ones and exits the program.
    #[clap(short = 'g', long)]
    pub cert_gen: bool,
}


#[cfg(not(feature = "certgen"))] 
#[derive(Parser)]
pub struct Args {
    /// Path to the server certificate file
    #[clap(short, long, parse(from_os_str), default_value = "server.crt")]
    pub cert: std::path::PathBuf,

    /// Path to the server private key file
    #[clap(short, long, parse(from_os_str), default_value = "server.key")]
    pub key: std::path::PathBuf,

    /// Port for the tls control connection
    #[clap(short, long, default_value_t = 9999)]
    pub tls_port: u16,
}