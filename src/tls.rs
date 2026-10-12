//! Private TLS services. Native resources follow heap-owned zero-length byte tokens.
use crate::{Fault, ManagedHeap, Value, metadata::Type};
use neoclr_native_tls::resources::OwnedContext;
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Copy)]
pub(crate) enum Operation {
    ClientConfig,
    ServerConfig,
    Session,
    Release,
    Abort,
}

#[derive(Debug, Default)]
pub(crate) struct Resources {
    context: Option<OwnedContext>,
    owners: BTreeMap<usize, u64>,
}
impl Resources {
    fn context(&mut self) -> &mut OwnedContext {
        self.context
            .get_or_insert_with(|| OwnedContext::new(1024).expect("valid TLS quota"))
    }
    /// Called after successful tracing only. No I/O, callbacks or managed allocation.
    pub(crate) fn reap(&mut self, live: &HashSet<usize>) {
        if let Some(context) = &mut self.context {
            self.owners.retain(|owner, id| {
                if live.contains(owner) {
                    true
                } else {
                    context.release(*id);
                    false
                }
            });
        }
    }
    fn release(&mut self, owner: usize) -> i32 {
        match self.owners.remove(&owner) {
            Some(id) => self.context().release(id),
            None => 0,
        }
    }
}
fn array(value: &Value, heap: &ManagedHeap) -> Result<Vec<u8>, Fault> {
    let Value::ObjectReference(object) = value else {
        return Err(Fault::new("TLS requires managed byte arrays"));
    };
    heap.check_reference_owner(&object.reference)?;
    let Value::Array {
        element: Type::Byte,
        elements,
    } = object.reference.read()?
    else {
        return Err(Fault::new("TLS requires managed byte arrays"));
    };
    elements
        .into_iter()
        .map(|value| match value {
            Value::Byte(byte) => Ok(byte),
            _ => Err(Fault::new("TLS requires initialized byte input")),
        })
        .collect()
}
fn owner(value: &Value, heap: &ManagedHeap) -> Result<usize, Fault> {
    let Value::ObjectReference(object) = value else {
        return Err(Fault::new("TLS requires a managed owner token"));
    };
    heap.check_reference_owner(&object.reference)?;
    if object.reference.target() != &Type::Array(Box::new(Type::Byte))
        || object.reference.array_length()? != 0
    {
        return Err(Fault::new("TLS owner must be an empty byte array"));
    }
    Ok(object.allocation_id())
}
impl Operation {
    pub(crate) fn invoke(self, args: &[Value], heap: &mut ManagedHeap) -> Result<Value, Fault> {
        let id = owner(
            args.first()
                .ok_or_else(|| Fault::new("missing TLS owner"))?,
            heap,
        )?;
        match self {
            Self::Release => return Ok(Value::Int32(heap.tls.release(id))),
            Self::Abort => {
                let Some(handle) = heap.tls.owners.get(&id).copied() else {
                    return Ok(Value::Int32(-18));
                };
                return Ok(Value::Int32(heap.tls.context().abort(handle)));
            }
            _ => (),
        }
        if heap.tls.owners.contains_key(&id) {
            return Err(Fault::new("TLS owner already has a resource"));
        }
        let created = match (self, args) {
            (Self::ClientConfig, [_, roots, alpn]) => {
                let roots = array(roots, heap)?;
                let alpn = array(alpn, heap)?;
                heap.tls.context().client_config(&roots, &alpn)
            }
            (Self::ServerConfig, [_, cert, key, alpn]) => {
                let cert = array(cert, heap)?;
                let key = array(key, heap)?;
                let alpn = array(alpn, heap)?;
                heap.tls.context().server_config(&cert, &key, &alpn)
            }
            (Self::Session, [_, config, name]) => {
                let config = owner(config, heap)?;
                let name = array(name, heap)?;
                let Some(handle) = heap.tls.owners.get(&config).copied() else {
                    return Ok(Value::Int32(-18));
                };
                heap.tls.context().session(handle, &name)
            }
            _ => return Err(Fault::new("invalid TLS service arguments")),
        };
        Ok(Value::Int32(match created {
            Ok(handle) => {
                heap.tls.owners.insert(id, handle);
                0
            }
            Err(status) => status,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bytes(heap: &mut ManagedHeap, bytes: &[u8]) -> Value {
        let id = heap
            .allocate(Value::Array {
                element: Type::Byte,
                elements: bytes.iter().copied().map(Value::Byte).collect(),
            })
            .unwrap();
        Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(id).unwrap(),
            view: None,
        })
    }
    #[test]
    fn invalid_and_foreign_owner_tokens_fault_without_resources() {
        let mut heap = ManagedHeap::default();
        let mut foreign = ManagedHeap::default();
        let other = bytes(&mut foreign, &[]);
        let owner = bytes(&mut heap, &[]);
        let nonempty = bytes(&mut heap, &[0]);
        assert!(Operation::Release.invoke(&[other], &mut heap).is_err());
        assert!(Operation::Release.invoke(&[nonempty], &mut heap).is_err());
        assert_eq!(
            Operation::Release
                .invoke(&[owner.clone()], &mut heap)
                .unwrap(),
            Value::Int32(0)
        );
        assert_eq!(
            Operation::Abort.invoke(&[owner], &mut heap).unwrap(),
            Value::Int32(-18)
        );
        assert!(heap.tls.context.is_none());
    }
    #[test]
    fn failed_creation_publishes_no_resource_and_rejects_uninitialized_input() {
        let mut heap = ManagedHeap::default();
        let owner = bytes(&mut heap, &[]);
        let empty = bytes(&mut heap, &[]);
        assert_eq!(
            Operation::ClientConfig
                .invoke(&[owner.clone(), empty.clone(), empty.clone()], &mut heap)
                .unwrap(),
            Value::Int32(-14)
        );
        assert!(heap.tls.owners.is_empty());
        let input_id = heap
            .allocate(Value::Array {
                element: Type::Byte,
                elements: vec![Value::Uninitialized(Type::Byte)],
            })
            .unwrap();
        let input = Value::ObjectReference(crate::value::ObjectReference {
            reference: heap.address(input_id).unwrap(),
            view: None,
        });
        assert!(
            Operation::ClientConfig
                .invoke(&[owner, input, empty], &mut heap)
                .unwrap_err()
                .message
                .contains("initialized")
        );
        assert!(heap.tls.owners.is_empty());
    }
    #[test]
    #[ignore = "run by test-native-security.py with ephemeral certificate paths"]
    fn collection_releases_resources_without_rooting_their_owners() {
        let mut heap = ManagedHeap::default();
        let cert = std::fs::read(std::env::var("NEOCLR_HTTPS_CERTIFICATE").unwrap()).unwrap();
        let key = std::fs::read(std::env::var("NEOCLR_HTTPS_PRIVATE_KEY").unwrap()).unwrap();
        let owner = bytes(&mut heap, &[]);
        let cert = bytes(&mut heap, &cert);
        let key = bytes(&mut heap, &key);
        let empty = bytes(&mut heap, &[]);
        let args = [owner.clone(), cert, key, empty.clone()];
        assert_eq!(
            Operation::ServerConfig.invoke(&args, &mut heap).unwrap(),
            Value::Int32(0)
        );
        assert!(
            Operation::ServerConfig.invoke(&args, &mut heap).is_err(),
            "cannot replace a live owner resource"
        );
        let session = bytes(&mut heap, &[]);
        assert_eq!(
            Operation::Session
                .invoke(&[session.clone(), owner, empty], &mut heap)
                .unwrap(),
            Value::Int32(0)
        );
        let roots = heap.value_roots(&session).unwrap();
        assert!(
            heap.collect(vec![usize::MAX], crate::CollectionReason::ExplicitRequest)
                .is_err()
        );
        assert_eq!(
            heap.tls.owners.len(),
            2,
            "failed tracing must not reclaim resources"
        );
        heap.collect(roots, crate::CollectionReason::ExplicitRequest)
            .unwrap();
        assert_eq!(heap.tls.owners.len(), 1);
        assert_eq!(heap.tls.context().resource_count(), 1);
        assert_eq!(
            Operation::Abort
                .invoke(&[session.clone()], &mut heap)
                .unwrap(),
            Value::Int32(0)
        );
        heap.collect(vec![], crate::CollectionReason::ExplicitRequest)
            .unwrap();
        assert!(heap.tls.owners.is_empty());
        assert_eq!(heap.tls.context().resource_count(), 0);
        assert!(
            Operation::Release.invoke(&[session], &mut heap).is_err(),
            "collected reference is invalid"
        );
    }
    #[test]
    #[ignore = "run by test-native-security.py with ephemeral certificate paths"]
    fn explicit_release_is_idempotent_while_owner_is_live() {
        let mut heap = ManagedHeap::default();
        let cert = std::fs::read(std::env::var("NEOCLR_HTTPS_CERTIFICATE").unwrap()).unwrap();
        let owner = bytes(&mut heap, &[]);
        let cert = bytes(&mut heap, &cert);
        let empty = bytes(&mut heap, &[]);
        assert_eq!(
            Operation::ClientConfig
                .invoke(&[owner.clone(), cert, empty], &mut heap)
                .unwrap(),
            Value::Int32(0)
        );
        for _ in 0..2 {
            assert_eq!(
                Operation::Release
                    .invoke(&[owner.clone()], &mut heap)
                    .unwrap(),
                Value::Int32(0)
            );
        }
        assert_eq!(heap.tls.context().resource_count(), 0);
    }
}
