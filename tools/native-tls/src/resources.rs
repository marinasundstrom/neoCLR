//! Checked resource IDs for future managed adapters. Only the host owns Context.
//! No guest integer is interpreted as a native pointer, and no callbacks are retained.
use super::*;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

const INVALID_INPUT: i32 = -10;
const RESOURCE_LIMIT: i32 = -17;
const INVALID_HANDLE: i32 = -18;
const INVALID_NAME: i32 = -19;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

enum Resource {
    Server(Arc<ServerConfig>),
    Client(Arc<ClientConfig>),
    Session(Box<Session>),
}
struct Context {
    resources: BTreeMap<u64, Resource>,
    limit: usize,
}
impl Context {
    fn insert(&mut self, resource: Resource) -> Result<u64, i32> {
        if self.resources.len() >= self.limit {
            return Err(RESOURCE_LIMIT);
        }
        // Never reuse an ID, even across contexts. Exhaustion fails closed.
        let id = NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| RESOURCE_LIMIT)?;
        self.resources.insert(id, resource);
        Ok(id)
    }
    fn session(&mut self, id: u64) -> Result<*mut c_void, i32> {
        match self.resources.get_mut(&id) {
            Some(Resource::Session(session)) => Ok((&mut **session as *mut Session).cast()),
            _ => Err(INVALID_HANDLE),
        }
    }
}
unsafe fn context<'a>(pointer: *mut c_void) -> Result<&'a mut Context, i32> {
    if pointer.is_null() {
        return Err(INVALID_INPUT);
    }
    Ok(unsafe { &mut *pointer.cast::<Context>() })
}
unsafe fn input<'a>(pointer: *const u8, count: u32) -> Result<&'a [u8], i32> {
    unsafe { bytes(pointer, count) }.ok_or(INVALID_INPUT)
}
unsafe fn publish(output: *mut u64, create: impl FnOnce() -> Result<u64, i32>) -> i32 {
    if output.is_null() {
        return INVALID_INPUT;
    }
    unsafe { *output = 0 };
    match create() {
        Ok(id) => {
            unsafe { *output = id };
            0
        }
        Err(error) => error,
    }
}

/// Host-owned, serialized context; limit is 1..65536 live resources.
#[unsafe(no_mangle)]
pub extern "C" fn neoclr_tls_context_new(limit: u32) -> *mut c_void {
    if limit == 0 || limit > 65536 {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(Context {
        resources: BTreeMap::new(),
        limit: limit as usize,
    }))
    .cast()
}
/// Deterministic abortive cleanup. Performs no I/O and does not retain callbacks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_free(pointer: *mut c_void) {
    if !pointer.is_null() {
        drop(unsafe { Box::from_raw(pointer.cast::<Context>()) });
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_release(pointer: *mut c_void, id: u64) -> i32 {
    let Ok(context) = (unsafe { context(pointer) }) else {
        return INVALID_INPUT;
    };
    if context.resources.remove(&id).is_some() {
        0
    } else {
        INVALID_HANDLE
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_server_config(
    pointer: *mut c_void,
    cert: *const u8,
    cert_count: u32,
    key: *const u8,
    key_count: u32,
    alpn: *const u8,
    alpn_count: u32,
    output: *mut u64,
) -> i32 {
    unsafe {
        publish(output, || {
            let context = context(pointer)?;
            if context.resources.len() >= context.limit {
                return Err(RESOURCE_LIMIT);
            }
            let config = configuration::server(
                input(cert, cert_count)?,
                input(key, key_count)?,
                input(alpn, alpn_count)?,
            )
            .map_err(|error| error as i32)?;
            context.insert(Resource::Server(Arc::new(config)))
        })
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_client_config(
    pointer: *mut c_void,
    roots: *const u8,
    roots_count: u32,
    alpn: *const u8,
    alpn_count: u32,
    output: *mut u64,
) -> i32 {
    unsafe {
        publish(output, || {
            let context = context(pointer)?;
            if context.resources.len() >= context.limit {
                return Err(RESOURCE_LIMIT);
            }
            let config =
                configuration::client(input(roots, roots_count)?, input(alpn, alpn_count)?)
                    .map_err(|error| error as i32)?;
            context.insert(Resource::Client(Arc::new(config)))
        })
    }
}
/// Client configs require the verified UTF-8 peer name. Server configs require empty name.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_session(
    pointer: *mut c_void,
    config: u64,
    name: *const u8,
    name_count: u32,
    output: *mut u64,
) -> i32 {
    unsafe {
        publish(output, || {
            let context = context(pointer)?;
            if context.resources.len() >= context.limit {
                return Err(RESOURCE_LIMIT);
            }
            let name = input(name, name_count)?;
            let connection = match context.resources.get(&config) {
                Some(Resource::Server(config)) => {
                    if !name.is_empty() {
                        return Err(INVALID_NAME);
                    }
                    ServerConnection::new(config.clone()).map(Connection::Server)
                }
                Some(Resource::Client(config)) => {
                    let name = std::str::from_utf8(name).map_err(|_| INVALID_NAME)?;
                    let name = rustls::pki_types::ServerName::try_from(name.to_owned())
                        .map_err(|_| INVALID_NAME)?;
                    ClientConnection::new(config.clone(), name).map(Connection::Client)
                }
                _ => return Err(INVALID_HANDLE),
            }
            .map_err(|_| configuration::ConfigurationError::ProviderFailure as i32)?;
            // Reuse the existing session constructor and its bounds/state defaults.
            let session = Box::new(make_session(connection));
            context.insert(Resource::Session(session))
        })
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_handshake(
    pointer: *mut c_void,
    id: u64,
    io: *mut c_void,
    callback: Transfer,
) -> i32 {
    match unsafe { context(pointer) }.and_then(|context| context.session(id)) {
        Ok(session) => unsafe { neoclr_tls_handshake(session, io, callback) },
        Err(error) => error,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_shutdown(
    pointer: *mut c_void,
    id: u64,
    io: *mut c_void,
    callback: Transfer,
) -> i32 {
    match unsafe { context(pointer) }.and_then(|context| context.session(id)) {
        Ok(session) => unsafe { neoclr_tls_shutdown(session, io, callback) },
        Err(error) => error,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_abort(pointer: *mut c_void, id: u64) -> i32 {
    match unsafe { context(pointer) }.and_then(|context| context.session(id)) {
        Ok(session) => {
            unsafe { neoclr_tls_abort(session) };
            0
        }
        Err(error) => error,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn neoclr_tls_context_transfer(
    pointer: *mut c_void,
    id: u64,
    io: *mut c_void,
    callback: Transfer,
    data: *mut u8,
    count: u32,
    sending: i32,
) -> i64 {
    if count as usize > LIMIT || (count != 0 && data.is_null()) || !matches!(sending, 0 | 1) {
        return INVALID_INPUT as i64;
    }
    let data = if data.is_null() {
        std::ptr::NonNull::<u8>::dangling().as_ptr()
    } else {
        data
    };
    match unsafe { context(pointer) }.and_then(|context| context.session(id)) {
        Ok(session) => unsafe { neoclr_tls_transfer(session, io, callback, data, count, sending) },
        Err(error) => error as i64,
    }
}

/// Safe Rust owner for the interpreter adapter. Not a managed/public platform API.
/// Mutable access serializes provider calls; dropping this value releases all resources.
pub struct OwnedContext(Box<Context>);
impl std::fmt::Debug for OwnedContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsContext")
            .field("resources", &self.0.resources.len())
            .finish()
    }
}
impl OwnedContext {
    pub fn new(limit: u32) -> Option<Self> {
        (limit > 0 && limit <= 65536).then(|| {
            Self(Box::new(Context {
                resources: BTreeMap::new(),
                limit: limit as usize,
            }))
        })
    }
    pub fn resource_count(&self) -> usize {
        self.0.resources.len()
    }
    fn pointer(&mut self) -> *mut c_void {
        (&mut *self.0 as *mut Context).cast()
    }
    pub fn client_config(&mut self, roots: &[u8], alpn: &[u8]) -> Result<u64, i32> {
        if roots.len() > LIMIT || alpn.len() > LIMIT {
            return Err(INVALID_INPUT);
        }
        let mut id = 0;
        let status = unsafe {
            neoclr_tls_context_client_config(
                self.pointer(),
                roots.as_ptr(),
                roots.len() as u32,
                alpn.as_ptr(),
                alpn.len() as u32,
                &mut id,
            )
        };
        if status == 0 { Ok(id) } else { Err(status) }
    }
    pub fn server_config(&mut self, cert: &[u8], key: &[u8], alpn: &[u8]) -> Result<u64, i32> {
        if cert.len() > LIMIT || key.len() > LIMIT || alpn.len() > LIMIT {
            return Err(INVALID_INPUT);
        }
        let mut id = 0;
        let status = unsafe {
            neoclr_tls_context_server_config(
                self.pointer(),
                cert.as_ptr(),
                cert.len() as u32,
                key.as_ptr(),
                key.len() as u32,
                alpn.as_ptr(),
                alpn.len() as u32,
                &mut id,
            )
        };
        if status == 0 { Ok(id) } else { Err(status) }
    }
    pub fn session(&mut self, config: u64, name: &[u8]) -> Result<u64, i32> {
        if name.len() > LIMIT {
            return Err(INVALID_INPUT);
        }
        let mut id = 0;
        let status = unsafe {
            neoclr_tls_context_session(
                self.pointer(),
                config,
                name.as_ptr(),
                name.len() as u32,
                &mut id,
            )
        };
        if status == 0 { Ok(id) } else { Err(status) }
    }
    pub fn release(&mut self, id: u64) -> i32 {
        unsafe { neoclr_tls_context_release(self.pointer(), id) }
    }
    pub fn abort(&mut self, id: u64) -> i32 {
        unsafe { neoclr_tls_context_abort(self.pointer(), id) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn client() -> Arc<ClientConfig> {
        Arc::new(
            ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_root_certificates(rustls::RootCertStore::empty())
                .with_no_client_auth(),
        )
    }
    #[test]
    fn handles_reject_stale_foreign_and_wrong_kinds_and_enforce_quota() {
        unsafe {
            assert!(neoclr_tls_context_new(0).is_null());
            assert!(neoclr_tls_context_new(65537).is_null());
            let a = neoclr_tls_context_new(1);
            let b = neoclr_tls_context_new(1);
            let id = context(a)
                .unwrap()
                .insert(Resource::Client(client()))
                .unwrap();
            assert_eq!(
                context(a).unwrap().insert(Resource::Client(client())),
                Err(RESOURCE_LIMIT)
            );
            assert_eq!(context(a).unwrap().session(id), Err(INVALID_HANDLE));
            assert_eq!(neoclr_tls_context_release(b, id), INVALID_HANDLE);
            assert_eq!(neoclr_tls_context_release(a, id), 0);
            assert_eq!(neoclr_tls_context_release(a, id), INVALID_HANDLE);
            let next = context(a)
                .unwrap()
                .insert(Resource::Client(client()))
                .unwrap();
            assert_ne!(id, next);
            assert_eq!(neoclr_tls_context_abort(a, id), INVALID_HANDLE);
            assert_eq!(
                neoclr_tls_context_abort(std::ptr::null_mut(), next),
                INVALID_INPUT
            );
            neoclr_tls_context_free(a);
            neoclr_tls_context_free(b);
            neoclr_tls_context_free(std::ptr::null_mut());
        }
    }
    #[test]
    fn session_retains_config_until_context_teardown() {
        unsafe {
            let pointer = neoclr_tls_context_new(2);
            let config = client();
            let weak = Arc::downgrade(&config);
            let id = context(pointer)
                .unwrap()
                .insert(Resource::Client(config))
                .unwrap();
            let mut session = 99;
            assert_eq!(
                neoclr_tls_context_session(pointer, id, b"localhost".as_ptr(), 9, &mut session),
                0
            );
            assert_eq!(neoclr_tls_context_release(pointer, id), 0);
            assert!(weak.upgrade().is_some());
            let mut other = 99;
            assert_eq!(
                neoclr_tls_context_session(pointer, session, b"localhost".as_ptr(), 9, &mut other),
                INVALID_HANDLE
            );
            assert_eq!(other, 0);
            assert_eq!(neoclr_tls_context_abort(pointer, session), 0);
            neoclr_tls_context_free(pointer);
            assert!(
                weak.upgrade().is_none(),
                "context teardown releases retained configuration"
            );
        }
    }
    #[test]
    fn failed_creations_clear_ids_and_do_not_consume_capacity() {
        unsafe {
            let pointer = neoclr_tls_context_new(2);
            let null = std::ptr::null();
            let mut id = 99;
            assert_eq!(
                neoclr_tls_context_client_config(pointer, null, 1, null, 0, &mut id),
                INVALID_INPUT
            );
            assert_eq!(id, 0);
            assert_eq!(
                neoclr_tls_context_server_config(pointer, null, 0, null, 0, null, 0, &mut id),
                -11
            );
            assert!(context(pointer).unwrap().resources.is_empty());
            let config = context(pointer)
                .unwrap()
                .insert(Resource::Client(client()))
                .unwrap();
            for name in [&b""[..], &b"bad name"[..], &b"\xff"[..]] {
                id = 99;
                assert_eq!(
                    neoclr_tls_context_session(
                        pointer,
                        config,
                        name.as_ptr(),
                        name.len() as u32,
                        &mut id
                    ),
                    INVALID_NAME
                );
                assert_eq!(id, 0);
            }
            assert_eq!(context(pointer).unwrap().resources.len(), 1);
            neoclr_tls_context_free(pointer);
        }
    }
}
