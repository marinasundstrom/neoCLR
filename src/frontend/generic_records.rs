//! Generic source record definitions use existing CLR-like type-parameter metadata.
use super::*;

impl Parser {
    pub(super) fn type_parameters(&mut self) -> Result<Vec<String>, Fault> {
        let mut names = Vec::new();
        if self.eat("<") {
            loop {
                let name = self.name()?;
                if names.contains(&name.text) || PREDEFINED_NAMES.contains(&name.text.as_str()) {
                    return Err(name.error("duplicate or reserved generic type parameter"));
                }
                names.push(name.text);
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(">")?;
        }
        Ok(names)
    }
}

impl Record {
    pub(super) fn declaration_name(&self) -> String {
        if self.generic_parameters.is_empty() {
            self.name.text.clone()
        } else {
            format!("{}<{}>", self.name.text, self.generic_parameters.join(","))
        }
    }
}

impl Source {
    pub(super) fn record_fields(&self, owner: &Ty) -> Result<Option<Vec<Field>>, Fault> {
        let metadata = crate::assembler::parse_type(&owner.il())?;
        let Some(record) = self
            .records
            .iter()
            .find(|record| Some(record.name.text.as_str()) == metadata.definition_name())
        else {
            return Ok(None);
        };
        let arguments = match &metadata {
            crate::metadata::Type::Constructed { arguments, .. } => arguments.as_slice(),
            _ => &[],
        };
        if arguments.len() != record.generic_parameters.len() {
            return Err(record.name.error("generic record argument count mismatch"));
        }
        let names = record
            .generic_parameters
            .iter()
            .cloned()
            .map(Some)
            .collect::<Vec<_>>();
        record
            .fields
            .iter()
            .map(|field| {
                let mut field = field.clone();
                let formal = crate::assembler::bind_parameters(
                    crate::assembler::parse_type(&field.ty.il())?,
                    &names,
                    false,
                );
                field.ty = Ty::from_metadata(&formal.substitute_type_parameters(arguments)?)?;
                Ok(field)
            })
            .collect::<Result<Vec<_>, Fault>>()
            .map(Some)
    }
}

impl Lowerer<'_> {
    pub(super) fn generic_record_constructor(
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
        let Some(record) = self
            .source
            .records
            .iter()
            .find(|r| r.name.text == path && !r.generic_parameters.is_empty())
        else {
            return Ok(None);
        };
        if arguments.len() != record.fields.len() {
            return Err(callee.at.error("argument count mismatch"));
        }
        // Keep the first slice explicit. Inference can build on this closed-owner path.
        if types.len() != record.generic_parameters.len() {
            return Err(callee
                .at
                .error("generic record requires explicit type arguments with matching arity"));
        }
        let owner = Ty::Record(format!(
            "{path}<{}>",
            types.iter().map(Ty::il).collect::<Vec<_>>().join(",")
        ));
        for (argument, parameter) in arguments
            .iter()
            .zip(self.source.record_fields(&owner)?.unwrap())
        {
            self.parameter_argument(argument, &parameter)?;
        }
        self.body.push(format!("newobj {}", owner.il()));
        Ok(Some(owner))
    }
}

impl Parser {
    pub(super) fn constraints(
        &mut self,
        parameters: &[String],
    ) -> Result<Vec<crate::metadata::GenericConstraint>, Fault> {
        let names = parameters.iter().cloned().map(Some).collect::<Vec<_>>();
        let mut result = Vec::new();
        loop {
            let saved = self.position;
            self.newlines();
            if !self.eat("where") {
                self.position = saved;
                break;
            }
            let parameter = self.name()?;
            self.expect(":")?;
            let mut text = parameter.text.clone();
            loop {
                text.push(' ');
                text.push_str(&self.take().text);
                if !self.eat(",") {
                    break;
                }
            }
            result.extend(
                crate::constraints::parse(&text, &names)
                    .map_err(|error| parameter.error(error.message))?,
            );
        }
        crate::constraints::validate(&result, names.len())
            .map_err(|error| self.current().error(error.message))?;
        Ok(result)
    }
}
