//! Source unions lower to ordinary carrier constructors and separate case types.
use super::*;

pub(super) struct Union {
    pub name: Token,
    pub generic_parameters: Vec<String>,
    pub variants: Vec<Ty>,
    pub inline: bool,
}

pub(super) fn parse(parser: &mut Parser, source: &mut Source) -> Result<(), Fault> {
    let name = parser.name()?;
    let generic_parameters = parser.type_parameters()?;
    let saved_generics = std::mem::replace(&mut parser.generic_names, generic_parameters.clone());
    let mut variants = Vec::new();
    parser.newlines();
    let inline = !parser.eat("(");
    if inline {
        if !generic_parameters.is_empty() {
            return Err(name.error(
                "generic inline cases are not yet supported; declare independent generic records",
            ));
        }
        parser.expect("{")?;
        parser.lines();
        while !parser.at("}") && !parser.at("") {
            parser.expect("case")?;
            let mut case = parser.name()?;
            case.text = format!("{}.{}", name.text, case.text);
            let fields = if parser.at("(") {
                parser.fields(false)?
            } else {
                Vec::new()
            };
            variants.push(Ty::Record(case.text.clone()));
            source.records.push(Record {
                generic_parameters: Vec::new(),
                generic_constraints: Vec::new(),
                name: case,
                fields,
                is_class: false,
                is_abstract: false,
                field_initializers: Vec::new(),
                base: None,
                inherited_fields: 0,
                implements: Vec::new(),
                methods: Vec::new(),
            });
            if variants.len() > 1024 {
                return Err(name.error("union case limit exceeded"));
            }
            parser.end_statement()?;
        }
        parser.expect("}")?;
    } else {
        parser.newlines();
        if !parser.at(")") {
            loop {
                variants.push(parser.ty()?);
                if variants.len() > 1024 {
                    return Err(name.error("union case limit exceeded"));
                }
                parser.newlines();
                if !parser.eat("|") {
                    break;
                }
                parser.newlines();
            }
        }
        parser.expect(")")?;
        parser.end_statement()?;
    }
    parser.generic_names = saved_generics;
    source.unions.push(Union {
        name,
        generic_parameters,
        variants,
        inline,
    });
    Ok(())
}

pub(super) fn validate(source: &Source) -> Result<(), Fault> {
    for union in &source.unions {
        if union.variants.is_empty() {
            return Err(union.name.error("union requires at least one variant"));
        }
        let mut names = std::collections::HashSet::new();
        for variant in &union.variants {
            let Ty::Record(name) = variant else {
                return Err(union
                    .name
                    .error("union variants require declared source value types"));
            };
            let metadata = crate::assembler::parse_type(name)?;
            let name = metadata
                .definition_name()
                .ok_or_else(|| union.name.error("union case requires a named source type"))?;
            let arity = match &metadata {
                crate::metadata::Type::Constructed { arguments, .. } => arguments.len(),
                _ => 0,
            };
            let declared_arity = source
                .records
                .iter()
                .find(|r| r.name.text == name)
                .map(|r| r.generic_parameters.len())
                .or_else(|| {
                    source
                        .unions
                        .iter()
                        .find(|u| u.name.text == name)
                        .map(|u| u.generic_parameters.len())
                });
            if declared_arity.is_some_and(|count| count != arity) {
                return Err(union
                    .name
                    .error("union case generic argument count mismatch"));
            }
            if name == union.name.text
                || !source.records.iter().any(|r| r.name.text == name)
                    && !source.unions.iter().any(|u| u.name.text == name)
            {
                return Err(union
                    .name
                    .error("union variant must name a different declared source type"));
            }
            if source
                .records
                .iter()
                .any(|r| r.name.text == name && r.is_abstract)
            {
                return Err(union
                    .name
                    .error("union variants require concrete source value types"));
            }
            if !names.insert(name.rsplit('.').next().unwrap().to_owned()) {
                return Err(union.name.error("duplicate or ambiguous union case name"));
            }
        }
    }
    Ok(())
}

impl Union {
    pub fn variants_for(&self, owner: &Ty) -> Result<Vec<Ty>, Fault> {
        let metadata = crate::assembler::parse_type(&owner.il())?;
        let arguments = match &metadata {
            crate::metadata::Type::Constructed { arguments, .. } => arguments.as_slice(),
            _ => &[],
        };
        if arguments.len() != self.generic_parameters.len() {
            return Err(self
                .name
                .error("generic union requires explicit type arguments with matching arity"));
        }
        let names = self
            .generic_parameters
            .iter()
            .cloned()
            .map(Some)
            .collect::<Vec<_>>();
        self.variants
            .iter()
            .map(|variant| {
                let formal = crate::assembler::bind_parameters(
                    crate::assembler::parse_type(&variant.il())?,
                    &names,
                    false,
                );
                Ty::from_metadata(&formal.substitute_type_parameters(arguments)?)
            })
            .collect()
    }
    pub fn cases(&self, owner: &Ty) -> Result<Vec<library::Case>, Fault> {
        Ok(self
            .variants_for(owner)?
            .iter()
            .map(|variant| {
                let metadata = crate::assembler::parse_type(&variant.il()).unwrap();
                let name = metadata
                    .definition_name()
                    .unwrap()
                    .rsplit('.')
                    .next()
                    .unwrap()
                    .to_owned();
                library::Case {
                    test: format!("instance {}::get_Is{name}Case()", owner.il()),
                    extract: format!("instance {}::Get{name}Case()", owner.il()),
                    name,
                    // An empty accessor denotes the whole variant, not a wrapper payload.
                    payload: Some((variant.clone(), String::new())),
                }
            })
            .collect())
    }
    pub fn emit(&self, source: &Source) -> String {
        let owner = if self.generic_parameters.is_empty() {
            self.name.text.clone()
        } else {
            format!("{}<{}>", self.name.text, self.generic_parameters.join(","))
        };
        let mut il = format!(
            ".type {owner}\n.custom instance System.Runtime.CompilerServices.UnionAttribute::.ctor()\n.field private Stored System.Value\n"
        );
        if self.inline {
            for variant in &self.variants {
                let record = source
                    .records
                    .iter()
                    .find(|r| r.name.text == variant.il())
                    .unwrap();
                let name = record.name.text.rsplit('.').next().unwrap();
                il.push_str(&format!(".type {name}\n"));
                for field in &record.fields {
                    il.push_str(&format!(".field {} {}\n", field.name.text, field.ty.il()));
                }
                il.push_str(".end\n");
            }
        }
        for variant in &self.variants {
            let ty = variant.il();
            let metadata = crate::assembler::parse_type(&ty).unwrap();
            let name = metadata
                .definition_name()
                .unwrap()
                .rsplit('.')
                .next()
                .unwrap();
            il.push_str(&format!(
                ".method instance .ctor({ty} value) -> Void\nldarg value\nvalue.pack {ty}\nnewobj {owner}\nstarg this\nldvoid\nret\n.end\n\
                 .method instance get_Is{name}Case() -> Boolean\nldarg this\nldfld {owner}::Stored\nvalue.is {ty}\nret\n.end\n\
                 .property instance Is{name}() -> Boolean\n.get instance {owner}::get_Is{name}Case()\n.end\n\
                 .method instance Get{name}Case() -> {ty}\nldarg this\nldfld {owner}::Stored\nvalue.unpack {ty}\nret\n.end\n"
            ));
        }
        il.push_str(".end\n");
        il
    }
}

impl Source {
    pub(super) fn source_union(&self, ty: &Ty) -> Option<&Union> {
        let metadata = crate::assembler::parse_type(&ty.il()).ok()?;
        let name = metadata.definition_name()?;
        self.unions.iter().find(|u| u.name.text == name)
    }
}

impl Lowerer<'_> {
    pub(super) fn source_union_constructor(
        &mut self,
        callee: &Expr,
        types: &[Ty],
        arguments: &[Expr],
    ) -> Result<Option<Ty>, Fault> {
        let Some(path) = Self::qualified_name(callee) else {
            return Ok(None);
        };
        if self.bindings.contains_key(path.split('.').next().unwrap()) {
            return Ok(None);
        }
        let owner = Ty::Record(if types.is_empty() {
            path
        } else {
            format!(
                "{path}<{}>",
                types.iter().map(Ty::il).collect::<Vec<_>>().join(",")
            )
        });
        let Some(union) = self.source.source_union(&owner) else {
            return Ok(None);
        };
        let variants = union.variants_for(&owner)?;
        if arguments.len() != 1 {
            return Err(callee
                .at
                .error("union constructor requires one variant value"));
        }
        let actual = self.value_expression(&arguments[0])?;
        if !variants.contains(&actual) {
            return Err(callee
                .at
                .error("no union constructor accepts this variant type"));
        }
        self.body.push(format!(
            "newobj instance {}::.ctor({})",
            owner.il(),
            actual.il()
        ));
        Ok(Some(owner))
    }
}
