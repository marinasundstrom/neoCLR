//! Protocol-neutral credential/trust configuration shared by both host platforms.
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use std::sync::Arc;

fn alpn(protocol: &[u8]) -> Result<Vec<Vec<u8>>, ()> {
    if protocol.len() > 255 {
        return Err(());
    }
    Ok(if protocol.is_empty() {
        vec![]
    } else {
        vec![protocol.to_vec()]
    })
}

pub(crate) fn server(cert: &[u8], key: &[u8], protocol: &[u8]) -> Result<ServerConfig, ()> {
    let certs = rustls_pemfile::certs(&mut &cert[..])
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ())?;
    if certs.is_empty() {
        return Err(());
    }
    let key = rustls_pemfile::private_key(&mut &key[..])
        .map_err(|_| ())?
        .ok_or(())?;
    let mut config =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|_| ())?
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|_| ())?;
    config.alpn_protocols = alpn(protocol)?;
    config.send_tls13_tickets = 0;
    config.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});
    Ok(config)
}

pub(crate) fn client(cert: &[u8], protocol: &[u8]) -> Result<ClientConfig, ()> {
    let certs = rustls_pemfile::certs(&mut &cert[..])
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ())?;
    if certs.is_empty() {
        return Err(());
    }
    let mut roots = RootCertStore::empty();
    for cert in certs {
        roots.add(cert).map_err(|_| ())?;
    }
    let mut config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|_| ())?
            .with_root_certificates(roots)
            .with_no_client_auth();
    config.alpn_protocols = alpn(protocol)?;
    config.resumption = rustls::client::Resumption::disabled();
    Ok(config)
}
