//! Execution-scoped candidate indexes for an immutably borrowed module.
//! Indexes only narrow the search; signature, identity and access checks stay in
//! their existing callers. Mutable metadata outside execution uses linear lookup.
use crate::metadata::Module;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

struct Index {
    module: usize,
    types: HashMap<String, Rc<[usize]>>,
    functions: HashMap<String, Rc<[usize]>>,
}

thread_local! {
    static ACTIVE: RefCell<Option<Rc<Index>>> = const { RefCell::new(None) };
}

fn grouped(names: impl Iterator<Item = String>) -> HashMap<String, Rc<[usize]>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, name) in names.enumerate() {
        groups.entry(name).or_default().push(index);
    }
    groups
        .into_iter()
        .map(|(name, indexes)| (name, indexes.into()))
        .collect()
}

pub(crate) fn with_module<R>(module: &Module, operation: impl FnOnce() -> R) -> R {
    let address = module as *const Module as usize;
    if ACTIVE.with(|active| {
        active
            .borrow()
            .as_ref()
            .is_some_and(|index| index.module == address)
    }) {
        return operation();
    }
    // The immutable borrow lasts through operation, including unwinding. No
    // address is dereferenced or retained after the outermost scope returns.
    let index = Rc::new(Index {
        module: address,
        types: grouped(
            module
                .types
                .iter()
                .map(|definition| definition.name.clone()),
        ),
        functions: grouped(
            module
                .functions
                .iter()
                .map(|definition| definition.name.clone()),
        ),
    });
    struct Restore(Option<Rc<Index>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            ACTIVE.with(|active| *active.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(ACTIVE.with(|active| active.replace(Some(index))));
    operation()
}

fn candidates(module: &Module, name: &str, types: bool) -> Option<Rc<[usize]>> {
    ACTIVE.with(|active| {
        let active = active.borrow();
        let index = active.as_ref()?;
        if index.module != module as *const Module as usize {
            return None;
        }
        let groups = if types {
            &index.types
        } else {
            &index.functions
        };
        Some(groups.get(name).cloned().unwrap_or_default())
    })
}

pub(crate) fn types(module: &Module, name: &str) -> Option<Rc<[usize]>> {
    candidates(module, name, true)
}

pub(crate) fn functions(module: &Module, name: &str) -> Option<Rc<[usize]>> {
    candidates(module, name, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assemble,
        metadata::{FunctionRef, Type},
    };

    fn module(value: i32) -> Module {
        assemble(&format!(
            ".module Example\n.function Main() -> Int32\nldc.i4 {value}\nret\n.end\n.entry Main\n"
        ))
        .unwrap()
    }

    #[test]
    fn scopes_restore_after_nested_modules_and_unwinding() {
        let first = module(1);
        let second = module(2);
        assert!(functions(&first, "Main").is_none());
        with_module(&first, || {
            assert_eq!(&*functions(&first, "Main").unwrap(), &[0]);
            assert!(functions(&second, "Main").is_none());
            let caught = std::panic::catch_unwind(|| {
                with_module(&second, || {
                    assert!(functions(&first, "Main").is_none());
                    panic!("unwind nested lookup");
                })
            });
            assert!(caught.is_err());
            assert_eq!(&*functions(&first, "Main").unwrap(), &[0]);
        });
        assert!(functions(&first, "Main").is_none());
    }

    #[test]
    fn mutations_between_scopes_and_duplicate_candidates_keep_linear_semantics() {
        let mut module = module(1);
        let target = FunctionRef {
            definition: None,
            name: "Main".into(),
            owner: None,
            instance: false,
            generic_arguments: vec![],
            parameters: vec![],
        };
        with_module(&module, || {
            assert_eq!(
                crate::vm::resolve(&module, &target).unwrap().returns,
                Type::Int32
            )
        });
        module.functions.push(module.functions[0].clone());
        let expected = crate::vm::resolve(&module, &target).unwrap_err();
        with_module(&module, || {
            assert_eq!(&*functions(&module, "Main").unwrap(), &[0, 1]);
            assert_eq!(
                crate::vm::resolve(&module, &target).unwrap_err().message,
                expected.message
            );
            assert!(functions(&module, "Absent").unwrap().is_empty());
        });
    }
    #[test]
    fn type_arity_missing_types_and_threads_match_uncached_lookup() {
        let module = assemble(".module Types\n.type Cell\n.end\n.type Cell<T>\n.end\n").unwrap();
        let types = [
            Type::from_name("Cell"),
            crate::assembler::parse_type("Cell<Int32>").unwrap(),
            crate::assembler::parse_type("Cell<Int32,String>").unwrap(),
            Type::from_name("Absent"),
        ];
        let expected: Vec<_> = types
            .iter()
            .map(|ty| {
                module
                    .type_definition(ty)
                    .map(|definition| serde_json::to_value(definition).unwrap())
            })
            .collect();
        with_module(&module, || {
            for (ty, expected) in types.iter().zip(&expected) {
                assert_eq!(
                    module
                        .type_definition(ty)
                        .map(|definition| serde_json::to_value(definition).unwrap()),
                    *expected
                );
            }
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        assert!(super::types(&module, "Cell").is_none());
                        with_module(&module, || {
                            assert_eq!(&*super::types(&module, "Cell").unwrap(), &[0, 1])
                        });
                    })
                    .join()
                    .unwrap();
            });
        });
    }
}
