//! Shared explicit metadata dependency parsing for AOT and scalar JIT.
use std::{ffi::OsString, fs, path::Path};
type Error = Box<dyn std::error::Error>;
pub(super) struct Paths<'a> {
    pub modules: Vec<&'a OsString>,
    pub system: Option<&'a OsString>,
    pub object: Option<&'a OsString>,
}
pub(super) fn split(args: &mut Vec<OsString>) -> Vec<OsString> {
    args.iter()
        .position(|a| a == "--module" || a == "--system" || a == "--object-root")
        .map(|i| args.split_off(i))
        .unwrap_or_default()
}
pub(super) fn parse(args: &[OsString]) -> Result<Paths<'_>, Error> {
    if args.len() % 2 != 0 {
        return Err("expected trailing --module, --system or --object-root input pairs".into());
    }
    let mut module_paths = vec![];
    let mut system_path = None;
    let mut object_path = None;
    for pair in args.chunks(2) {
        match pair[0].to_str() {
            Some("--module") => module_paths.push(&pair[1]),
            Some("--system") if system_path.is_none() => system_path = Some(&pair[1]),
            Some("--object-root") if object_path.is_none() => object_path = Some(&pair[1]),
            _ => return Err("invalid or duplicate load-context option".into()),
        }
    }
    if object_path.is_some() && system_path.is_none() {
        return Err("--object-root requires explicit --system".into());
    }
    if system_path.is_some() && module_paths.is_empty() {
        return Err("explicit runtime context requires at least one --module dependency".into());
    }
    Ok(Paths {
        modules: module_paths,
        system: system_path,
        object: object_path,
    })
}
pub(super) fn object_root(
    module_paths: &[&OsString],
    object_path: Option<&OsString>,
    dependencies: &[neoclr::Module],
) -> Result<Option<neoclr::metadata::TypeDefId>, Error> {
    Ok(if let Some(path) = object_path {
        if Path::new(path).extension().is_some_and(|e| e == "neoil") {
            return Err("--object-root requires compiled metadata".into());
        }
        let selected = fs::canonicalize(path)?;
        let matches: Vec<_> = module_paths
            .iter()
            .enumerate()
            .filter(|(_, p)| fs::canonicalize(p).is_ok_and(|p| p == selected))
            .map(|(i, _)| i)
            .collect();
        let [index] = matches.as_slice() else {
            return Err("--object-root must identify exactly one explicit --module input".into());
        };
        let module = &dependencies[*index];
        let roots: Vec<_> = module
            .types
            .iter()
            .enumerate()
            .filter(|(_, t)| t.name == "System.Object")
            .map(|(i, _)| i)
            .collect();
        let [index] = roots.as_slice() else {
            return Err("Object root artifact must contain exactly one System.Object".into());
        };
        Some(neoclr::metadata::TypeDefId {
            module: module.name.clone(),
            revision: module.revision.clone(),
            index: *index as u32,
        })
    } else {
        None
    })
}
pub(super) fn verification(
    input: &neoclr::Module,
    system: &neoclr::Module,
    dependencies: &[neoclr::Module],
    root: Option<&neoclr::metadata::TypeDefId>,
) -> Result<neoclr::LoadedProgram, Error> {
    if let Some(root) = root {
        neoclr::LoadedProgram::with_modules_and_object_root(input, system, dependencies, root)
    } else {
        neoclr::LoadedProgram::with_modules(input, system, dependencies)
    }
    .map_err(|error| error.to_string().into())
}
