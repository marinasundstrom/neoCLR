//! Private nonblocking TLS adapter. The host owns sockets, deadlines and scheduling.
use rustls::{ServerConfig, ServerConnection};
use std::{
    ffi::{CStr, c_char, c_void},
    fs::File,
    io::{self, Read, Write},
    sync::Arc,
};

const LIMIT: usize = 65536;
type Transfer = unsafe extern "C" fn(*mut c_void, *mut u8, u32, i32) -> i64;
struct Wire {
    context: *mut c_void,
    transfer: Transfer,
}
impl Read for Wire {
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
        translate(unsafe {
            (self.transfer)(
                self.context,
                data.as_mut_ptr(),
                data.len().min(16384) as u32,
                0,
            )
        })
    }
}
impl Write for Wire {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        translate(unsafe {
            (self.transfer)(
                self.context,
                data.as_ptr().cast_mut(),
                data.len().min(16384) as u32,
                1,
            )
        })
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn translate(value: i64) -> io::Result<usize> {
    match value {
        -1 => Err(io::ErrorKind::WouldBlock.into()),
        n if n < 0 => Err(io::ErrorKind::ConnectionAborted.into()),
        n => Ok(n as usize),
    }
}
unsafe fn file(path: *const c_char) -> Option<Vec<u8>> {
    if path.is_null() {
        return None;
    }
    let name = unsafe { CStr::from_ptr(path) }.to_str().ok()?;
    let mut data = Vec::new();
    File::open(name)
        .ok()?
        .take((LIMIT + 1) as u64)
        .read_to_end(&mut data)
        .ok()?;
    (data.len() <= LIMIT).then_some(data)
}
/// Input paths are NUL-terminated UTF-8. Returns null on invalid credentials.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_config_new(
    cert: *const c_char,
    key: *const c_char,
) -> *mut c_void {
    let create = || -> Option<Arc<ServerConfig>> {
        let cert = unsafe { file(cert) }?;
        let key = unsafe { file(key) }?;
        let certs = rustls_pemfile::certs(&mut &cert[..])
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        let key = rustls_pemfile::private_key(&mut &key[..]).ok()??;
        let mut config =
            ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .ok()?
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .ok()?;
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        // No tickets/session cache in this first bounded hosting proof.
        config.send_tls13_tickets = 0;
        config.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});
        Some(Arc::new(config))
    };
    create().map_or(std::ptr::null_mut(), |config| {
        Box::into_raw(Box::new(config)).cast()
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_config_free(config: *mut c_void) {
    if !config.is_null() {
        drop(unsafe { Box::from_raw(config.cast::<Arc<ServerConfig>>()) });
    }
}
struct Session {
    connection: ServerConnection,
    pending: usize,
    failed: bool,
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_session_new(config: *mut c_void) -> *mut c_void {
    if config.is_null() {
        return std::ptr::null_mut();
    }
    let config = unsafe { &*config.cast::<Arc<ServerConfig>>() };
    match ServerConnection::new(config.clone()) {
        Ok(mut connection) => {
            connection.set_buffer_limit(Some(LIMIT));
            Box::into_raw(Box::new(Session {
                connection,
                pending: 0,
                failed: false,
            }))
            .cast()
        }
        Err(_) => std::ptr::null_mut(),
    }
}
fn flush(connection: &mut ServerConnection, wire: &mut Wire) -> io::Result<bool> {
    if connection.wants_write() {
        match connection.write_tls(wire) {
            Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
            Ok(_) => (),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) => return Err(error),
        }
    }
    Ok(!connection.wants_write())
}
fn transfer(
    session: &mut Session,
    wire: &mut Wire,
    data: &mut [u8],
    sending: bool,
) -> io::Result<usize> {
    let connection = &mut session.connection;
    // Bounded record writes and one record read per poll, never a blocking loop.
    if !flush(connection, wire)? {
        return Err(io::ErrorKind::WouldBlock.into());
    }
    if sending && session.pending != 0 {
        return Ok(std::mem::take(&mut session.pending));
    }
    if !sending && !connection.is_handshaking() {
        match connection.reader().read(data) {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => (),
            result => return result,
        }
    }
    if connection.is_handshaking() || !sending {
        match connection.read_tls(wire) {
            Ok(0) if connection.is_handshaking() => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(_) => {
                connection.process_new_packets().map_err(io::Error::other)?;
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => (),
            Err(error) => return Err(error),
        }
        if !flush(connection, wire)? || connection.is_handshaking() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
    }
    if sending {
        session.pending = connection.writer().write(data)?;
        if !flush(connection, wire)? {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        Ok(std::mem::take(&mut session.pending))
    } else {
        connection.reader().read(data)
    }
}
/// Returns -1 pending, -2 terminal transport/TLS failure, otherwise plaintext bytes.
/// A pending send must be retried with the same operation until completion or abort.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_transfer(
    session: *mut c_void,
    context: *mut c_void,
    callback: Transfer,
    data: *mut u8,
    count: u32,
    sending: i32,
) -> i64 {
    let session = unsafe { &mut *session.cast::<Session>() };
    if session.failed {
        return -2;
    }
    let data = unsafe { std::slice::from_raw_parts_mut(data, count as usize) };
    match transfer(
        session,
        &mut Wire {
            context,
            transfer: callback,
        },
        data,
        sending != 0,
    ) {
        Ok(value) => value as i64,
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => -1,
        Err(_) => {
            session.failed = true;
            -2
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_abort(session: *mut c_void) {
    if !session.is_null() {
        unsafe { &mut *session.cast::<Session>() }.failed = true;
    }
}
/// Synchronous teardown: best-effort close_notify, then drop; never waits for peer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_session_free(
    session: *mut c_void,
    context: *mut c_void,
    callback: Transfer,
) {
    if session.is_null() {
        return;
    }
    let mut session = unsafe { Box::from_raw(session.cast::<Session>()) };
    if !session.failed && session.pending == 0 && !session.connection.is_handshaking() {
        session.connection.send_close_notify();
        let _ = flush(
            &mut session.connection,
            &mut Wire {
                context,
                transfer: callback,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::VecDeque, ffi::CString, io::Cursor};

    struct Memory {
        incoming: VecDeque<u8>,
        outgoing: Vec<u8>,
        writes: usize,
        eof: bool,
    }
    unsafe extern "C" fn fragmented(
        state: *mut c_void,
        data: *mut u8,
        count: u32,
        sending: i32,
    ) -> i64 {
        let state = unsafe { &mut *state.cast::<Memory>() };
        let data = unsafe { std::slice::from_raw_parts_mut(data, count as usize) };
        if sending != 0 {
            state.writes += 1;
            if state.writes % 2 == 1 {
                return -1;
            }
            let count = data.len().min(7);
            state.outgoing.extend_from_slice(&data[..count]);
            count as i64
        } else {
            let count = data.len().min(11).min(state.incoming.len());
            if count == 0 {
                return if state.eof { 0 } else { -1 };
            }
            for item in &mut data[..count] {
                *item = state.incoming.pop_front().unwrap();
            }
            count as i64
        }
    }
    fn exchange(client: &mut rustls::ClientConnection, memory: &mut Memory) {
        if client.wants_write() {
            let mut bytes = Vec::new();
            client.write_tls(&mut bytes).unwrap();
            memory.incoming.extend(bytes);
        }
        if !memory.outgoing.is_empty() {
            client
                .read_tls(&mut Cursor::new(std::mem::take(&mut memory.outgoing)))
                .unwrap();
            client.process_new_packets().unwrap();
        }
    }

    fn connected() -> (*mut c_void, rustls::ClientConnection, Memory) {
        let cert_path = CString::new(std::env::var("NEOCLR_HTTPS_CERTIFICATE").unwrap()).unwrap();
        let key_path = CString::new(std::env::var("NEOCLR_HTTPS_PRIVATE_KEY").unwrap()).unwrap();
        let config = unsafe { neoclr_tls_config_new(cert_path.as_ptr(), key_path.as_ptr()) };
        assert!(!config.is_null());
        let raw = unsafe { neoclr_tls_session_new(config) };
        unsafe { neoclr_tls_config_free(config) }; // Session retains config ownership.
        let session = unsafe { &mut *raw.cast::<Session>() };
        let certs = unsafe { file(cert_path.as_ptr()) }.unwrap();
        let mut roots = rustls::RootCertStore::empty();
        for cert in rustls_pemfile::certs(&mut &certs[..]) {
            roots.add(cert.unwrap()).unwrap();
        }
        let client_config = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();
        let mut client =
            rustls::ClientConnection::new(Arc::new(client_config), "localhost".try_into().unwrap())
                .unwrap();
        let mut memory = Memory {
            incoming: VecDeque::new(),
            outgoing: Vec::new(),
            writes: 0,
            eof: false,
        };
        let mut scratch = [0; 32];
        for _ in 0..10000 {
            exchange(&mut client, &mut memory);
            let result = transfer(
                session,
                &mut Wire {
                    context: (&mut memory as *mut Memory).cast(),
                    transfer: fragmented,
                },
                &mut scratch,
                false,
            );
            assert!(matches!(result, Err(ref e) if e.kind() == io::ErrorKind::WouldBlock));
            if !client.is_handshaking() && !session.connection.is_handshaking() {
                break;
            }
        }
        assert!(!client.is_handshaking() && !session.connection.is_handshaking());
        (raw, client, memory)
    }

    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths"]
    fn fragmented_handshake_and_backpressured_send() {
        let (raw, mut client, mut memory) = connected();
        let session = unsafe { &mut *raw.cast::<Session>() };
        let mut scratch = [0; 32];
        let mut payload = vec![42; 8192];
        let mut acknowledged = false;
        let mut received = Vec::new();
        let mut pending_polls = 0;
        for _ in 0..10000 {
            if !acknowledged {
                match transfer(
                    session,
                    &mut Wire {
                        context: (&mut memory as *mut Memory).cast(),
                        transfer: fragmented,
                    },
                    &mut payload,
                    true,
                ) {
                    Ok(count) => {
                        assert_eq!(count, payload.len());
                        acknowledged = true;
                    }
                    Err(error) => {
                        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
                        pending_polls += 1;
                    }
                }
            }
            exchange(&mut client, &mut memory);
            loop {
                match client.reader().read(&mut scratch) {
                    Ok(0) => panic!("unexpected EOF"),
                    Ok(count) => received.extend_from_slice(&scratch[..count]),
                    Err(error) => {
                        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
                        break;
                    }
                }
            }
            if acknowledged && received.len() >= payload.len() {
                break;
            }
        }
        assert!(acknowledged && pending_polls > 100);
        assert_eq!(received, payload); // No replay while a write waits for its ciphertext flush.
        unsafe { neoclr_tls_abort(raw) };
        assert_eq!(
            unsafe {
                neoclr_tls_transfer(
                    raw,
                    (&mut memory as *mut Memory).cast(),
                    fragmented,
                    scratch.as_mut_ptr(),
                    32,
                    1,
                )
            },
            -2
        );
        unsafe { neoclr_tls_session_free(raw, (&mut memory as *mut Memory).cast(), fragmented) };
    }

    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths"]
    fn authenticated_eof_preserves_data_and_requires_close_notify() {
        for graceful in [false, true] {
            let (raw, mut client, mut memory) = connected();
            let payload = vec![73; 1024];
            client.writer().write_all(&payload).unwrap();
            if graceful {
                client.send_close_notify();
            }
            exchange(&mut client, &mut memory);
            memory.eof = true;
            let mut scratch = [0; 32];
            let mut received = Vec::new();
            let mut terminal = None;
            for _ in 0..10000 {
                let result = unsafe {
                    neoclr_tls_transfer(
                        raw,
                        (&mut memory as *mut Memory).cast(),
                        fragmented,
                        scratch.as_mut_ptr(),
                        scratch.len() as u32,
                        0,
                    )
                };
                exchange(&mut client, &mut memory);
                if result > 0 {
                    received.extend_from_slice(&scratch[..result as usize]);
                } else if result != -1 {
                    terminal = Some(result);
                    break;
                }
            }
            assert_eq!(
                received, payload,
                "buffered plaintext must be delivered before EOF"
            );
            let expected = if graceful { 0 } else { -2 };
            assert_eq!(terminal, Some(expected));
            // An abrupt EOF poisons the session; subsequent operations cannot resume it.
            assert_eq!(
                unsafe {
                    neoclr_tls_transfer(
                        raw,
                        (&mut memory as *mut Memory).cast(),
                        fragmented,
                        scratch.as_mut_ptr(),
                        scratch.len() as u32,
                        0,
                    )
                },
                expected
            );
            unsafe {
                neoclr_tls_session_free(raw, (&mut memory as *mut Memory).cast(), fragmented)
            };
        }
    }
}
