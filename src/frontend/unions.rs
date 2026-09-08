//! Source unions lower to ordinary carrier constructors and separate case types.
use super::*;

pub(super) struct Union {
    pub name: Token,
    pub variants: Vec<Ty>,
    pub inline: bool,
}

pub(super) fn parse(parser: &mut Parser, source: &mut Source) -> Result<(), Fault> {
    let name = parser.name()?;
    let mut variants = Vec::new();
    parser.newlines();
    let inline = !parser.eat("(");
    if inline {
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
    source.unions.push(Union {
        name,
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
            if name == &union.name.text
                || !source.records.iter().any(|r| r.name.text == *name)
                    && !source.unions.iter().any(|u| u.name.text == *name)
            {
                return Err(union
                    .name
                    .error("union variant must name a different declared source type"));
            }
            if source
                .records
                .iter()
                .any(|r| r.name.text == *name && r.is_abstract)
            {
                return Err(union
                    .name
                    .error("union variants require concrete source value types"));
            }
            if !names.insert(name.rsplit('.').next().unwrap()) {
                return Err(union.name.error("duplicate or ambiguous union case name"));
            }
        }
    }
    Ok(())
}

impl Union {
    pub fn cases(&self) -> Vec<library::Case> {
        self.variants
            .iter()
            .map(|variant| {
                let name = variant.il().rsplit('.').next().unwrap().to_owned();
                library::Case {
                    test: format!("instance {}::get_Is{name}Case()", self.name.text),
                    extract: format!("instance {}::Get{name}Case()", self.name.text),
                    name,
                    // An empty accessor denotes the whole variant, not a wrapper payload.
                    payload: Some((variant.clone(), String::new())),
                }
            })
            .collect()
    }
    pub fn emit(&self, source: &Source) -> String {
        let owner = &self.name.text;
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
            let name = ty.rsplit('.').next().unwrap();
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
