//! Native tuple element names are descriptive signature metadata, not type identity.
use crate::{Fault, metadata::Type};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    hash::{Hash, Hasher},
    ops::Deref,
};

/// Ordered tuple elements with optional native names. Equality and hashing use only
/// element types; distinct signatures may retain different names for the same type.
#[derive(Debug, Clone)]
pub struct TupleSignature {
    elements: Vec<Type>,
    names: Vec<Option<String>>,
}

impl TupleSignature {
    /// Creates a named signature. Names are optional per position, not attributes.
    /// A nonempty name vector must match the element count. Names must be nonempty,
    /// NUL-free UTF-8 strings of at most 1024 bytes, with unique effective names.
    pub fn new(elements: Vec<Type>, names: Vec<Option<String>>) -> Result<Self, Fault> {
        let signature = Self { elements, names };
        signature.validate_names()?;
        Ok(signature)
    }

    /// Native names in position order. Empty means every element is unnamed.
    pub fn names(&self) -> &[Option<String>] {
        &self.names
    }

    /// Declared name, or the conventional positional name for an unnamed element.
    pub fn element_name(&self, index: usize) -> Option<String> {
        self.elements.get(index)?;
        Some(
            self.names
                .get(index)
                .and_then(Clone::clone)
                .unwrap_or_else(|| format!("Item{}", index + 1)),
        )
    }

    pub(crate) fn validate_names(&self) -> Result<(), Fault> {
        if !self.names.is_empty() && self.names.len() != self.elements.len() {
            return Err(Fault::new("tuple names must match element count"));
        }
        let mut seen = std::collections::HashSet::new();
        for (index, name) in self.names.iter().enumerate() {
            if name
                .as_ref()
                .is_some_and(|n| n.is_empty() || n.len() > 1024 || n.contains('\0'))
            {
                return Err(Fault::new(
                    "tuple element names must be nonempty, NUL-free and at most 1024 bytes",
                ));
            }
            if !seen.insert(name.clone().unwrap_or_else(|| format!("Item{}", index + 1))) {
                return Err(Fault::new("tuple element names must be unique"));
            }
        }
        Ok(())
    }

    pub(crate) fn map_types(
        &self,
        map: impl FnMut(&Type) -> Result<Type, Fault>,
    ) -> Result<Self, Fault> {
        Self::new(
            self.elements.iter().map(map).collect::<Result<_, _>>()?,
            self.names.clone(),
        )
    }
}

impl From<Vec<Type>> for TupleSignature {
    fn from(elements: Vec<Type>) -> Self {
        Self {
            elements,
            names: vec![],
        }
    }
}
impl Deref for TupleSignature {
    type Target = [Type];
    fn deref(&self) -> &[Type] {
        &self.elements
    }
}
impl<'a> IntoIterator for &'a TupleSignature {
    type Item = &'a Type;
    type IntoIter = std::slice::Iter<'a, Type>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}
impl PartialEq for TupleSignature {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}
impl Eq for TupleSignature {}
impl Hash for TupleSignature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.elements.hash(state);
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedShape {
    elements: Vec<Type>,
    names: Vec<Option<String>>,
}
impl Serialize for TupleSignature {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.names.is_empty() {
            self.elements.serialize(serializer)
        } else {
            NamedShape {
                elements: self.elements.clone(),
                names: self.names.clone(),
            }
            .serialize(serializer)
        }
    }
}
impl<'de> Deserialize<'de> for TupleSignature {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Shape {
            Named(NamedShape),
            Positional(Vec<Type>),
        }
        match Shape::deserialize(deserializer)? {
            Shape::Positional(elements) => Ok(elements.into()),
            Shape::Named(shape) => {
                Self::new(shape.elements, shape.names).map_err(serde::de::Error::custom)
            }
        }
    }
}
