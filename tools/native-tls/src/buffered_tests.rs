use crate::resources::{OwnedContext, PumpOperation, PumpProgress};
use std::collections::VecDeque;

struct Pair {
    context: OwnedContext,
    client: u64,
    server: u64,
    to_client: VecDeque<u8>,
    to_server: VecDeque<u8>,
}
impl Pair {
    fn new(name: &[u8]) -> Self {
        let cert = std::fs::read(std::env::var("NEOCLR_HTTPS_CERTIFICATE").unwrap()).unwrap();
        let key = std::fs::read(std::env::var("NEOCLR_HTTPS_PRIVATE_KEY").unwrap()).unwrap();
        let mut context = OwnedContext::new(4).unwrap();
        let server_config = context.server_config(&cert, &key, b"http/1.1").unwrap();
        let client_config = context.client_config(&cert, b"http/1.1").unwrap();
        let server = context.session(server_config, b"").unwrap();
        let client = context.session(client_config, name).unwrap();
        assert_eq!(context.release(server_config), 0);
        assert_eq!(context.release(client_config), 0);
        Self {
            context,
            client,
            server,
            to_client: VecDeque::new(),
            to_server: VecDeque::new(),
        }
    }
    fn poll(
        &mut self,
        client: bool,
        operation: PumpOperation,
        plaintext: &mut [u8],
    ) -> PumpProgress {
        let (id, incoming, outgoing) = if client {
            (self.client, &mut self.to_client, &mut self.to_server)
        } else {
            (self.server, &mut self.to_server, &mut self.to_client)
        };
        // Fragment both directions independently. Never discard unconsumed input.
        let input: Vec<_> = incoming.iter().copied().take(31).collect();
        let mut output = [0xa5; 17];
        let progress = self
            .context
            .pump(id, operation, &input, false, &mut output, plaintext);
        assert!(progress.consumed <= input.len());
        assert!(progress.produced <= output.len());
        incoming.drain(..progress.consumed);
        outgoing.extend(&output[..progress.produced]);
        assert!(output[progress.produced..].iter().all(|b| *b == 0xa5));
        progress
    }
    fn handshake(&mut self) {
        for _ in 0..10000 {
            let c = self.poll(true, PumpOperation::Handshake, &mut []).status;
            let s = self.poll(false, PumpOperation::Handshake, &mut []).status;
            assert!(c == -1 || c == 0, "client {c}");
            assert!(s == -1 || s == 0, "server {s}");
            if c == 0 && s == 0 {
                return;
            }
        }
        panic!("handshake did not finish");
    }
    fn transfer(&mut self, client: bool, data: &[u8]) {
        let mut sent = false;
        let mut received = Vec::new();
        let mut input = data.to_vec();
        let mut pending = false;
        for _ in 0..10000 {
            if !sent {
                let progress = self.poll(client, PumpOperation::Write, &mut input);
                if progress.status == -1 {
                    pending = true;
                } else {
                    assert_eq!(progress.status as usize, data.len());
                    sent = true;
                }
                assert_eq!(input, data);
            }
            let mut output = [0xa5; 23];
            let progress = self.poll(!client, PumpOperation::Read, &mut output);
            assert!(progress.status == -1 || progress.status > 0);
            if progress.status > 0 {
                received.extend_from_slice(&output[..progress.status as usize]);
                assert!(
                    output[progress.status as usize..]
                        .iter()
                        .all(|b| *b == 0xa5)
                );
            }
            if sent && received.len() == data.len() {
                assert!(pending, "must exercise pending-write retry");
                assert_eq!(received, data);
                return;
            }
        }
        panic!("transfer did not finish");
    }
}

#[test]
#[ignore = "run with disposable credentials using test-native-security.py"]
fn buffered_transport_authenticates_transfers_and_closes_under_backpressure() {
    let mut pair = Pair::new(b"localhost");
    let blocked = pair.context.pump(
        pair.client,
        PumpOperation::Handshake,
        &[],
        false,
        &mut [],
        &mut [],
    );
    assert_eq!(
        blocked,
        PumpProgress {
            status: -1,
            consumed: 0,
            produced: 0
        }
    );
    pair.handshake();
    pair.transfer(true, &vec![42; 4096]);
    pair.transfer(false, b"server response survives independent buffers");
    let mut closed = false;
    let mut eof = false;
    for _ in 0..10000 {
        if !closed {
            let progress = pair.poll(true, PumpOperation::Shutdown, &mut []);
            assert!(progress.status == 0 || progress.status == -1);
            closed = progress.status == 0;
        }
        let progress = pair.poll(false, PumpOperation::Read, &mut [0; 1]);
        assert!(progress.status == 0 || progress.status == -1);
        eof |= progress.status == 0;
        if closed && eof {
            break;
        }
    }
    assert!(closed && eof);
    assert_eq!(pair.poll(true, PumpOperation::Shutdown, &mut []).status, 0);
    assert_eq!(pair.poll(true, PumpOperation::Write, &mut [1]).status, -3);
    // A local close does not prevent receiving the peer's remaining data.
    pair.transfer(false, b"response after client half-close");
}

#[test]
#[ignore = "run with disposable credentials using test-native-security.py"]
fn buffered_transport_distinguishes_pending_from_eof_and_terminal_failure() {
    let mut pair = Pair::new(b"localhost");
    assert_eq!(
        pair.context
            .pump(
                pair.server,
                PumpOperation::Handshake,
                &[],
                false,
                &mut [0; 64],
                &mut []
            )
            .status,
        -1
    );
    assert_eq!(
        pair.context
            .pump(
                pair.server,
                PumpOperation::Handshake,
                &[],
                true,
                &mut [0; 64],
                &mut []
            )
            .status,
        -2
    );
    assert_eq!(
        pair.context
            .pump(
                pair.server,
                PumpOperation::Handshake,
                &[],
                false,
                &mut [0; 64],
                &mut []
            )
            .status,
        -2
    );
    // A completed handshake does not make a transport EOF an authenticated close.
    let mut pair = Pair::new(b"localhost");
    pair.handshake();
    pair.transfer(true, b"data before truncated connection");
    assert!(pair.to_server.is_empty());
    assert_eq!(
        pair.context
            .pump(
                pair.server,
                PumpOperation::Read,
                &[],
                true,
                &mut [0; 64],
                &mut [0; 1]
            )
            .status,
        -2
    );
    assert_eq!(
        pair.poll(false, PumpOperation::Read, &mut [0; 1]).status,
        -2
    );
    let mut pair = Pair::new(b"wrong.example");
    let mut rejected = false;
    for _ in 0..10000 {
        let c = pair.poll(true, PumpOperation::Handshake, &mut []).status;
        if c == -2 {
            rejected = true;
            break;
        }
        assert_eq!(c, -1);
        let s = pair.poll(false, PumpOperation::Handshake, &mut []).status;
        assert!(s == -1 || s == 0);
    }
    assert!(rejected);
    assert_eq!(
        pair.poll(true, PumpOperation::Handshake, &mut []).status,
        -2
    );
}

#[test]
fn buffered_transport_rejects_invalid_requests_without_touching_buffers() {
    let mut context = OwnedContext::new(1).unwrap();
    let mut output = [0xa5; 32];
    let mut plaintext = [0x5a; 8];
    assert_eq!(
        context
            .pump(
                99,
                PumpOperation::Read,
                &[],
                false,
                &mut output,
                &mut plaintext
            )
            .status,
        -18
    );
    assert_eq!(
        context
            .pump(
                99,
                PumpOperation::Handshake,
                &[],
                false,
                &mut output,
                &mut plaintext
            )
            .status,
        -10
    );
    assert_eq!(
        context
            .pump(
                99,
                PumpOperation::Read,
                &vec![0; 65537],
                false,
                &mut output,
                &mut plaintext
            )
            .status,
        -10
    );
    assert_eq!(output, [0xa5; 32]);
    assert_eq!(plaintext, [0x5a; 8]);
}

#[test]
fn buffered_c_boundary_rejects_overlap_null_and_oversized_ranges() {
    use crate::resources::*;
    let context = neoclr_tls_context_new(1);
    assert!(!context.is_null());
    let mut data = [0xa5; 32];
    let mut progress = NativePumpProgress {
        consumed: 123,
        produced: 456,
    };
    unsafe {
        for (input, size, operation, eof) in [
            (data.as_ptr(), 4, 2, 0),
            (std::ptr::null(), 1, 2, 0),
            (std::ptr::null(), 0, 0, 0),
            (std::ptr::null(), 0, 2, 2),
            (data.as_ptr(), 65537, 2, 0),
        ] {
            assert_eq!(
                neoclr_tls_context_pump_v1(
                    context,
                    99,
                    operation,
                    input,
                    size,
                    eof,
                    data.as_mut_ptr(),
                    32,
                    std::ptr::null_mut(),
                    0,
                    &mut progress
                ),
                -10
            );
            assert_eq!(
                progress,
                NativePumpProgress {
                    consumed: 123,
                    produced: 456
                }
            );
            assert_eq!(data, [0xa5; 32]);
        }
        // Zero-length null buffers are legal; an invalid session clears progress.
        assert_eq!(
            neoclr_tls_context_pump_v1(
                context,
                99,
                2,
                std::ptr::null(),
                0,
                0,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                0,
                &mut progress
            ),
            -18
        );
        assert_eq!(
            progress,
            NativePumpProgress {
                consumed: 0,
                produced: 0
            }
        );
        neoclr_tls_context_free(context);
    }
}
