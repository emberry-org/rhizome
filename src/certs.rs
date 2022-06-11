#[cfg(feature = "certgen")]
use rcgen::generate_simple_self_signed;
#[cfg(feature = "certgen")]
use std::fs::OpenOptions;
use std::{fs::File, io, path::PathBuf};
use tokio_rustls::rustls::{Certificate, PrivateKey};

// Load public certificate from file.
pub fn load_certs(filename: PathBuf) -> io::Result<Vec<Certificate>> {
    // Open certificate file.
    let certfile = File::open(filename)?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    let certs = rustls_pemfile::certs(&mut reader)?;

    Ok(certs.into_iter().map(Certificate).collect())
}

// Load private key from file.
pub fn load_private_key(filename: PathBuf) -> io::Result<PrivateKey> {
    // Open keyfile.
    let keyfile = File::open(filename)?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    match rustls_pemfile::read_one(&mut reader)? {
        Some(rustls_pemfile::Item::PKCS8Key(key)) => Ok(PrivateKey(key)),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            "Private key has to be the first entry in the key file.".to_string(),
        )),
    }
}

#[cfg(feature = "certgen")]
pub fn regenerate_certs(certfile: PathBuf, keyfile: PathBuf) -> io::Result<()> {
    use std::io::Write;

    let subject_alt_names = vec!["rhizome".to_string(), "localhost".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();

    let mut file_crt = OpenOptions::new().create(true).write(true).open(certfile)?;
    file_crt.write_all(cert.serialize_pem().unwrap().as_bytes())?;

    let mut file_key = OpenOptions::new().create(true).write(true).open(keyfile)?;
    file_key.write_all(cert.serialize_private_key_pem().as_bytes())?;
    Ok(())
}
