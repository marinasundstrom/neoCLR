//! Protocol-neutral credential/trust configuration shared by both host platforms.
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use std::sync::Arc;

/// Private ABI status values; never serialize provider error strings or key material.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConfigurationError {
    InvalidInput = -10,
    InvalidCertificate = -11,
    InvalidPrivateKey = -12,
    KeyMismatch = -13,
    InvalidTrustAnchor = -14,
    InvalidProtocol = -15,
    ProviderFailure = -16,
}

fn alpn(protocol: &[u8]) -> Result<Vec<Vec<u8>>, ConfigurationError> {
    if protocol.len() > 255 {
        return Err(ConfigurationError::InvalidProtocol);
    }
    Ok(if protocol.is_empty() {
        vec![]
    } else {
        vec![protocol.to_vec()]
    })
}

pub(crate) fn server(
    cert: &[u8],
    key: &[u8],
    protocol: &[u8],
) -> Result<ServerConfig, ConfigurationError> {
    let protocols = alpn(protocol)?;
    let certs = rustls_pemfile::certs(&mut &cert[..])
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ConfigurationError::InvalidCertificate)?;
    if certs.is_empty() {
        return Err(ConfigurationError::InvalidCertificate);
    }
    let key = rustls_pemfile::private_key(&mut &key[..])
        .map_err(|_| ConfigurationError::InvalidPrivateKey)?
        .ok_or(ConfigurationError::InvalidPrivateKey)?;
    let mut config =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|_| ConfigurationError::ProviderFailure)?
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|error| match error {
                rustls::Error::InconsistentKeys(rustls::InconsistentKeys::KeyMismatch) => {
                    ConfigurationError::KeyMismatch
                }
                rustls::Error::InvalidCertificate(_) => ConfigurationError::InvalidCertificate,
                _ => ConfigurationError::InvalidPrivateKey,
            })?;
    config.alpn_protocols = protocols;
    config.send_tls13_tickets = 0;
    config.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});
    Ok(config)
}

pub(crate) fn client(cert: &[u8], protocol: &[u8]) -> Result<ClientConfig, ConfigurationError> {
    let protocols = alpn(protocol)?;
    let certs = rustls_pemfile::certs(&mut &cert[..])
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ConfigurationError::InvalidTrustAnchor)?;
    if certs.is_empty() {
        return Err(ConfigurationError::InvalidTrustAnchor);
    }
    let mut roots = RootCertStore::empty();
    for cert in certs {
        roots
            .add(cert)
            .map_err(|_| ConfigurationError::InvalidTrustAnchor)?;
    }
    let mut config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|_| ConfigurationError::ProviderFailure)?
            .with_root_certificates(roots)
            .with_no_client_auth();
    config.alpn_protocols = protocols;
    config.resumption = rustls::client::Resumption::disabled();
    Ok(config)
}
