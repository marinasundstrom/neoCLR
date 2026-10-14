//! Private nonblocking TLS adapter. The host owns sockets, deadlines and scheduling.
use rustls::{ClientConfig, ClientConnection, Connection, ServerConfig, ServerConnection};
mod configuration;
pub mod resources;
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
///
/// # Safety
/// Nonnull paths must point to readable NUL-terminated strings for the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_config_new(
    cert: *const c_char,
    key: *const c_char,
) -> *mut c_void {
    let create = || -> Option<Arc<ServerConfig>> {
        let cert = unsafe { file(cert) }?;
        let key = unsafe { file(key) }?;
        configuration::server(&cert, &key, b"http/1.1")
            .ok()
            .map(Arc::new)
    };
    create().map_or(std::ptr::null_mut(), |config| {
        Box::into_raw(Box::new(config)).cast()
    })
}

// These byte-oriented entry points are private runtime ABI, not guest APIs.
// Input buffers are borrowed only for the call; configs own parsed credentials.
unsafe fn bytes<'a>(data: *const u8, count: u32) -> Option<&'a [u8]> {
    if count == 0 {
        return Some(&[]);
    }
    if data.is_null() || count as usize > LIMIT {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts(data, count as usize) })
}

/// Creates an owned configuration or returns a typed private status. Clears output
/// before validation; the output slot must be writable and must not alias inputs.
///
/// # Safety
/// All nonempty input buffers must be readable for their declared lengths for the call.
/// They must not be mutated concurrently.
/// A nonnull output slot must be aligned, writable, and disjoint from all inputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_server_config_create_pem(
    cert: *const u8,
    cert_count: u32,
    key: *const u8,
    key_count: u32,
    alpn: *const u8,
    alpn_count: u32,
    output: *mut *mut c_void,
) -> i32 {
    use configuration::ConfigurationError;
    if output.is_null() {
        return ConfigurationError::InvalidInput as i32;
    }
    unsafe { *output = std::ptr::null_mut() };
    let create = || {
        configuration::server(
            unsafe { bytes(cert, cert_count) }.ok_or(ConfigurationError::InvalidInput)?,
            unsafe { bytes(key, key_count) }.ok_or(ConfigurationError::InvalidInput)?,
            unsafe { bytes(alpn, alpn_count) }.ok_or(ConfigurationError::InvalidInput)?,
        )
    };
    match create() {
        Ok(config) => {
            unsafe { *output = Box::into_raw(Box::new(Arc::new(config))).cast() };
            0
        }
        Err(error) => error as i32,
    }
}

/// Explicit trust anchors only; output and ownership follow the server constructor.
///
/// # Safety
/// All nonempty input buffers must be readable for their declared lengths for the call.
/// They must not be mutated concurrently.
/// A nonnull output slot must be aligned, writable, and disjoint from all inputs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_client_config_create_pem(
    roots: *const u8,
    roots_count: u32,
    alpn: *const u8,
    alpn_count: u32,
    output: *mut *mut c_void,
) -> i32 {
    use configuration::ConfigurationError;
    if output.is_null() {
        return ConfigurationError::InvalidInput as i32;
    }
    unsafe { *output = std::ptr::null_mut() };
    let create = || {
        configuration::client(
            unsafe { bytes(roots, roots_count) }.ok_or(ConfigurationError::InvalidInput)?,
            unsafe { bytes(alpn, alpn_count) }.ok_or(ConfigurationError::InvalidInput)?,
        )
    };
    match create() {
        Ok(config) => {
            unsafe { *output = Box::into_raw(Box::new(Arc::new(config))).cast() };
            0
        }
        Err(error) => error as i32,
    }
}

/// Compatibility entry point for existing hosts. Empty ALPN is protocol-neutral.
///
/// # Safety
/// All nonempty input buffers must be readable for their declared lengths for the call.
/// They must not be mutated concurrently.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_server_config_from_pem(
    cert: *const u8,
    cert_count: u32,
    key: *const u8,
    key_count: u32,
    alpn: *const u8,
    alpn_count: u32,
) -> *mut c_void {
    let mut output = std::ptr::null_mut();
    unsafe {
        neoclr_tls_server_config_create_pem(
            cert,
            cert_count,
            key,
            key_count,
            alpn,
            alpn_count,
            &mut output,
        )
    };
    output
}

///
/// # Safety
/// All nonempty input buffers must be readable for their declared lengths for the call.
/// They must not be mutated concurrently.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_client_config_from_pem(
    roots: *const u8,
    roots_count: u32,
    alpn: *const u8,
    alpn_count: u32,
) -> *mut c_void {
    let mut output = std::ptr::null_mut();
    unsafe {
        neoclr_tls_client_config_create_pem(roots, roots_count, alpn, alpn_count, &mut output)
    };
    output
}

///
/// # Safety
/// A nonnull config must be an owned client configuration from this adapter.
/// Consume it exactly once, with no concurrent access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_client_config_free(config: *mut c_void) {
    if !config.is_null() {
        drop(unsafe { Box::from_raw(config.cast::<Arc<ClientConfig>>()) });
    }
}

/// The UTF-8 server name is verified, and is retained independently of this call.
///
/// # Safety
/// A nonnull config must be a live client configuration from this adapter,
/// which must not be freed during the call.
/// All nonempty input buffers must be readable for their declared lengths for the call.
/// They must not be mutated concurrently.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_client_session_new(
    config: *mut c_void,
    name: *const u8,
    name_count: u32,
) -> *mut c_void {
    let create = || {
        if config.is_null() {
            return None;
        }
        let name = std::str::from_utf8(unsafe { bytes(name, name_count) }?).ok()?;
        let name = rustls::pki_types::ServerName::try_from(name.to_owned()).ok()?;
        let config = unsafe { &*config.cast::<Arc<ClientConfig>>() };
        ClientConnection::new(config.clone(), name)
            .ok()
            .map(Connection::Client)
    };
    create().map_or(std::ptr::null_mut(), new_session)
}

fn make_session(mut connection: Connection) -> Session {
    connection.set_buffer_limit(Some(LIMIT));
    Session {
        connection,
        pending: 0,
        failed: false,
        closing: false,
    }
}
fn new_session(connection: Connection) -> *mut c_void {
    Box::into_raw(Box::new(make_session(connection))).cast()
}

///
/// # Safety
/// A nonnull config must be an owned server configuration from this adapter.
/// Consume it exactly once, with no concurrent access.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_config_free(config: *mut c_void) {
    if !config.is_null() {
        drop(unsafe { Box::from_raw(config.cast::<Arc<ServerConfig>>()) });
    }
}
struct Session {
    connection: Connection,
    pending: usize,
    failed: bool,
    closing: bool,
}
///
/// # Safety
/// A nonnull config must be a live server configuration from this adapter,
/// which must not be freed during the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_session_new(config: *mut c_void) -> *mut c_void {
    if config.is_null() {
        return std::ptr::null_mut();
    }
    let config = unsafe { &*config.cast::<Arc<ServerConfig>>() };
    ServerConnection::new(config.clone())
        .ok()
        .map(Connection::Server)
        .map_or(std::ptr::null_mut(), new_session)
}

fn flush(connection: &mut Connection, wire: &mut Wire) -> io::Result<bool> {
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
/// Drives authentication without reading or writing application plaintext.
/// Returns 0 authenticated, -1 pending, -2 terminal failure. Host serializes all polls.
///
/// # Safety
/// The session must be a live session created by this adapter and exclusively
/// accessible for the call.
/// The callback must remain valid for the call and accept the supplied context.
/// It must not unwind, retain buffer pointers, or reenter the session/context.
/// For receives it may write only the supplied range; for sends it must not mutate it.
/// Its nonnegative byte count must not exceed the supplied length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_handshake(
    session: *mut c_void,
    context: *mut c_void,
    callback: Transfer,
) -> i32 {
    if session.is_null() {
        return -2;
    }
    let session = unsafe { &mut *session.cast::<Session>() };
    if session.failed {
        return -2;
    }
    let connection = &mut session.connection;
    let mut wire = Wire {
        context,
        transfer: callback,
    };
    let result = (|| -> io::Result<bool> {
        if !flush(connection, &mut wire)? {
            return Ok(false);
        }
        if !connection.is_handshaking() {
            return Ok(true);
        }
        match connection.read_tls(&mut wire) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(_) => {
                connection.process_new_packets().map_err(io::Error::other)?;
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(false),
            Err(error) => return Err(error),
        }
        Ok(flush(connection, &mut wire)? && !connection.is_handshaking())
    })();
    match result {
        Ok(true) => 0,
        Ok(false) => -1,
        Err(_) => {
            session.failed = true;
            -2
        }
    }
}

/// Flushes a close-notify without waiting for the peer or closing the transport.
/// Returns 0 flushed, -1 pending, -2 terminal failure, -3 invalid operation state.
/// Complete authentication and any pending write first; serialize all session calls.
///
/// # Safety
/// The session must be a live session created by this adapter and exclusively
/// accessible for the call.
/// The callback must remain valid for the call and accept the supplied context.
/// It must not unwind, retain buffer pointers, or reenter the session/context.
/// For receives it may write only the supplied range; for sends it must not mutate it.
/// Its nonnegative byte count must not exceed the supplied length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_shutdown(
    session: *mut c_void,
    context: *mut c_void,
    callback: Transfer,
) -> i32 {
    if session.is_null() {
        return -2;
    }
    let session = unsafe { &mut *session.cast::<Session>() };
    if session.failed {
        return -2;
    }
    if session.connection.is_handshaking() || session.pending != 0 {
        return -3;
    }
    if !session.closing {
        session.closing = true;
        session.connection.send_close_notify();
    }
    match flush(
        &mut session.connection,
        &mut Wire {
            context,
            transfer: callback,
        },
    ) {
        Ok(true) => 0,
        Ok(false) => -1,
        Err(_) => {
            session.failed = true;
            -2
        }
    }
}

/// Returns -1 pending, -2 terminal transport/TLS failure, otherwise plaintext bytes.
/// Returns -3 for a new send after local shutdown; reads remain available.
/// A pending send must be retried with the same operation until completion or abort.
///
/// # Safety
/// The session must be a live session created by this adapter and exclusively
/// accessible for the call.
/// Data must be nonnull even for zero length, writable for count bytes, and
/// exclusively borrowed for the call, disjoint from session and callback state.
/// The callback must remain valid for the call and accept the supplied context.
/// It must not unwind, retain buffer pointers, or reenter the session/context.
/// For receives it may write only the supplied range; for sends it must not mutate it.
/// Its nonnegative byte count must not exceed the supplied length.
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
    if sending != 0 && session.closing {
        return -3;
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
///
/// # Safety
/// A nonnull session must be a live session from this adapter, exclusively accessible.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_abort(session: *mut c_void) {
    if !session.is_null() {
        unsafe { &mut *session.cast::<Session>() }.failed = true;
    }
}
/// Synchronous teardown: best-effort close_notify, then drop; never waits for peer.
///
/// # Safety
/// A nonnull session must be an owned session from this adapter.
/// Consume it exactly once, with no concurrent access.
/// The callback must remain valid for the call and accept the supplied context.
/// It must not unwind, retain buffer pointers, or reenter the session/context.
/// For receives it may write only the supplied range; for sends it must not mutate it.
/// Its nonnegative byte count must not exceed the supplied length.
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
        if !session.closing {
            session.connection.send_close_notify();
        }
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
    struct Endpoint {
        session: *mut c_void,
        wire: Memory,
    }
    impl Endpoint {
        fn new(session: *mut c_void) -> Self {
            assert!(!session.is_null());
            Self {
                session,
                wire: Memory {
                    incoming: VecDeque::new(),
                    outgoing: vec![],
                    writes: 0,
                    eof: false,
                },
            }
        }
        fn handshake(&mut self) -> i32 {
            unsafe {
                neoclr_tls_handshake(
                    self.session,
                    (&mut self.wire as *mut Memory).cast(),
                    fragmented,
                )
            }
        }
        fn shutdown(&mut self) -> i32 {
            unsafe {
                neoclr_tls_shutdown(
                    self.session,
                    (&mut self.wire as *mut Memory).cast(),
                    fragmented,
                )
            }
        }
        fn transfer(&mut self, data: &mut [u8], sending: bool) -> i64 {
            unsafe {
                neoclr_tls_transfer(
                    self.session,
                    (&mut self.wire as *mut Memory).cast(),
                    fragmented,
                    data.as_mut_ptr(),
                    data.len() as u32,
                    sending as i32,
                )
            }
        }
    }
    impl Drop for Endpoint {
        fn drop(&mut self) {
            unsafe {
                neoclr_tls_session_free(
                    self.session,
                    (&mut self.wire as *mut Memory).cast(),
                    fragmented,
                )
            }
        }
    }
    fn pump(a: &mut Endpoint, b: &mut Endpoint) {
        a.wire.incoming.extend(b.wire.outgoing.drain(..));
        b.wire.incoming.extend(a.wire.outgoing.drain(..));
    }
    fn pair(name: &str, protocol: &[u8]) -> (Endpoint, Endpoint) {
        pair_with_config(name, protocol, |_| {})
    }
    fn pair_with_config(
        name: &str,
        protocol: &[u8],
        configure: impl FnOnce(&mut ClientConfig),
    ) -> (Endpoint, Endpoint) {
        let cert = std::fs::read(std::env::var("NEOCLR_HTTPS_CERTIFICATE").unwrap()).unwrap();
        let key = std::fs::read(std::env::var("NEOCLR_HTTPS_PRIVATE_KEY").unwrap()).unwrap();
        unsafe {
            let mut server = std::ptr::null_mut();
            assert_eq!(
                neoclr_tls_server_config_create_pem(
                    cert.as_ptr(),
                    cert.len() as u32,
                    key.as_ptr(),
                    key.len() as u32,
                    protocol.as_ptr(),
                    protocol.len() as u32,
                    &mut server,
                ),
                0
            );
            let mut client = std::ptr::null_mut();
            assert_eq!(
                neoclr_tls_client_config_create_pem(
                    cert.as_ptr(),
                    cert.len() as u32,
                    protocol.as_ptr(),
                    protocol.len() as u32,
                    &mut client,
                ),
                0
            );
            assert!(!server.is_null() && !client.is_null());
            configure(Arc::get_mut(&mut *client.cast::<Arc<ClientConfig>>()).unwrap());
            let a = Endpoint::new(neoclr_tls_session_new(server));
            let b = Endpoint::new(neoclr_tls_client_session_new(
                client,
                name.as_ptr(),
                name.len() as u32,
            ));
            // Drop source buffers and config handles before authentication: sessions own them.
            neoclr_tls_config_free(server);
            neoclr_tls_client_config_free(client);
            (a, b)
        }
    }
    fn authenticate(a: &mut Endpoint, b: &mut Endpoint) -> bool {
        for _ in 0..20000 {
            let x = a.handshake();
            let y = b.handshake();
            pump(a, b);
            if x == -2 || y == -2 {
                return false;
            }
            if x == 0 && y == 0 {
                return true;
            }
        }
        panic!("handshake never settled");
    }
    fn deliver(a: &mut Endpoint, b: &mut Endpoint) {
        let mut payload = vec![53; 8192];
        let mut received = Vec::new();
        let mut acknowledged = false;
        for _ in 0..20000 {
            if !acknowledged {
                match a.transfer(&mut payload, true) {
                    -1 => (),
                    n if n == payload.len() as i64 => acknowledged = true,
                    n => panic!("unexpected write {n}"),
                }
            }
            pump(a, b);
            let mut scratch = [0; 79];
            match b.transfer(&mut scratch, false) {
                -1 => (),
                n if n > 0 => received.extend_from_slice(&scratch[..n as usize]),
                n => panic!("unexpected read {n}"),
            }
            if acknowledged && received.len() >= payload.len() {
                break;
            }
        }
        assert!(acknowledged);
        assert_eq!(received, payload);
        assert_eq!(b.transfer(&mut [0; 1], false), -1, "no plaintext replay");
    }
    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths"]
    fn shared_client_server_authentication_and_bidirectional_transport() {
        for protocol in [&b""[..], &b"http/1.1"[..]] {
            let (mut server, mut client) = pair("localhost", protocol);
            assert!(authenticate(&mut server, &mut client));
            for endpoint in [&server, &client] {
                let connection = &unsafe { &*endpoint.session.cast::<Session>() }.connection;
                assert_eq!(
                    connection.alpn_protocol(),
                    if protocol.is_empty() {
                        None
                    } else {
                        Some(protocol)
                    }
                );
            }
            deliver(&mut client, &mut server);
            deliver(&mut server, &mut client);
            unsafe { neoclr_tls_abort(client.session) };
            assert_eq!(client.handshake(), -2);
        }
    }
    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths"]
    fn shared_client_rejects_wrong_peer_name() {
        let (mut server, mut client) = pair("wrong.invalid", b"");
        assert!(!authenticate(&mut server, &mut client));
        assert_eq!(client.handshake(), -2);
    }
    #[test]
    fn configuration_rejects_invalid_inputs() {
        let null = std::ptr::null();
        unsafe {
            assert!(neoclr_tls_client_config_from_pem(null, 0, null, 0).is_null());
            assert!(neoclr_tls_client_config_from_pem(null, 1, null, 0).is_null());
            assert!(neoclr_tls_client_config_from_pem(b"invalid".as_ptr(), 7, null, 0).is_null());
            assert!(neoclr_tls_server_config_from_pem(null, 0, null, 0, null, 0).is_null());
            assert!(neoclr_tls_client_session_new(std::ptr::null_mut(), null, 0).is_null());
            assert!(bytes(b"x".as_ptr(), (LIMIT + 1) as u32).is_none());
        }
    }

    #[test]
    fn configuration_status_clears_output_on_failure() {
        use configuration::ConfigurationError as Error;
        let null = std::ptr::null();
        // A non-null sentinel detects stale handle publication, never dereferenced.
        let mut output = std::ptr::dangling_mut::<c_void>();
        unsafe {
            assert_eq!(
                neoclr_tls_client_config_create_pem(null, 1, null, 0, &mut output),
                Error::InvalidInput as i32
            );
            assert!(output.is_null());
            assert_eq!(
                neoclr_tls_client_config_create_pem(null, 0, null, 0, &mut output),
                Error::InvalidTrustAnchor as i32
            );
            assert!(output.is_null());
            assert_eq!(
                neoclr_tls_server_config_create_pem(null, 0, null, 0, null, 0, &mut output),
                Error::InvalidCertificate as i32
            );
            assert!(output.is_null());
            let protocol = [b'x'; 256];
            assert_eq!(
                neoclr_tls_client_config_create_pem(null, 0, protocol.as_ptr(), 256, &mut output),
                Error::InvalidProtocol as i32
            );
            assert!(output.is_null());
            assert_eq!(
                neoclr_tls_client_config_create_pem(null, 0, null, 0, std::ptr::null_mut()),
                Error::InvalidInput as i32
            );
            assert_eq!(
                neoclr_tls_server_config_create_pem(
                    null,
                    0,
                    null,
                    0,
                    null,
                    0,
                    std::ptr::null_mut()
                ),
                Error::InvalidInput as i32
            );
        }
    }

    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths and OpenSSL"]
    fn certificate_credentials_report_invalid_and_mismatched_keys() {
        use configuration::ConfigurationError as Error;
        let cert = std::fs::read(std::env::var("NEOCLR_HTTPS_CERTIFICATE").unwrap()).unwrap();
        let key = std::fs::read(std::env::var("NEOCLR_HTTPS_PRIVATE_KEY").unwrap()).unwrap();
        let other = std::process::Command::new("openssl")
            .args([
                "genpkey",
                "-algorithm",
                "RSA",
                "-pkeyopt",
                "rsa_keygen_bits:2048",
            ])
            .output()
            .expect("OpenSSL fixture key generation");
        assert!(
            other.status.success(),
            "OpenSSL fixture key generation failed"
        );
        for (credential, expected) in [
            (&b"not a key"[..], Error::InvalidPrivateKey),
            (&other.stdout[..], Error::KeyMismatch),
        ] {
            let mut output = std::ptr::dangling_mut::<c_void>();
            let result = unsafe {
                neoclr_tls_server_config_create_pem(
                    cert.as_ptr(),
                    cert.len() as u32,
                    credential.as_ptr(),
                    credential.len() as u32,
                    std::ptr::null(),
                    0,
                    &mut output,
                )
            };
            assert_eq!(result, expected as i32);
            assert!(output.is_null());
        }
        let invalid_der = b"-----BEGIN CERTIFICATE-----\nAQID\n-----END CERTIFICATE-----\n";
        assert!(matches!(
            configuration::server(invalid_der, &key, b""),
            Err(Error::InvalidCertificate)
        ));
        assert!(matches!(
            configuration::client(invalid_der, b""),
            Err(Error::InvalidTrustAnchor)
        ));
        // Loading credentials validates their structure/key match, not peer trust.
        assert!(configuration::server(&cert, &key, b"").is_ok());
    }

    #[derive(Debug)]
    struct FixedTime(u64);
    impl rustls::time_provider::TimeProvider for FixedTime {
        fn current_time(&self) -> Option<rustls::pki_types::UnixTime> {
            Some(rustls::pki_types::UnixTime::since_unix_epoch(
                std::time::Duration::from_secs(self.0),
            ))
        }
    }

    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths"]
    fn certificate_verification_rejects_untrusted_expired_and_future_peers() {
        for scenario in 0..3 {
            let (mut server, mut client) = pair_with_config("localhost", b"", |config| {
                match scenario {
                    0 => {
                        *config = ClientConfig::builder_with_provider(Arc::new(
                            rustls::crypto::ring::default_provider(),
                        ))
                        .with_safe_default_protocol_versions()
                        .unwrap()
                        .with_root_certificates(rustls::RootCertStore::empty())
                        .with_no_client_auth()
                    }
                    1 => config.time_provider = Arc::new(FixedTime(4102444800)), // 2100: fixture expired
                    _ => config.time_provider = Arc::new(FixedTime(0)), // 1970: fixture not yet valid
                }
            });
            assert!(!authenticate(&mut server, &mut client));
            assert_eq!(client.handshake(), -2, "rejection stays terminal");
            assert_eq!(client.transfer(&mut [0; 8], false), -2);
            assert_eq!(client.transfer(&mut [1; 8], true), -2);
            // Failure is session-local; fresh valid sessions still exchange data.
            let (mut valid_server, mut valid_client) = pair("localhost", b"");
            assert!(authenticate(&mut valid_server, &mut valid_client));
            deliver(&mut valid_client, &mut valid_server);
        }
    }

    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths"]
    fn graceful_shutdown_flushes_under_backpressure_and_preserves_reads() {
        for version in [&rustls::version::TLS12, &rustls::version::TLS13] {
            let (mut server, mut client) = pair_with_config("localhost", b"", |config| {
                let pem =
                    std::fs::read(std::env::var("NEOCLR_HTTPS_CERTIFICATE").unwrap()).unwrap();
                let mut roots = rustls::RootCertStore::empty();
                for cert in rustls_pemfile::certs(&mut &pem[..]) {
                    roots.add(cert.unwrap()).unwrap();
                }
                *config = ClientConfig::builder_with_provider(Arc::new(
                    rustls::crypto::ring::default_provider(),
                ))
                .with_protocol_versions(&[version])
                .unwrap()
                .with_root_certificates(roots)
                .with_no_client_auth();
            });
            assert_eq!(
                client.shutdown(),
                -3,
                "cannot close an unauthenticated session"
            );
            assert!(authenticate(&mut server, &mut client));
            deliver(&mut client, &mut server);
            let mut pending = 0;
            let mut flushed = false;
            let mut eof = false;
            for _ in 0..2000 {
                match client.shutdown() {
                    0 => flushed = true,
                    -1 => pending += 1,
                    status => panic!("unexpected shutdown {status}"),
                }
                pump(&mut client, &mut server);
                match server.transfer(&mut [0; 1], false) {
                    0 => eof = true,
                    -1 => (),
                    status => panic!("unexpected peer read {status}"),
                }
                if flushed && eof {
                    break;
                }
            }
            assert!(flushed && eof && pending > 0);
            let writes = client.wire.writes;
            assert_eq!(client.shutdown(), 0);
            assert_eq!(
                client.wire.writes, writes,
                "repeat shutdown must not send another alert"
            );
            assert_eq!(client.transfer(&mut [9], true), -3);
            // Closing local output does not discard authenticated peer response bytes.
            deliver(&mut server, &mut client);
        }
    }

    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths"]
    fn graceful_shutdown_rejects_pending_send_and_honors_abort() {
        let (mut server, mut client) = pair("localhost", b"");
        assert!(authenticate(&mut server, &mut client));
        let mut payload = vec![53; 8192];
        assert_eq!(client.transfer(&mut payload, true), -1);
        assert!(unsafe { &*client.session.cast::<Session>() }.pending > 0);
        assert_eq!(client.shutdown(), -3);
        // Resume the exact payload; shutdown rejection did not consume its acknowledgement.
        deliver(&mut client, &mut server);
        assert_eq!(client.shutdown(), -1);
        unsafe { neoclr_tls_abort(client.session) };
        assert_eq!(client.shutdown(), -2);
        assert_eq!(client.transfer(&mut [0], false), -2);
    }

    unsafe extern "C" fn failed_transport(_: *mut c_void, _: *mut u8, _: u32, _: i32) -> i64 {
        -2
    }
    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths"]
    fn graceful_shutdown_transport_failure_is_terminal() {
        let (mut server, mut client) = pair("localhost", b"");
        assert!(authenticate(&mut server, &mut client));
        assert_eq!(
            unsafe { neoclr_tls_shutdown(client.session, std::ptr::null_mut(), failed_transport) },
            -2
        );
        assert_eq!(client.shutdown(), -2);
        assert_eq!(client.handshake(), -2);
    }

    #[test]
    #[ignore = "run by HTTPS validation with ephemeral certificate/key paths"]
    fn checked_context_transports_after_configuration_release() {
        use super::resources::*;
        let cert = std::fs::read(std::env::var("NEOCLR_HTTPS_CERTIFICATE").unwrap()).unwrap();
        let key = std::fs::read(std::env::var("NEOCLR_HTTPS_PRIVATE_KEY").unwrap()).unwrap();
        let context = neoclr_tls_context_new(4);
        let mut server_config = 0;
        let mut client_config = 0;
        let mut server = 0;
        let mut client = 0;
        unsafe {
            assert_eq!(
                neoclr_tls_context_server_config(
                    context,
                    cert.as_ptr(),
                    cert.len() as u32,
                    key.as_ptr(),
                    key.len() as u32,
                    std::ptr::null(),
                    0,
                    &mut server_config
                ),
                0
            );
            assert_eq!(
                neoclr_tls_context_client_config(
                    context,
                    cert.as_ptr(),
                    cert.len() as u32,
                    std::ptr::null(),
                    0,
                    &mut client_config
                ),
                0
            );
            assert_eq!(
                neoclr_tls_context_session(
                    context,
                    server_config,
                    std::ptr::null(),
                    0,
                    &mut server
                ),
                0
            );
            assert_eq!(
                neoclr_tls_context_session(
                    context,
                    client_config,
                    b"localhost".as_ptr(),
                    9,
                    &mut client
                ),
                0
            );
            assert_eq!(neoclr_tls_context_release(context, server_config), 0);
            assert_eq!(neoclr_tls_context_release(context, client_config), 0);
        }
        drop(cert);
        drop(key);
        let make_wire = || Memory {
            incoming: VecDeque::new(),
            outgoing: vec![],
            writes: 0,
            eof: false,
        };
        let mut a = make_wire();
        let mut b = make_wire();
        let pump = |a: &mut Memory, b: &mut Memory| {
            a.incoming.extend(b.outgoing.drain(..));
            b.incoming.extend(a.outgoing.drain(..));
        };
        let mut authenticated = false;
        for _ in 0..20000 {
            let x = unsafe {
                neoclr_tls_context_handshake(
                    context,
                    server,
                    (&mut a as *mut Memory).cast(),
                    fragmented,
                )
            };
            let y = unsafe {
                neoclr_tls_context_handshake(
                    context,
                    client,
                    (&mut b as *mut Memory).cast(),
                    fragmented,
                )
            };
            assert!(matches!(x, 0 | -1) && matches!(y, 0 | -1));
            pump(&mut a, &mut b);
            if x == 0 && y == 0 {
                authenticated = true;
                break;
            }
        }
        assert!(authenticated);
        let mut payload = [37; 4096];
        let mut received = Vec::new();
        let mut acknowledged = false;
        for _ in 0..20000 {
            if !acknowledged {
                let result = unsafe {
                    neoclr_tls_context_transfer(
                        context,
                        client,
                        (&mut b as *mut Memory).cast(),
                        fragmented,
                        payload.as_mut_ptr(),
                        payload.len() as u32,
                        1,
                    )
                };
                assert!(result == -1 || result == payload.len() as i64);
                acknowledged = result > 0;
            }
            pump(&mut a, &mut b);
            let mut scratch = [0; 59];
            let result = unsafe {
                neoclr_tls_context_transfer(
                    context,
                    server,
                    (&mut a as *mut Memory).cast(),
                    fragmented,
                    scratch.as_mut_ptr(),
                    scratch.len() as u32,
                    0,
                )
            };
            if result > 0 {
                received.extend_from_slice(&scratch[..result as usize]);
            } else {
                assert_eq!(result, -1);
            }
            if acknowledged && received.len() == payload.len() {
                break;
            }
        }
        assert!(acknowledged);
        assert_eq!(received, payload);
        let mut closed = false;
        for _ in 0..2000 {
            let status = unsafe {
                neoclr_tls_context_shutdown(
                    context,
                    client,
                    (&mut b as *mut Memory).cast(),
                    fragmented,
                )
            };
            assert!(matches!(status, 0 | -1));
            pump(&mut a, &mut b);
            let mut scratch = [0];
            let read = unsafe {
                neoclr_tls_context_transfer(
                    context,
                    server,
                    (&mut a as *mut Memory).cast(),
                    fragmented,
                    scratch.as_mut_ptr(),
                    1,
                    0,
                )
            };
            assert!(matches!(read, 0 | -1));
            if status == 0 && read == 0 {
                closed = true;
                break;
            }
        }
        assert!(closed);
        unsafe {
            assert_eq!(neoclr_tls_context_release(context, client), 0);
            assert_eq!(
                neoclr_tls_context_handshake(
                    context,
                    client,
                    (&mut b as *mut Memory).cast(),
                    fragmented
                ),
                -18
            );
            assert_eq!(
                neoclr_tls_context_transfer(
                    context,
                    server,
                    (&mut a as *mut Memory).cast(),
                    fragmented,
                    std::ptr::null_mut(),
                    1,
                    0
                ),
                -10
            );
            neoclr_tls_context_free(context); // remaining server session is released without I/O
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

#[cfg(test)]
mod buffered_tests;
