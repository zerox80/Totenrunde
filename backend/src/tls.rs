use std::{fs::File, io::BufReader, path::Path};

use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use crate::DynError;

pub fn server_config(
    certificates: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> Result<rustls::ServerConfig, DynError> {
    Ok(rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates, key)?)
}

pub fn client_config(
    certificates: Vec<CertificateDer<'static>>,
    alpn: &[u8],
) -> Result<rustls::ClientConfig, DynError> {
    let mut roots = rustls::RootCertStore::empty();
    for certificate in certificates {
        roots.add(certificate)?;
    }
    let mut tls = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls.alpn_protocols = vec![alpn.to_vec()];
    Ok(tls)
}

pub fn load_certificates(path: &Path) -> Result<Vec<CertificateDer<'static>>, DynError> {
    let file = File::open(path).map_err(|error| {
        format!(
            "TLS-Zertifikat {} konnte nicht geöffnet werden: {error}",
            path.display()
        )
    })?;
    let certificates =
        rustls_pemfile::certs(&mut BufReader::new(file)).collect::<Result<Vec<_>, _>>()?;
    if certificates.is_empty() {
        return Err(format!("TLS-Zertifikat {} ist leer", path.display()).into());
    }
    Ok(certificates)
}

pub fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>, DynError> {
    let file = File::open(path).map_err(|error| {
        format!(
            "TLS-Schlüssel {} konnte nicht geöffnet werden: {error}",
            path.display()
        )
    })?;
    rustls_pemfile::private_key(&mut BufReader::new(file))?
        .ok_or_else(|| format!("TLS-Schlüssel {} ist leer oder ungültig", path.display()).into())
}
