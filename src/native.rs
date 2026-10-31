//! Explicit runtime binding registry. A name alone never activates host dispatch.
use crate::{
    Fault, Value,
    metadata::{Function, Type},
};
use unicode_segmentation::UnicodeSegmentation;

pub(crate) enum Binding {
    MemoryView(crate::memory_view_services::Operation),
    StackTraceCapture,
    Cryptography(crate::cryptography::Operation),
    Tls(crate::tls::Operation),
    GcCollect,
    GcInfo(u8),
    GcKeepAlive,
    Socket(crate::socket_io::Operation),
    Resolve(crate::name_resolution::Operation),
    Timer(crate::timers::Operation),
    #[cfg(test)]
    TestSocketReceive,
    SystemCultureName,
    TimeZoneExists,
    TimeZoneOffset,
    TimeZoneMapLocal,
    TimeZoneDatabaseVersion,
    SystemTimeZoneName,
    EnvironmentPlatformInfo,
    EnvironmentArguments,
    EnvironmentCurrentDirectory,
    EnvironmentVariable,
    PathCombine,
    PathGetFileName,
    UnixTimeTicks,
    UnixTimeToLocal,
    Math(crate::math::Operation),
    Reflection(crate::reflection::Query),
    ReflectionConstructionCheck,
    ReflectionConstruct,
    InvokeFunction,
    FunctionInfo,
    ReflectionMember(u8),
    ReflectionMemberCheck(u8),
    ReflectionAssignable,
    ReflectionSubclass,
    ReflectionArray(u8),
    ReflectionCollection(u8),
    ReflectionPropertyCheck(bool),
    ReflectionProperty(bool),
    ObjectTypeHandle,
    ObjectReferenceEquals,
    ObjectEquals,
    ObjectIdentityHash,
    ExecutingAssembly,
    CurrentTaskQueue,
    GenericCurrentTaskQueue,
    GenericDefaultTaskQueue,
    GenericRegisterTaskQueue,
    DefaultTaskQueue,
    RegisterDefaultTaskQueue,
    DrainEntryTasks,
    RuntimeAsyncYield,
    RuntimeAsyncCreate,
    RuntimeAsyncStart,
    RuntimeAsyncStartManaged,
    RuntimeAsyncCreateOwned,
    RuntimeAsyncCurrentOwner,
    RuntimeAsyncDelay,
    RuntimeAsyncComplete,
    RuntimeAsyncCompleteUnit,
    RuntimeAsyncCancel,
    RuntimeAsyncAwait,
    RuntimeAsyncAwaitUnit,
    RuntimeAsyncIsCompleted,
    RuntimeAsyncIsCancelled,
    RuntimeAsyncWaitCompletion,
    RuntimeAsyncRelease,
    ScheduleTask,
    SchedulerSubmit,
    SchedulerCapture,
    SchedulerSubmitTo,
    SchedulerRegister,
    SchedulerActivate,
    SchedulerCancel,
    StartWorker(bool),
    JoinWorker,
    JoinWorkerResult,
    RequestWorkerCancellation,
    NotifyWorker,
    AssemblyInfo(crate::assembly_info::Query),
    TypeName,
    TypeEquals,
    TypeArgumentCount,
    TypeArgument,
    ParseInt32,
    ParseInt64,
    ParseNumber(crate::numeric_parse::Kind),
    StringCasing(bool),
    Int32ToString,
    IntegerToString,
    FloatingFormat(bool),
    NativeIntegerTo64,
    NativeAllocate,
    NativeFree,
    NativeMultiplyChecked,
    NativeCopy(bool),
    WriteLine,
    Fault,
    CharCategory,
    Utf8Encode,
    Utf8Decode,
    Utf8DecodeSlice,
    StringConcat,
    StringReplaceOrdinal,
    StringReplaceLineEndings,
    StringJoinParts,
    StringIntern,
    StringFromChars,
    StringSliceGraphemes,
    StringGraphemeAt,
    StringGraphemeCount,
    CharIsSingleGrapheme,
    CharCompareOrdinal,
    CharCompareOrdinalIgnoreCase,
    CharFromString,
    CharText,
    StringGraphemes,
    StringScalars,
    StringByteCount,
    StringCompareOrdinal,
    StringCompareOrdinalIgnoreCase,
    StringCompareCanonical,
    StringHashCanonical,
    CharCompareCanonical,
    CharHashCanonical,
    StringHashOrdinal,
    StringHashOrdinalIgnoreCase,
    StringContainsCanonical,
    StringStartsWithCanonical,
    StringEndsWithCanonical,
    StringContainsOrdinalIgnoreCase,
    StringStartsWithOrdinalIgnoreCase,
    StringEndsWithOrdinalIgnoreCase,
    StringContainsOrdinal,
    StringStartsWithOrdinal,
    StringEndsWithOrdinal,
    StringSliceUtf8,
    FileResource(crate::file_streams::Operation),
    StringSnapshot,
    ReadAllText,
    WriteAllText,
    ConsoleReadByte,
    ConsoleTerminalOpen,
    ConsoleTerminalReadByte,
    ConsoleTerminalClose,
    ConsoleWriteBytes,
    ConsoleFlush,
}

/// Bind descriptor signatures against their actual source-owned nominal identity.
/// Other services retain their existing exact physical-signature contract.
pub(crate) fn bind_in(module: &crate::Module, function: &Function) -> Result<Binding, Fault> {
    if function.name == "neoCLR.Runtime.TypeModule"
        && function.returns != Type::from_name("System.Introspection.ModuleInfo")
    {
        crate::reflection::source_module_provider(module, &function.returns)?;
    }
    // Normalize only the descriptive service ABI, using the identities named in
    // this declaration. The actual function signature remains scoped and is used
    // when materializing its result; this never aliases the loaded type catalog.
    fn descriptive(module: &crate::Module, ty: &Type) -> Type {
        match ty {
            Type::Array(element) => Type::Array(Box::new(descriptive(module, element))),
            Type::ArrayRef(element) => Type::ArrayRef(Box::new(descriptive(module, element))),
            Type::Named(_) | Type::Constructed { .. } => {
                let Some(origin) = module.type_definition(ty).and_then(|d| d.origin.as_ref())
                else {
                    return ty.clone();
                };
                let name = origin.name.trim_end_matches("`1");
                if !(name.starts_with("System.Introspection.") || name == "System.Optional") {
                    return ty.clone();
                }
                if ty.generic_arguments().is_empty() {
                    Type::from_name(name)
                } else {
                    Type::Constructed {
                        definition: name.into(),
                        arguments: ty
                            .generic_arguments()
                            .iter()
                            .map(|t| descriptive(module, t))
                            .collect(),
                    }
                }
            }
            _ => ty.clone(),
        }
    }
    let reflection = crate::reflection::Query::binding(&function.name).is_some();
    let assembly = crate::assembly_info::Query::binding(&function.name).is_some();
    let relevant = reflection
        || assembly
        || matches!(
            function.name.as_str(),
            "neoCLR.Runtime.ExecutingAssembly"
                | "neoCLR.Runtime.ReflectionArrayCreate"
                | "neoCLR.Runtime.FunctionInfo"
        );
    if relevant
        || function
            .name
            .starts_with("neoCLR.Runtime.ReflectionCollection")
    {
        let mut contract = function.clone();
        contract.parameters = function
            .parameters
            .iter()
            .map(|t| descriptive(module, t))
            .collect();
        contract.returns = descriptive(module, &function.returns);
        if reflection || assembly {
            if let Type::ArrayRef(element) = contract.returns {
                contract.returns = Type::Array(element);
            }
        }
        bind(&contract)
    } else {
        bind(function)
    }
}

pub(crate) fn bind(function: &Function) -> Result<Binding, Fault> {
    if !function.is_internal_call() || function.instance || function.owner.is_some() {
        return Err(Fault::new("native binding requires InternalCall metadata"));
    }
    // Return convention belongs to the signature, not a second service-name
    // allowlist. Each binding below still checks its exact parameter/result types.
    // Check this before any early-returning binding, including generic services.
    if function.no_result && function.returns != Type::Void {
        return Err(Fault::new(
            "no-result native service requires a Void return",
        ));
    }
    if let Some(operation) = crate::memory_view_services::bind(function) {
        return operation.map(Binding::MemoryView);
    }
    if matches!(
        function.name.as_str(),
        "neoCLR.Runtime.GetDefaultTaskQueue"
            | "neoCLR.Runtime.GetCurrentTaskQueue"
            | "neoCLR.Runtime.RegisterTaskQueue"
    ) {
        let parameter = match function.generic_arguments.as_slice() {
            [] => Type::MethodTypeParameter(0),
            [argument] => argument.clone(),
            _ => return Err(Fault::new("task queue service requires one type argument")),
        };
        let register = function.name == "neoCLR.Runtime.RegisterTaskQueue";
        if function.generic_parameters.len() != 1
            || !function.generic_constraints.is_empty()
            || (function.no_result && !register)
            || function.parameters
                != if register {
                    vec![parameter.clone()]
                } else {
                    vec![]
                }
            || function.returns != if register { Type::Void } else { parameter }
        {
            return Err(Fault::new("invalid generic task queue service signature"));
        }
        return Ok(if register {
            Binding::GenericRegisterTaskQueue
        } else if function.name == "neoCLR.Runtime.GetDefaultTaskQueue" {
            Binding::GenericDefaultTaskQueue
        } else {
            Binding::GenericCurrentTaskQueue
        });
    }
    if function.name.starts_with("neoCLR.Runtime.Crypto")
        && (!function.generic_parameters.is_empty()
            || !function.generic_arguments.is_empty()
            || !function.generic_constraints.is_empty())
    {
        return Err(Fault::new("cryptography service cannot be generic"));
    }
    if let Some((operation, arity)) = crate::math::Operation::binding(&function.name) {
        if function.parameters != vec![Type::Double; arity] || function.returns != Type::Double {
            return Err(Fault::new("math binding signature mismatch"));
        }
        return Ok(Binding::Math(operation));
    }
    if let Some((query, count, result)) = crate::assembly_info::Query::binding(&function.name) {
        let mut parameters = vec![Type::String; count];
        if matches!(query, crate::assembly_info::Query::ModuleMemberAttributes) {
            parameters[2] = Type::Int32;
        }
        if function.parameters != parameters
            || function.returns != crate::assembler::parse_type(result)?
        {
            return Err(Fault::new("assembly service signature mismatch"));
        }
        return Ok(Binding::AssemblyInfo(query));
    }
    if let Some((query, integer, returns)) = crate::reflection::Query::binding(&function.name) {
        let expected = if function.name == "neoCLR.Runtime.MemberCustomAttributes" {
            vec![
                Type::from_name("System.Introspection.TypeInfo"),
                Type::Int32,
            ]
        } else if matches!(
            function.name.as_str(),
            "neoCLR.Runtime.TypeApplyGenericArgument"
                | "neoCLR.Runtime.TypeApplyGenericArgumentCheck"
        ) {
            vec![
                Type::RuntimeTypeHandle,
                Type::Int32,
                Type::RuntimeTypeHandle,
            ]
        } else if integer {
            vec![Type::RuntimeTypeHandle, Type::Int32]
        } else {
            vec![Type::RuntimeTypeHandle]
        };
        // The Raven model uses TypeInfo throughout; the historical Neo profile
        // retains Type. Admit only the corresponding exact result signature.
        fn info_result(ty: &Type) -> Type {
            match ty {
                Type::Named(name) if name == "System.Type" => {
                    Type::from_name("System.Introspection.TypeInfo")
                }
                Type::Array(element) => Type::Array(Box::new(info_result(element))),
                Type::Constructed {
                    definition,
                    arguments,
                } => Type::Constructed {
                    definition: definition.clone(),
                    arguments: arguments.iter().map(info_result).collect(),
                },
                _ => ty.clone(),
            }
        }
        if function.parameters != expected
            || (function.returns != returns && function.returns != info_result(&returns))
        {
            return Err(Fault::new("reflection binding signature mismatch"));
        }
        return Ok(Binding::Reflection(query));
    }
    for (kind, name) in ["Kind", "Count", "Get", "Key", "Create"].iter().enumerate() {
        if function.name != format!("neoCLR.Runtime.ReflectionCollection{name}") {
            continue;
        }
        let object = Type::from_name("System.Object");
        let mut expected = vec![Type::from_name("System.Introspection.TypeInfo")];
        match kind {
            0 => {}
            1 => expected.push(object.clone()),
            2 | 3 => expected.extend([object.clone(), Type::Int32]),
            _ => expected.extend([
                Type::ArrayRef(Box::new(object.clone())),
                Type::ArrayRef(Box::new(Type::String)),
            ]),
        }
        let returns = match kind {
            0 | 1 => Type::Int32,
            3 => Type::String,
            _ => object,
        };
        if function.parameters != expected || function.returns != returns || function.no_result {
            return Err(Fault::new("reflection collection signature mismatch"));
        }
        return Ok(Binding::ReflectionCollection(kind as u8));
    }
    if function.name == "neoCLR.Runtime.FunctionInfo" {
        if function.parameters != [Type::from_name("System.Object")]
            || function.returns != Type::from_name("System.Introspection.MethodInfo")
            || function.no_result
        {
            return Err(Fault::new("function inspection signature mismatch"));
        }
        return Ok(Binding::FunctionInfo);
    }
    if matches!(
        function.name.as_str(),
        "neoCLR.Runtime.InvokeFunction" | "neoCLR.Runtime.InvokeFunctionCheck"
    ) {
        let object = Type::from_name("System.Object");
        let returns = if function.name.ends_with("Check") {
            Type::Int32
        } else {
            object.clone()
        };
        if function.parameters != [object.clone(), Type::ArrayRef(Box::new(object))]
            || function.returns != returns
            || function.no_result
        {
            return Err(Fault::new("dynamic function invocation signature mismatch"));
        }
        return Ok(Binding::InvokeFunction);
    }
    for (kind, name) in ["Length", "Get", "Create"].iter().enumerate() {
        if function.name != format!("neoCLR.Runtime.ReflectionArray{name}") {
            continue;
        }
        let object = Type::from_name("System.Object");
        let expected = match kind {
            0 => vec![object.clone()],
            1 => vec![object.clone(), Type::Int32],
            _ => vec![
                Type::from_name("System.Introspection.TypeInfo"),
                Type::ArrayRef(Box::new(object.clone())),
            ],
        };
        let returns = if kind == 0 { Type::Int32 } else { object };
        if function.parameters != expected || function.returns != returns || function.no_result {
            return Err(Fault::new("reflection array signature mismatch"));
        }
        return Ok(Binding::ReflectionArray(kind as u8));
    }
    if function.name == "neoCLR.Runtime.ReflectionAssignable" {
        if function.parameters != [Type::RuntimeTypeHandle, Type::RuntimeTypeHandle]
            || function.returns != Type::Boolean
            || function.no_result
        {
            return Err(Fault::new("reflection assignability signature mismatch"));
        }
        return Ok(Binding::ReflectionAssignable);
    }
    for (kind, name) in [
        "ConstructArgs",
        "Invoke",
        "FieldGet",
        "FieldSet",
        "ConstructorInvoke",
    ]
    .iter()
    .enumerate()
    {
        let execution = format!("neoCLR.Runtime.Reflection{name}");
        let check = function.name == format!("{execution}Check");
        if function.name != execution && !check {
            continue;
        }
        let object = Type::from_name("System.Object");
        let expected = vec![
            Type::RuntimeTypeHandle,
            Type::Int32,
            object.clone(),
            Type::ArrayRef(Box::new(object.clone())),
        ];
        if function.parameters != expected
            || function.returns != if check { Type::Int32 } else { object }
            || function.no_result
        {
            return Err(Fault::new("reflection member signature mismatch"));
        }
        return Ok(if check {
            Binding::ReflectionMemberCheck(kind as u8)
        } else {
            Binding::ReflectionMember(kind as u8)
        });
    }
    if let Some((setter, check)) = match function.name.as_str() {
        "neoCLR.Runtime.ReflectionPropertyGetCheck" => Some((false, true)),
        "neoCLR.Runtime.ReflectionPropertySetCheck" => Some((true, true)),
        "neoCLR.Runtime.ReflectionPropertyGet" => Some((false, false)),
        "neoCLR.Runtime.ReflectionPropertySet" => Some((true, false)),
        _ => None,
    } {
        let mut expected = vec![
            Type::RuntimeTypeHandle,
            Type::Int32,
            Type::from_name("System.Object"),
        ];
        if setter {
            expected.push(Type::from_name("System.Object"));
        }
        let returns = if check {
            Type::Int32
        } else if setter {
            Type::Void
        } else {
            Type::from_name("System.Object")
        };
        if function.parameters != expected
            || function.returns != returns
            || (function.no_result && (!setter || check))
        {
            return Err(Fault::new("reflection property binding signature mismatch"));
        }
        return Ok(if check {
            Binding::ReflectionPropertyCheck(setter)
        } else {
            Binding::ReflectionProperty(setter)
        });
    }
    if function.name == "neoCLR.Runtime.StackTraceCapture" {
        if function.parameters != [Type::Int32]
            || function.returns != Type::String
            || function.no_result
        {
            return Err(Fault::new("stack capture service signature mismatch"));
        }
        return Ok(Binding::StackTraceCapture);
    }
    if function.name == "neoCLR.Runtime.GCCollect" || function.name == "neoCLR.Runtime.GCKeepAlive"
    {
        let keep_alive = function.name.ends_with("GCKeepAlive");
        let expected = if keep_alive {
            vec![Type::from_name("System.Object")]
        } else {
            vec![]
        };
        if function.parameters != expected || function.returns != Type::Void {
            return Err(Fault::new("GC control service signature mismatch"));
        }
        return Ok(if keep_alive {
            Binding::GcKeepAlive
        } else {
            Binding::GcCollect
        });
    }
    const GC_INFO: [&str; 6] = [
        "GCCollectionCount",
        "GCAllocatedObjectCount",
        "GCHeapObjectCount",
        "GCPeakHeapObjectCount",
        "GCReclaimedObjectCount",
        "GCHeapObjectLimit",
    ];
    if let Some(index) = GC_INFO
        .iter()
        .position(|name| function.name == format!("neoCLR.Runtime.{name}"))
    {
        if !function.parameters.is_empty() || function.no_result || function.returns != Type::Int64
        {
            return Err(Fault::new("GC information service signature mismatch"));
        }
        return Ok(Binding::GcInfo(index as u8));
    }
    let (binding, returns) = match (function.name.as_str(), function.parameters.as_slice()) {
        ("neoCLR.Runtime.ReflectionConstructionCheck", [Type::RuntimeTypeHandle]) => {
            (Binding::ReflectionConstructionCheck, Type::Int32)
        }
        ("neoCLR.Runtime.ReflectionConstruct", [Type::RuntimeTypeHandle]) => (
            Binding::ReflectionConstruct,
            Type::from_name("System.Object"),
        ),
        ("neoCLR.Runtime.TimeZoneExists", [Type::String]) => {
            (Binding::TimeZoneExists, Type::Boolean)
        }
        ("neoCLR.Runtime.TimeZoneOffset", [Type::String, Type::Int64]) => {
            (Binding::TimeZoneOffset, Type::Int32)
        }
        ("neoCLR.Runtime.TimeZoneMapLocal", [Type::String, Type::Int64]) => (
            Binding::TimeZoneMapLocal,
            if function.returns == Type::ArrayRef(Box::new(Type::Int64)) {
                function.returns.clone()
            } else {
                Type::Array(Box::new(Type::Int64))
            },
        ),
        ("neoCLR.Runtime.TimeZoneDatabaseVersion", []) => {
            (Binding::TimeZoneDatabaseVersion, Type::String)
        }
        ("neoCLR.Runtime.SystemTimeZoneName", []) => (Binding::SystemTimeZoneName, Type::String),
        ("neoCLR.Runtime.SystemCultureName", []) => (Binding::SystemCultureName, Type::String),
        ("neoCLR.Runtime.EnvironmentArguments", []) => (
            Binding::EnvironmentArguments,
            if function.returns == Type::ArrayRef(Box::new(Type::String)) {
                function.returns.clone()
            } else {
                Type::Array(Box::new(Type::String))
            },
        ),
        ("neoCLR.Runtime.EnvironmentPlatformInfo", [Type::Int32]) => {
            (Binding::EnvironmentPlatformInfo, Type::String)
        }
        ("neoCLR.Runtime.EnvironmentCurrentDirectory", []) => {
            (Binding::EnvironmentCurrentDirectory, Type::Value)
        }
        ("neoCLR.Runtime.EnvironmentVariable", [Type::String]) => {
            (Binding::EnvironmentVariable, Type::Value)
        }
        ("neoCLR.Runtime.PathCombine", [Type::String, Type::String]) => {
            (Binding::PathCombine, Type::String)
        }
        ("neoCLR.Runtime.PathGetFileName", [Type::String]) => {
            (Binding::PathGetFileName, Type::String)
        }
        ("neoCLR.Runtime.UnixTimeToLocal", [Type::Int64]) => {
            (Binding::UnixTimeToLocal, Type::Array(Box::new(Type::Int32)))
        }
        ("neoCLR.Runtime.UnixTimeTicks", []) => (Binding::UnixTimeTicks, Type::Int64),
        ("neoCLR.Runtime.StringToUpperInvariant", [Type::String]) => {
            (Binding::StringCasing(true), Type::String)
        }
        ("neoCLR.Runtime.StringToLowerInvariant", [Type::String]) => {
            (Binding::StringCasing(false), Type::String)
        }
        ("neoCLR.Runtime.ParseInt64", [Type::String]) => (Binding::ParseInt64, Type::Value),
        ("neoCLR.Runtime.ParseSByte", [Type::String]) => (
            Binding::ParseNumber(crate::numeric_parse::Kind::SByte),
            Type::Value,
        ),
        ("neoCLR.Runtime.ParseByte", [Type::String]) => (
            Binding::ParseNumber(crate::numeric_parse::Kind::Byte),
            Type::Value,
        ),
        ("neoCLR.Runtime.ParseInt16", [Type::String]) => (
            Binding::ParseNumber(crate::numeric_parse::Kind::Int16),
            Type::Value,
        ),
        ("neoCLR.Runtime.ParseUInt16", [Type::String]) => (
            Binding::ParseNumber(crate::numeric_parse::Kind::UInt16),
            Type::Value,
        ),
        ("neoCLR.Runtime.ParseUInt32", [Type::String]) => (
            Binding::ParseNumber(crate::numeric_parse::Kind::UInt32),
            Type::Value,
        ),
        ("neoCLR.Runtime.ParseUInt64", [Type::String]) => (
            Binding::ParseNumber(crate::numeric_parse::Kind::UInt64),
            Type::Value,
        ),
        ("neoCLR.Runtime.ParseSingle", [Type::String]) => (
            Binding::ParseNumber(crate::numeric_parse::Kind::Single),
            Type::Value,
        ),
        ("neoCLR.Runtime.ParseDouble", [Type::String]) => (
            Binding::ParseNumber(crate::numeric_parse::Kind::Double),
            Type::Value,
        ),
        ("neoCLR.Runtime.ParseBoolean", [Type::String]) => (
            Binding::ParseNumber(crate::numeric_parse::Kind::Boolean),
            Type::Value,
        ),
        ("neoCLR.Runtime.ParseInt32", [Type::String]) => (Binding::ParseInt32, Type::Value),
        ("neoCLR.Runtime.IntPtrToInt64", [Type::IntPtr]) => {
            (Binding::NativeIntegerTo64, Type::Int64)
        }
        ("neoCLR.Runtime.UIntPtrToUInt64", [Type::UIntPtr]) => {
            (Binding::NativeIntegerTo64, Type::UInt64)
        }
        ("neoCLR.Runtime.Int32ToString", [Type::Int32]) => (Binding::Int32ToString, Type::String),
        ("neoCLR.Runtime.Int64ToString", [Type::Int64])
        | ("neoCLR.Runtime.UInt64ToString", [Type::UInt64]) => {
            (Binding::IntegerToString, Type::String)
        }
        (
            "neoCLR.Runtime.FormatSingle",
            [Type::Single, Type::Int32, Type::Int32, Type::Boolean],
        ) => (Binding::FloatingFormat(true), Type::String),
        (
            "neoCLR.Runtime.FormatDouble",
            [Type::Double, Type::Int32, Type::Int32, Type::Boolean],
        ) => (Binding::FloatingFormat(false), Type::String),
        ("neoCLR.Runtime.NativeAllocate", [Type::UIntPtr]) => {
            (Binding::NativeAllocate, Type::Ptr(Box::new(Type::Void)))
        }
        ("neoCLR.Runtime.NativeFree", [Type::Ptr(element)])
            if **element == Type::Void && function.no_result =>
        {
            (Binding::NativeFree, Type::Void)
        }
        ("neoCLR.Runtime.NativeCopyTo", [view, Type::Ptr(element), Type::UIntPtr])
            if **element == Type::Void
                && function.no_result
                && *view
                    == (Type::Constructed {
                        definition: "System.ReadOnlySlice".into(),
                        arguments: vec![Type::Byte],
                    }) =>
        {
            (Binding::NativeCopy(true), Type::Void)
        }
        ("neoCLR.Runtime.NativeCopyFrom", [Type::Ptr(element), Type::UIntPtr, view])
            if **element == Type::Void
                && function.no_result
                && *view
                    == (Type::Constructed {
                        definition: "System.Slice".into(),
                        arguments: vec![Type::Byte],
                    }) =>
        {
            (Binding::NativeCopy(false), Type::Void)
        }
        ("neoCLR.Runtime.NativeMultiplyChecked", [Type::UIntPtr, Type::UIntPtr]) => {
            (Binding::NativeMultiplyChecked, Type::UIntPtr)
        }
        ("neoCLR.Runtime.Fault", [Type::String]) => (Binding::Fault, Type::Void),
        ("neoCLR.Runtime.Fail", [Type::String]) if function.no_result => {
            (Binding::Fault, Type::Void)
        }
        ("neoCLR.Runtime.WriteLine", [Type::String]) => (Binding::WriteLine, Type::Void),
        ("neoCLR.Runtime.CharCategory", [Type::UInt32]) => (Binding::CharCategory, Type::Int32),
        ("neoCLR.Runtime.TlsPump", types)
            if types.len() == 5
                && types
                    .iter()
                    .all(|ty| *ty == Type::ArrayRef(Box::new(Type::Byte))) =>
        {
            (Binding::Tls(crate::tls::Operation::Pump), Type::Int32)
        }
        ("neoCLR.Runtime.TlsCreateClient", types)
            if types.len() == 3
                && types
                    .iter()
                    .all(|ty| *ty == Type::ArrayRef(Box::new(Type::Byte))) =>
        {
            (
                Binding::Tls(crate::tls::Operation::ClientConfig),
                Type::Int32,
            )
        }
        ("neoCLR.Runtime.TlsCreateServer", types)
            if types.len() == 4
                && types
                    .iter()
                    .all(|ty| *ty == Type::ArrayRef(Box::new(Type::Byte))) =>
        {
            (
                Binding::Tls(crate::tls::Operation::ServerConfig),
                Type::Int32,
            )
        }
        ("neoCLR.Runtime.TlsCreateSession", types)
            if types.len() == 3
                && types
                    .iter()
                    .all(|ty| *ty == Type::ArrayRef(Box::new(Type::Byte))) =>
        {
            (Binding::Tls(crate::tls::Operation::Session), Type::Int32)
        }
        ("neoCLR.Runtime.TlsRelease", types)
            if types.len() == 1
                && types
                    .iter()
                    .all(|ty| *ty == Type::ArrayRef(Box::new(Type::Byte))) =>
        {
            (Binding::Tls(crate::tls::Operation::Release), Type::Int32)
        }
        ("neoCLR.Runtime.TlsAbort", types)
            if types.len() == 1
                && types
                    .iter()
                    .all(|ty| *ty == Type::ArrayRef(Box::new(Type::Byte))) =>
        {
            (Binding::Tls(crate::tls::Operation::Abort), Type::Int32)
        }
        ("neoCLR.Runtime.CryptoSha256", [data, output])
            if *data == crate::cryptography::view_type(true)
                && *output == crate::cryptography::view_type(false) =>
        {
            (
                Binding::Cryptography(crate::cryptography::Operation::Sha256),
                Type::Int32,
            )
        }
        ("neoCLR.Runtime.CryptoHmacSha256", [key, data, output])
            if *key == crate::cryptography::view_type(true)
                && *data == crate::cryptography::view_type(true)
                && *output == crate::cryptography::view_type(false) =>
        {
            (
                Binding::Cryptography(crate::cryptography::Operation::HmacSha256),
                Type::Int32,
            )
        }
        ("neoCLR.Runtime.CryptoHmacSha256Verify", [key, data, tag])
            if [key, data, tag]
                .iter()
                .all(|ty| **ty == crate::cryptography::view_type(true)) =>
        {
            (
                Binding::Cryptography(crate::cryptography::Operation::VerifyHmacSha256),
                Type::Boolean,
            )
        }
        ("neoCLR.Runtime.CryptoRandomFill", [output])
            if *output == crate::cryptography::view_type(false) =>
        {
            (
                Binding::Cryptography(crate::cryptography::Operation::RandomFill),
                Type::Int32,
            )
        }
        ("neoCLR.Runtime.Utf8Encode", [Type::String]) => {
            (Binding::Utf8Encode, Type::Array(Box::new(Type::Byte)))
        }
        ("neoCLR.Runtime.Utf8DecodeSlice", [input])
            if *input == crate::cryptography::view_type(true) =>
        {
            (Binding::Utf8DecodeSlice, Type::Value)
        }
        ("neoCLR.Runtime.Utf8Decode", [Type::ArrayRef(element)]) if **element == Type::Byte => {
            (Binding::Utf8Decode, Type::Value)
        }
        ("neoCLR.Runtime.StringFromChars", [Type::ArrayRef(element)])
            if **element == Type::Char =>
        {
            (Binding::StringFromChars, Type::String)
        }
        ("neoCLR.Runtime.StringSliceGraphemes", [Type::String, Type::Int32, Type::Int32]) => {
            (Binding::StringSliceGraphemes, Type::String)
        }
        ("neoCLR.Runtime.StringGraphemeAt", [Type::String, Type::Int32]) => {
            (Binding::StringGraphemeAt, Type::Char)
        }
        ("neoCLR.Runtime.StringIntern", [Type::String]) => (Binding::StringIntern, Type::String),
        (
            "neoCLR.Runtime.StringJoinParts",
            [
                Type::ArrayRef(element),
                Type::Int32,
                Type::String,
                Type::Int32,
            ],
        ) if **element == Type::String => (Binding::StringJoinParts, Type::String),
        ("neoCLR.Runtime.StringReplaceLineEndings", [Type::String, Type::String]) => {
            (Binding::StringReplaceLineEndings, Type::String)
        }
        ("neoCLR.Runtime.StringReplaceOrdinal", [Type::String, Type::String, Type::String]) => {
            (Binding::StringReplaceOrdinal, Type::String)
        }
        ("neoCLR.Runtime.StringConcat", [Type::String, Type::String]) => {
            (Binding::StringConcat, Type::String)
        }
        ("neoCLR.Runtime.StringCompareOrdinal", [Type::String, Type::String]) => {
            (Binding::StringCompareOrdinal, Type::Int32)
        }
        ("neoCLR.Runtime.StringCompareCanonical", [Type::String, Type::String]) => {
            (Binding::StringCompareCanonical, Type::Int32)
        }
        ("neoCLR.Runtime.StringHashCanonical", [Type::String]) => {
            (Binding::StringHashCanonical, Type::Int32)
        }
        ("neoCLR.Runtime.CharCompareCanonical", [Type::Char, Type::Char]) => {
            (Binding::CharCompareCanonical, Type::Int32)
        }
        ("neoCLR.Runtime.CharHashCanonical", [Type::Char]) => {
            (Binding::CharHashCanonical, Type::Int32)
        }
        ("neoCLR.Runtime.StringCompareOrdinalIgnoreCase", [Type::String, Type::String]) => {
            (Binding::StringCompareOrdinalIgnoreCase, Type::Int32)
        }
        ("neoCLR.Runtime.StringHashOrdinal", [Type::String]) => {
            (Binding::StringHashOrdinal, Type::Int32)
        }
        ("neoCLR.Runtime.StringHashOrdinalIgnoreCase", [Type::String]) => {
            (Binding::StringHashOrdinalIgnoreCase, Type::Int32)
        }
        ("neoCLR.Runtime.StringContainsCanonical", [Type::String, Type::String]) => {
            (Binding::StringContainsCanonical, Type::Boolean)
        }
        ("neoCLR.Runtime.StringStartsWithCanonical", [Type::String, Type::String]) => {
            (Binding::StringStartsWithCanonical, Type::Boolean)
        }
        ("neoCLR.Runtime.StringEndsWithCanonical", [Type::String, Type::String]) => {
            (Binding::StringEndsWithCanonical, Type::Boolean)
        }
        ("neoCLR.Runtime.StringContainsOrdinalIgnoreCase", [Type::String, Type::String]) => {
            (Binding::StringContainsOrdinalIgnoreCase, Type::Boolean)
        }
        ("neoCLR.Runtime.StringStartsWithOrdinalIgnoreCase", [Type::String, Type::String]) => {
            (Binding::StringStartsWithOrdinalIgnoreCase, Type::Boolean)
        }
        ("neoCLR.Runtime.StringEndsWithOrdinalIgnoreCase", [Type::String, Type::String]) => {
            (Binding::StringEndsWithOrdinalIgnoreCase, Type::Boolean)
        }
        ("neoCLR.Runtime.StringContainsOrdinal", [Type::String, Type::String]) => {
            (Binding::StringContainsOrdinal, Type::Boolean)
        }
        ("neoCLR.Runtime.StringStartsWithOrdinal", [Type::String, Type::String]) => {
            (Binding::StringStartsWithOrdinal, Type::Boolean)
        }
        ("neoCLR.Runtime.StringEndsWithOrdinal", [Type::String, Type::String]) => {
            (Binding::StringEndsWithOrdinal, Type::Boolean)
        }
        ("neoCLR.Runtime.StringGraphemeCount", [Type::String]) => {
            (Binding::StringGraphemeCount, Type::Int32)
        }
        ("neoCLR.Runtime.CharIsSingleGrapheme", [Type::String]) => {
            (Binding::CharIsSingleGrapheme, Type::Boolean)
        }
        ("neoCLR.Runtime.CharCompareOrdinal", [Type::Char, Type::Char]) => {
            (Binding::CharCompareOrdinal, Type::Int32)
        }
        ("neoCLR.Runtime.CharCompareOrdinalIgnoreCase", [Type::Char, Type::Char]) => {
            (Binding::CharCompareOrdinalIgnoreCase, Type::Int32)
        }
        ("neoCLR.Runtime.CharFromString", [Type::String]) => (Binding::CharFromString, Type::Char),
        ("neoCLR.Runtime.CharText", [Type::Char]) => (Binding::CharText, Type::String),
        ("neoCLR.Runtime.StringGraphemes", [Type::String]) => {
            (Binding::StringGraphemes, Type::Array(Box::new(Type::Char)))
        }
        ("neoCLR.Runtime.StringScalars", [Type::String]) => {
            (Binding::StringScalars, Type::Array(Box::new(Type::UInt32)))
        }
        ("neoCLR.Runtime.StringByteCount", [Type::String]) => {
            (Binding::StringByteCount, Type::Int32)
        }
        ("neoCLR.Runtime.StringSliceUtf8", [Type::String, Type::Int32, Type::Int32]) => {
            (Binding::StringSliceUtf8, Type::Value)
        }
        ("neoCLR.Runtime.TimerStart", [Type::Int32, callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (Binding::Timer(crate::timers::Operation::Start), Type::Int64)
        }
        ("neoCLR.Runtime.TimerCancel", [Type::Int64]) => (
            Binding::Timer(crate::timers::Operation::Cancel),
            Type::Boolean,
        ),
        ("neoCLR.Runtime.SocketDeadlineAfter", [Type::Int32]) => (
            Binding::Socket(crate::socket_io::Operation::DeadlineAfter),
            Type::Int64,
        ),
        ("neoCLR.Runtime.SocketDeadlineExpired", [Type::Int64]) => (
            Binding::Socket(crate::socket_io::Operation::DeadlineExpired),
            Type::Boolean,
        ),
        ("neoCLR.Runtime.DnsLookupUntil", [Type::String, Type::Int64, callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Resolve(crate::name_resolution::Operation::LookupUntil),
                Type::Value,
            )
        }
        (
            "neoCLR.Runtime.SocketConnectAddressesUntil",
            [addresses, Type::Int32, Type::Int64, callback],
        ) if *addresses == Type::ArrayRef(Box::new(Type::String))
            && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::ConnectAddressesUntil),
                Type::Value,
            )
        }
        (
            "neoCLR.Runtime.SocketReceiveUntil",
            [
                Type::Int64,
                buffer,
                Type::Int32,
                Type::Int32,
                Type::Int64,
                callback,
            ],
        ) if *buffer == crate::assembler::parse_type("arrayref<Byte>")?
            && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::ReceiveUntil),
                Type::Value,
            )
        }
        (
            "neoCLR.Runtime.SocketSendUntil",
            [
                Type::Int64,
                buffer,
                Type::Int32,
                Type::Int32,
                Type::Int64,
                callback,
            ],
        ) if *buffer == crate::assembler::parse_type("arrayref<Byte>")?
            && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::SendUntil),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.DnsLookup", [Type::String, callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Resolve(crate::name_resolution::Operation::Lookup),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.DnsResult", [Type::Int64]) => (
            Binding::Resolve(crate::name_resolution::Operation::Result),
            Type::Value,
        ),
        ("neoCLR.Runtime.SocketListen", [Type::String, Type::Int32, Type::Int32]) => (
            Binding::Socket(crate::socket_io::Operation::Listen),
            Type::Value,
        ),
        ("neoCLR.Runtime.SocketAccept", [Type::Int64, callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::Accept),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.SocketWaitReadClosure", [Type::Int64, callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::WaitReadClosure),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.SocketLocalPort", [Type::Int64]) => (
            Binding::Socket(crate::socket_io::Operation::LocalPort),
            Type::Value,
        ),
        ("neoCLR.Runtime.SocketConnect", [Type::String, Type::Int32, callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::Connect),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.SocketConnectAddresses", [addresses, Type::Int32, callback])
            if *addresses == Type::ArrayRef(Box::new(Type::String))
                && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::ConnectAddresses),
                Type::Value,
            )
        }
        (
            "neoCLR.Runtime.SocketReceive",
            [Type::Int64, buffer, Type::Int32, Type::Int32, callback],
        ) if *buffer == crate::assembler::parse_type("arrayref<Byte>")?
            && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::Receive),
                Type::Value,
            )
        }
        (
            "neoCLR.Runtime.SocketSend",
            [Type::Int64, buffer, Type::Int32, Type::Int32, callback],
        ) if *buffer == crate::assembler::parse_type("arrayref<Byte>")?
            && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::Send),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.SocketReceive", [Type::Int64, buffer, callback])
            if *buffer == crate::file_streams::memory_type(false)
                && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::Receive),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.SocketReceiveUntil", [Type::Int64, buffer, Type::Int64, callback])
            if *buffer == crate::file_streams::memory_type(false)
                && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::ReceiveUntil),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.SocketSend", [Type::Int64, buffer, callback])
            if *buffer == crate::file_streams::memory_type(true)
                && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::Send),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.SocketSendUntil", [Type::Int64, buffer, Type::Int64, callback])
            if *buffer == crate::file_streams::memory_type(true)
                && *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (
                Binding::Socket(crate::socket_io::Operation::SendUntil),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.SocketConnectResult", [Type::Int64]) => (
            Binding::Socket(crate::socket_io::Operation::ConnectResult),
            Type::Value,
        ),
        ("neoCLR.Runtime.SocketTransferResult", [Type::Int64]) => (
            Binding::Socket(crate::socket_io::Operation::TransferResult),
            Type::Value,
        ),
        ("neoCLR.Runtime.SocketCancel", [Type::Int64]) => (
            Binding::Socket(crate::socket_io::Operation::Cancel),
            Type::Boolean,
        ),
        ("neoCLR.Runtime.DnsCancel", [Type::Int64]) => (
            Binding::Resolve(crate::name_resolution::Operation::Cancel),
            Type::Boolean,
        ),
        ("neoCLR.Runtime.SocketClose", [Type::Int64]) => (
            Binding::Socket(crate::socket_io::Operation::Close),
            Type::Value,
        ),
        (
            "neoCLR.Runtime.FileWatch",
            [
                Type::Int32,
                Type::String,
                Type::String,
                Type::Int64,
                Type::Int32,
            ],
        ) => (
            Binding::FileResource(crate::file_streams::Operation::Watch),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileOpenRead", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::OpenRead),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileOpenWrite", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::OpenWrite),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileCreateNew", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::CreateNew),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileReadChunk", [Type::Int32, Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::Read),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileReadInto", [Type::Int32, memory])
            if *memory == crate::file_streams::memory_type(false) =>
        {
            (
                Binding::FileResource(crate::file_streams::Operation::ReadInto),
                Type::Value,
            )
        }
        ("neoCLR.Runtime.FileWriteChunk", [Type::Int32, memory])
            if *memory == crate::file_streams::memory_type(true) =>
        {
            (
                Binding::FileResource(crate::file_streams::Operation::Write),
                Type::Value,
            )
        }
        // Four-argument forms remain private legacy bootstrap/test bindings.
        (
            "neoCLR.Runtime.FileReadInto",
            [
                Type::Int32,
                Type::ArrayRef(element),
                Type::Int32,
                Type::Int32,
            ],
        ) if **element == Type::Byte => (
            Binding::FileResource(crate::file_streams::Operation::ReadInto),
            Type::Value,
        ),
        (
            "neoCLR.Runtime.FileWriteChunk",
            [
                Type::Int32,
                Type::ArrayRef(element),
                Type::Int32,
                Type::Int32,
            ],
        ) if **element == Type::Byte => (
            Binding::FileResource(crate::file_streams::Operation::Write),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileFlush", [Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::Flush),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileClose", [Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::Close),
            Type::Value,
        ),
        ("neoCLR.Runtime.FilePosition", [Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::Position),
            Type::Value,
        ),
        ("neoCLR.Runtime.FileSeek", [Type::Int32, Type::Int64]) => (
            Binding::FileResource(crate::file_streams::Operation::Seek),
            Type::Value,
        ),
        ("neoCLR.Runtime.StorageNames" | "neoCLR.Runtime.DnsAddresses", [Type::Value]) => (
            Binding::StringSnapshot,
            Type::ArrayRef(Box::new(Type::String)),
        ),
        ("neoCLR.Runtime.StorageList", [Type::String, Type::Int32]) => (
            Binding::FileResource(crate::file_streams::Operation::List),
            Type::Value,
        ),
        ("neoCLR.Runtime.StorageKind", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::Kind),
            Type::Value,
        ),
        ("neoCLR.Runtime.StorageCreateDirectory", [Type::String]) => (
            Binding::FileResource(crate::file_streams::Operation::CreateDirectory),
            Type::Value,
        ),
        ("neoCLR.Runtime.WriteAllText", [Type::String, Type::String, Type::Int32]) => {
            (Binding::WriteAllText, Type::Int32)
        }
        ("neoCLR.Runtime.ReadAllText", [Type::String, Type::Int32]) => {
            (Binding::ReadAllText, Type::Value)
        }
        (
            "neoCLR.Runtime.ConsoleWriteBytes",
            [
                Type::Boolean,
                Type::ArrayRef(element),
                Type::Int32,
                Type::Int32,
            ],
        ) if **element == Type::Byte => (Binding::ConsoleWriteBytes, Type::Value),
        ("neoCLR.Runtime.ConsoleWriteBytes", [Type::Boolean, source])
            if *source == crate::file_streams::memory_type(true)
                || *source == crate::file_streams::console_slice_type() =>
        {
            (Binding::ConsoleWriteBytes, Type::Value)
        }
        ("neoCLR.Runtime.ConsoleFlush", [Type::Boolean]) => (Binding::ConsoleFlush, Type::Value),
        ("neoCLR.Runtime.ConsoleTerminalOpen", [Type::Boolean]) => {
            (Binding::ConsoleTerminalOpen, Type::Int32)
        }
        ("neoCLR.Runtime.ConsoleTerminalReadByte", [Type::Int32, Type::Int32]) => {
            (Binding::ConsoleTerminalReadByte, Type::Int32)
        }
        ("neoCLR.Runtime.ConsoleTerminalClose", [Type::Int32]) => {
            (Binding::ConsoleTerminalClose, Type::Int32)
        }
        ("neoCLR.Runtime.ConsoleReadByte", []) => (Binding::ConsoleReadByte, Type::Value),
        ("neoCLR.Runtime.StartWorker", [callback, Type::String])
            if *callback == crate::assembler::parse_type("fn<String,String>")? =>
        {
            (Binding::StartWorker(false), Type::Int32)
        }
        ("neoCLR.Runtime.QueueWorker", [callback, Type::String])
            if *callback == crate::assembler::parse_type("fn<String,String>")? =>
        {
            (Binding::StartWorker(true), Type::Int32)
        }
        ("neoCLR.Runtime.RequestWorkerCancellation", [Type::Int32]) => {
            (Binding::RequestWorkerCancellation, Type::Boolean)
        }
        ("neoCLR.Runtime.JoinWorkerResult", [Type::Int32]) => {
            (Binding::JoinWorkerResult, Type::Value)
        }
        #[cfg(test)]
        (
            "neoCLR.Runtime.TestSocketReceive",
            [Type::ArrayRef(element), Type::Int32, Type::Int32, callback],
        ) if **element == Type::Byte && callback == &crate::assembler::parse_type("fn<Void>")? => {
            (Binding::TestSocketReceive, Type::Void)
        }
        ("neoCLR.Runtime.JoinWorker", [Type::Int32]) => (Binding::JoinWorker, Type::String),
        ("neoCLR.Runtime.NotifyWorker", [Type::Int32, callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (Binding::NotifyWorker, Type::Void)
        }
        ("neoCLR.Runtime.SchedulerRegister", [Type::Int64, callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (Binding::SchedulerRegister, Type::Int64)
        }
        ("neoCLR.Runtime.SchedulerActivate", [Type::Int64]) => {
            (Binding::SchedulerActivate, Type::Boolean)
        }
        ("neoCLR.Runtime.SchedulerCancel", [Type::Int64]) => {
            (Binding::SchedulerCancel, Type::Boolean)
        }
        ("neoCLR.Runtime.SchedulerCapture", []) => (Binding::SchedulerCapture, Type::Int64),
        ("neoCLR.Runtime.SchedulerSubmitTo", [Type::Int64, callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (Binding::SchedulerSubmitTo, Type::Int32)
        }
        ("neoCLR.Runtime.SchedulerSubmit", [callback])
            if *callback == crate::assembler::parse_type("fn<Void>")? =>
        {
            (Binding::SchedulerSubmit, Type::Int32)
        }
        ("neoCLR.Runtime.ScheduleTask", [callback])
            if *callback == crate::assembler::parse_type("fn<Void>")?
                || *callback == crate::assembler::parse_type("fn<void>")? =>
        {
            (Binding::ScheduleTask, Type::Void)
        }
        ("neoCLR.Runtime.RuntimeAsyncStart", [body])
            if *body == crate::assembler::parse_type("fn<Int32>")?
                || *body == crate::assembler::parse_type("fn<Void>")? =>
        {
            (Binding::RuntimeAsyncStart, Type::Int64)
        }
        ("neoCLR.Runtime.RuntimeAsyncStartManaged", [owner, body])
            if *owner == Type::Named("System.Object".into())
                && *body == crate::assembler::parse_type("fn<Void>")? =>
        {
            (Binding::RuntimeAsyncStartManaged, Type::Int64)
        }
        ("neoCLR.Runtime.RuntimeAsyncCreateOwned", [owner])
            if *owner == Type::Named("System.Object".into()) =>
        {
            (Binding::RuntimeAsyncCreateOwned, Type::Int64)
        }
        ("neoCLR.Runtime.RuntimeAsyncCurrentOwner", []) => (
            Binding::RuntimeAsyncCurrentOwner,
            Type::Named("System.Object".into()),
        ),
        ("neoCLR.Runtime.RuntimeAsyncCreate", []) => (Binding::RuntimeAsyncCreate, Type::Int64),
        ("neoCLR.Runtime.RuntimeAsyncDelay", [Type::Int32]) => {
            (Binding::RuntimeAsyncDelay, Type::Int64)
        }
        ("neoCLR.Runtime.RuntimeAsyncComplete", [Type::Int64, Type::Int32]) => {
            (Binding::RuntimeAsyncComplete, Type::Boolean)
        }
        ("neoCLR.Runtime.RuntimeAsyncCompleteUnit", [Type::Int64]) => {
            (Binding::RuntimeAsyncCompleteUnit, Type::Boolean)
        }
        ("neoCLR.Runtime.RuntimeAsyncAwaitUnit", [Type::Int64]) => {
            (Binding::RuntimeAsyncAwaitUnit, Type::Void)
        }
        ("neoCLR.Runtime.RuntimeAsyncCancel", [Type::Int64]) => {
            (Binding::RuntimeAsyncCancel, Type::Boolean)
        }
        ("neoCLR.Runtime.RuntimeAsyncRelease", [Type::Int64]) => {
            (Binding::RuntimeAsyncRelease, Type::Boolean)
        }
        ("neoCLR.Runtime.RuntimeAsyncIsCompleted", [Type::Int64]) => {
            (Binding::RuntimeAsyncIsCompleted, Type::Boolean)
        }
        ("neoCLR.Runtime.RuntimeAsyncIsCancelled", [Type::Int64]) => {
            (Binding::RuntimeAsyncIsCancelled, Type::Boolean)
        }
        ("neoCLR.Runtime.RuntimeAsyncWaitCompletion", [Type::Int64]) => {
            (Binding::RuntimeAsyncWaitCompletion, Type::Boolean)
        }
        ("neoCLR.Runtime.RuntimeAsyncAwait", [Type::Int64]) => {
            (Binding::RuntimeAsyncAwait, Type::Int32)
        }
        ("neoCLR.Runtime.RuntimeAsyncYield", []) => (Binding::RuntimeAsyncYield, Type::Void),
        ("neoCLR.Runtime.DrainEntryTasks", []) => (Binding::DrainEntryTasks, Type::Void),
        ("neoCLR.Runtime.DefaultTaskQueue", []) => (
            Binding::DefaultTaskQueue,
            Type::from_name("System.Concurrency.Tasks.TaskQueue"),
        ),
        ("neoCLR.Runtime.RegisterDefaultTaskQueue", [queue])
            if queue == &Type::from_name("System.Concurrency.Tasks.TaskQueue") =>
        {
            (Binding::RegisterDefaultTaskQueue, Type::Void)
        }
        ("neoCLR.Runtime.CurrentTaskQueue", []) => (
            Binding::CurrentTaskQueue,
            Type::from_name("System.Concurrency.Tasks.TaskQueue"),
        ),
        ("neoCLR.Runtime.ExecutingAssembly", []) => (
            Binding::ExecutingAssembly,
            Type::from_name("System.Introspection.AssemblyInfo"),
        ),
        ("neoCLR.Runtime.ObjectEquals", [Type::Named(left), Type::Named(right)])
            if left == "System.Object" && right == "System.Object" =>
        {
            (Binding::ObjectEquals, Type::Boolean)
        }
        ("neoCLR.Runtime.ObjectReferenceEquals", [Type::Named(left), Type::Named(right)])
            if left == "System.Object" && right == "System.Object" =>
        {
            (Binding::ObjectReferenceEquals, Type::Boolean)
        }
        ("neoCLR.Runtime.ObjectIdentityHash", [Type::Named(name)]) if name == "System.Object" => {
            (Binding::ObjectIdentityHash, Type::Int32)
        }
        ("neoCLR.Runtime.ObjectTypeHandle", [Type::Named(name)]) if name == "System.Object" => {
            (Binding::ObjectTypeHandle, Type::RuntimeTypeHandle)
        }
        ("neoCLR.Runtime.TypeName", [Type::RuntimeTypeHandle]) => (Binding::TypeName, Type::String),
        (
            "neoCLR.Runtime.ReflectionSubclass",
            [Type::RuntimeTypeHandle, Type::RuntimeTypeHandle],
        ) => (Binding::ReflectionSubclass, Type::Boolean),
        ("neoCLR.Runtime.TypeEquals", [Type::RuntimeTypeHandle, Type::RuntimeTypeHandle]) => {
            (Binding::TypeEquals, Type::Boolean)
        }
        ("neoCLR.Runtime.TypeArgumentCount", [Type::RuntimeTypeHandle]) => {
            (Binding::TypeArgumentCount, Type::Int32)
        }
        ("neoCLR.Runtime.TypeArgument", [Type::RuntimeTypeHandle, Type::Int32]) => {
            (Binding::TypeArgument, Type::RuntimeTypeHandle)
        }
        _ => {
            return Err(Fault::new(format!(
                "no runtime binding for {}({:?})",
                function.name, function.parameters
            )));
        }
    };
    let array_reference_result = matches!(
        binding,
        Binding::Utf8Encode
            | Binding::StringGraphemes
            | Binding::StringScalars
            | Binding::UnixTimeToLocal
    ) && matches!((&returns, &function.returns), (Type::Array(expected), Type::ArrayRef(actual)) if expected == actual);
    if function.returns != returns && !array_reference_result {
        return Err(Fault::new(format!(
            "runtime binding return type mismatch for {}",
            function.name
        )));
    }
    Ok(binding)
}

impl Binding {
    // Keep the native invocation boundary explicit; these are independent host capabilities.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn invoke(
        &self,
        args: Vec<Value>,
        executing_assembly: Option<&str>,
        module: &crate::Module,
        limits: &crate::Limits,
        output: &mut Vec<String>,
        console_bytes: &mut [Vec<u8>; 2],
        options: &crate::ExecutionOptions,
    ) -> Result<Value, Fault> {
        // HostCall normally snapshots under heap access. Keep direct invocation
        // consistent for embedders/tests that already hold graph access.
        let args = if matches!(self, Self::ConsoleWriteBytes) && args.len() == 2 {
            let view = crate::file_streams::console_bytes(&args[1])?;
            if view.length() > crate::file_streams::MAX_TRANSFER {
                return Ok(Value::Erased(Box::new(Value::Byte(8))));
            }
            vec![
                args[0].clone(),
                crate::file_streams::snapshot(view)?,
                Value::Int32(0),
                Value::Int32(view.length() as i32),
            ]
        } else {
            args
        };
        let console = options.console.as_deref();
        if let Self::Cryptography(operation) = self {
            return operation.invoke(&args);
        }
        if let Self::Math(operation) = self {
            return operation.invoke(&args);
        }
        if let Self::AssemblyInfo(query) = self {
            return query.invoke(module, &args, limits);
        }
        if let Self::ExecutingAssembly = self {
            return crate::assembly_info::assembly_value(
                module,
                executing_assembly.ok_or_else(|| {
                    Fault::new("ExecutingAssembly requires source origin metadata")
                })?,
            );
        }
        if let Self::Reflection(query) = self {
            return query.invoke(module, &args, limits);
        }
        match (self, args.as_slice()) {
            (Self::GcKeepAlive, [_]) => Ok(Value::Void),
            (Self::ReflectionSubclass, [source, target]) => {
                let source = crate::reflection_execution::bound_type(module, source)
                    .map_err(|_| Fault::new("invalid source type handle"))?;
                let target = crate::reflection_execution::bound_type(module, target)
                    .map_err(|_| Fault::new("invalid target type handle"))?;
                Ok(Value::Boolean(crate::type_relationships::subclass(
                    module, &source, &target,
                )?))
            }
            (Self::ReflectionAssignable, args) => Ok(Value::Boolean(
                crate::reflection_members::assignable(module, args),
            )),
            (Self::ReflectionMemberCheck(kind), args) => Ok(Value::Int32(
                crate::reflection_members::check(module, args, *kind),
            )),
            (Self::ReflectionPropertyCheck(setter), args) => Ok(Value::Int32(
                crate::reflection_properties::check(module, args, *setter),
            )),
            (Self::ReflectionConstructionCheck, [handle]) => Ok(Value::Int32(
                crate::reflection_execution::check(module, handle),
            )),
            (Self::ObjectEquals, [left, right]) => {
                crate::object_identity::equals(left, right).map(Value::Boolean)
            }
            (Self::ObjectReferenceEquals, [left, right]) => {
                crate::object_identity::reference_equals(left, right).map(Value::Boolean)
            }
            (Self::ObjectIdentityHash, [value]) => {
                crate::object_identity::hash(value).map(Value::Int32)
            }
            (Self::FunctionInfo, [Value::Function(binding)]) => {
                let target = crate::vm::resolve(module, binding.managed_target()?)?;
                crate::reflection::bound_function(module, binding, &target, limits)
            }
            (Self::FunctionInfo, [Value::NullObjectReference(_)]) => Err(Fault::coded(
                crate::FaultCode::NullReference,
                "Function inspection requires a non-null function value",
            )),
            (Self::ObjectTypeHandle, [value]) => {
                let concrete = match value {
                    Value::ObjectReference(object) => {
                        object.reference.assigned()?;
                        object.concrete_type()
                    }
                    Value::String(_) => Type::String,
                    Value::Function(binding) => binding.ty.clone(),
                    Value::NullObjectReference(_) => {
                        return Err(Fault::coded(
                            crate::FaultCode::NullReference,
                            "GetType requires a non-null instance",
                        ));
                    }
                    _ => return Err(Fault::new("GetType requires an object reference")),
                };
                Ok(Value::RuntimeTypeHandle(Box::new(
                    crate::type_identity::describe_loaded(module, &concrete)?,
                )))
            }
            (Self::TimeZoneExists, [Value::String(id)]) => {
                Ok(Value::Boolean(crate::time_zones::exists(id)))
            }
            (Self::TimeZoneOffset, [Value::String(id), Value::Int64(ticks)]) => {
                Ok(Value::Int32(crate::time_zones::offset(id, *ticks)))
            }
            (Self::TimeZoneMapLocal, [Value::String(id), Value::Int64(ticks)]) => {
                Ok(crate::time_zones::map_local(id, *ticks))
            }
            (Self::TimeZoneDatabaseVersion, []) => {
                Ok(Value::String(chrono_tz::IANA_TZDB_VERSION.into()))
            }
            (Self::SystemTimeZoneName, []) => Ok(Value::String(
                iana_time_zone::get_timezone().unwrap_or_default().into(),
            )),
            (Self::SystemCultureName, []) => Ok(Value::String(
                sys_locale::get_locale().unwrap_or_default().into(),
            )),
            (Self::EnvironmentPlatformInfo, [Value::Int32(selector)]) => Ok(Value::String(
                crate::environment_platform::read(*selector).into(),
            )),
            (Self::EnvironmentArguments, []) => Ok(Value::Array {
                element: Type::String,
                elements: options
                    .arguments
                    .iter()
                    .cloned()
                    .map(|text| Value::String(text.into()))
                    .collect(),
            }),
            (Self::EnvironmentCurrentDirectory, []) => {
                let payload = std::env::current_dir()
                    .ok()
                    .and_then(|p| p.into_os_string().into_string().ok())
                    .map(|text| Value::String(text.into()))
                    .unwrap_or(Value::Int32(1));
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::EnvironmentVariable, [Value::String(name)]) => {
                let payload = if name.is_empty() || name.contains(['=', '\0']) {
                    Value::Int32(1)
                } else {
                    match std::env::var(name.as_str()) {
                        Ok(value) => Value::String(value.into()),
                        Err(std::env::VarError::NotPresent) => Value::Void,
                        Err(std::env::VarError::NotUnicode(_)) => Value::Int32(1),
                    }
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::PathCombine, [Value::String(left), Value::String(right)]) => {
                Ok(Value::String(crate::path::combine(left, right).into()))
            }
            (Self::PathGetFileName, [Value::String(path)]) => {
                Ok(Value::String(crate::path::file_name(path).into()))
            }
            (Self::UnixTimeToLocal, [Value::Int64(ticks)]) => crate::clock::local_at(*ticks),
            (Self::UnixTimeTicks, []) => crate::clock::read_instant(),
            (Self::TypeName, [Value::RuntimeTypeHandle(handle)]) => {
                Ok(Value::String(handle.name.clone().into()))
            }
            (
                Self::TypeEquals,
                [
                    Value::RuntimeTypeHandle(left),
                    Value::RuntimeTypeHandle(right),
                ],
            ) => Ok(Value::Boolean(left.identity == right.identity)),
            (Self::TypeArgumentCount, [Value::RuntimeTypeHandle(handle)]) => {
                Ok(Value::Int32(handle.generic_arguments.len() as i32))
            }
            (Self::TypeArgument, [Value::RuntimeTypeHandle(handle), Value::Int32(index)]) => {
                let argument = usize::try_from(*index)
                    .ok()
                    .and_then(|index| handle.generic_arguments.get(index))
                    .ok_or_else(|| Fault::new("generic argument index out of range"))?;
                Ok(Value::RuntimeTypeHandle(Box::new(argument.clone())))
            }
            (
                Self::ConsoleWriteBytes,
                [
                    Value::Boolean(error),
                    array,
                    Value::Int32(offset),
                    Value::Int32(count),
                ],
            ) => {
                let Value::Array {
                    element: Type::Byte,
                    elements,
                } = (match array {
                    Value::ObjectReference(array) => array.reference.read()?,
                    Value::Array { .. } => array.clone(),
                    _ => return Err(Fault::new("write requires a byte array")),
                })
                else {
                    return Err(Fault::new("Console write requires a byte array"));
                };
                let payload = match (usize::try_from(*offset), usize::try_from(*count)) {
                    (Ok(offset), Ok(count))
                        if offset <= elements.len() && count <= elements.len() - offset =>
                    {
                        if count > 65536 {
                            Value::Byte(8)
                        } else if count == 0 {
                            Value::Int32(0)
                        } else {
                            let bytes = elements[offset..offset + count]
                                .iter()
                                .map(|v| match v {
                                    Value::Byte(b) => Ok(*b),
                                    _ => {
                                        Err(Fault::new("Console write requires initialized bytes"))
                                    }
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            match console {
                                Some(host) => match host.write_bytes(*error, &bytes) {
                                    Ok(written) if written <= count => Value::Int32(written as i32),
                                    _ => Value::Byte(10),
                                },
                                None => {
                                    console_bytes[usize::from(*error)].extend_from_slice(&bytes);
                                    Value::Int32(count as i32)
                                }
                            }
                        }
                    }
                    _ => Value::Byte(7),
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::ConsoleFlush, [Value::Boolean(error)]) => {
                let payload = match console {
                    Some(host) if host.flush(*error).is_err() => Value::Byte(10),
                    _ => Value::Int32(0),
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::ConsoleReadByte, []) => {
                // Byte = data, Void = EOF; Int32 1 = Unavailable, 2 = ReadFailed.
                // A worker's bounded output sink does not provide console input.
                let console = if crate::workers::is_worker() {
                    None
                } else {
                    console
                };
                let payload = match console {
                    None => Value::Int32(1),
                    Some(console) => match console.read_byte() {
                        Ok(Some(byte)) => Value::Byte(byte),
                        Ok(None) => Value::Void,
                        Err(_) => Value::Int32(2),
                    },
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (
                Self::WriteAllText,
                [
                    Value::String(path),
                    Value::String(text),
                    Value::Int32(limit),
                ],
            ) => Ok(Value::Int32(crate::file_io::write_all_text(
                path, text, *limit,
            ))),
            (Self::ReadAllText, [Value::String(path), Value::Int32(max_bytes)]) => {
                crate::file_io::read_all_text(path, *max_bytes)
            }
            (Self::ParseInt32, [Value::String(text)]) => {
                // Validate the whole grammar first: malformed text wins over overflow.
                // Byte 1 = InvalidFormat, Byte 2 = Overflow; Int32 = success.
                let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
                let payload = if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                    Value::Byte(1)
                } else {
                    match text.parse::<i32>() {
                        Ok(n) => Value::Int32(n),
                        Err(_) => Value::Byte(2),
                    }
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::ParseNumber(kind), [Value::String(text)]) => {
                Ok(crate::numeric_parse::parse(text, *kind))
            }
            (Self::ParseInt64, [Value::String(text)]) => {
                // Validate the whole grammar first: malformed text wins over overflow.
                // Byte 1 = InvalidFormat, Byte 2 = Overflow; Int64 = success.
                let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
                let payload = if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                    Value::Byte(1)
                } else {
                    match text.parse::<i64>() {
                        Ok(n) => Value::Int64(n),
                        Err(_) => Value::Byte(2),
                    }
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::StringCasing(uppercase), [Value::String(text)]) => Ok(Value::String(
                crate::string_casing::convert(text, *uppercase).into(),
            )),
            (Self::NativeMultiplyChecked, [Value::UIntPtr(left), Value::UIntPtr(right)]) => {
                left.checked_mul(*right).map(Value::UIntPtr).ok_or_else(|| {
                    Fault::coded(
                        crate::FaultCode::ArithmeticOverflow,
                        "native allocation size overflow",
                    )
                })
            }
            (Self::NativeIntegerTo64, [Value::IntPtr(number)]) => Ok(Value::Int64(*number as i64)),
            (Self::NativeIntegerTo64, [Value::UIntPtr(number)]) => {
                Ok(Value::UInt64(*number as u64))
            }
            (Self::Int32ToString, [Value::Int32(number)]) => {
                Ok(Value::String(number.to_string().into()))
            }
            (
                Self::FloatingFormat(single),
                [
                    value,
                    Value::Int32(code),
                    Value::Int32(precision),
                    Value::Boolean(swedish),
                ],
            ) => {
                let value = match value {
                    Value::Single(v) => *v as f64,
                    Value::Double(v) => *v,
                    _ => return Err(Fault::new("invalid floating format")),
                };
                let buffer = crate::numeric_format_core::format(
                    value,
                    *single,
                    *code as u8,
                    *precision,
                    *swedish,
                )
                .map_err(|_| Fault::new("invalid floating format"))?;
                Ok(Value::String(
                    std::str::from_utf8(&buffer.bytes[..buffer.len])
                        .unwrap()
                        .to_owned()
                        .into(),
                ))
            }
            (Self::IntegerToString, [Value::Int64(number)]) => {
                Ok(Value::String(number.to_string().into()))
            }
            (Self::IntegerToString, [Value::UInt64(number)]) => {
                Ok(Value::String(number.to_string().into()))
            }
            (Self::Fault, [Value::String(message)]) => {
                Err(Fault::coded(crate::FaultCode::UserFault, message.as_str()))
            }
            (Self::WriteLine, [Value::String(text)]) => {
                if let Some(console) = console {
                    console
                        .write_line(text)
                        .map_err(|_| Fault::new("console output failed"))?;
                } else {
                    output.push(text.as_str().to_owned());
                    console_bytes[0].extend_from_slice(text.as_bytes());
                    console_bytes[0].push(b'\n');
                }
                Ok(Value::Void)
            }
            (Self::CharCategory, [Value::UInt32(value)]) => {
                char::from_u32(*value).ok_or_else(|| Fault::new("invalid Unicode scalar"))?;
                let ranges = crate::char_categories::RANGES;
                let index = ranges.partition_point(|(end, _)| end < value);
                Ok(Value::Int32(i32::from(ranges[index].1)))
            }
            (Self::Utf8Encode, [Value::String(text)]) => {
                crate::reflection::array("Byte", text.bytes().map(|b| Ok(Value::Byte(b))), limits)
            }
            (Self::Utf8DecodeSlice, [Value::MemoryView(view)]) => {
                if view.ty() != crate::cryptography::view_type(true) {
                    return Err(Fault::new("UTF-8 decoding requires a readonly byte slice"));
                }
                view.assigned()?;
                // Interpreter arrays store Values, not contiguous native bytes. Build
                // the owned string buffer directly; no managed temporary byte array.
                let mut buffer = Vec::new();
                buffer
                    .try_reserve_exact(view.length())
                    .map_err(|_| Fault::new("UTF-8 decoding allocation failed"))?;
                for index in 0..view.length() {
                    let Value::Byte(byte) = view.get(index)? else {
                        return Err(Fault::new("UTF-8 decoding requires initialized bytes"));
                    };
                    buffer.push(byte);
                }
                let payload = match String::from_utf8(buffer) {
                    Ok(text) => Value::String(text.into()),
                    Err(_) => Value::Byte(1),
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::Utf8Decode, [Value::ObjectReference(bytes)]) => {
                let Value::Array {
                    element: Type::Byte,
                    elements,
                } = bytes.reference.read()?
                else {
                    return Err(Fault::new("UTF-8 decoding requires a byte array"));
                };
                let mut buffer = Vec::new();
                buffer
                    .try_reserve_exact(elements.len())
                    .map_err(|_| Fault::new("UTF-8 decoding allocation failed"))?;
                for value in elements {
                    let Value::Byte(byte) = value else {
                        return Err(Fault::new("UTF-8 decoding requires initialized bytes"));
                    };
                    buffer.push(byte);
                }
                let payload = match String::from_utf8(buffer) {
                    Ok(text) => Value::String(text.into()),
                    Err(_) => Value::Byte(1),
                };
                Ok(Value::Erased(Box::new(payload)))
            }
            (Self::StringFromChars, [Value::ObjectReference(chars)]) => {
                let Value::Array {
                    element: Type::Char,
                    elements,
                } = chars.reference.read()?
                else {
                    return Err(Fault::new("String construction requires a Char array"));
                };
                let mut length = 0usize;
                for element in &elements {
                    element.initialized()?;
                    let Value::Char(text) = element else {
                        return Err(Fault::new(
                            "String construction requires initialized characters",
                        ));
                    };
                    length = length
                        .checked_add(text.len())
                        .ok_or_else(|| Fault::new("string size overflow"))?;
                }
                let mut text = String::new();
                text.try_reserve_exact(length)
                    .map_err(|_| Fault::new("string allocation failed"))?;
                for element in elements {
                    let Value::Char(character) = element else {
                        unreachable!()
                    };
                    text.push_str(&character);
                }
                Ok(Value::String(text.into()))
            }
            (
                Self::StringSliceGraphemes,
                [
                    Value::String(text),
                    Value::Int32(start),
                    Value::Int32(count),
                ],
            ) => {
                let invalid = || {
                    Fault::coded(
                        crate::FaultCode::IndexOutOfRange,
                        "String range is out of bounds",
                    )
                };
                let start = usize::try_from(*start).map_err(|_| invalid())?;
                let count = usize::try_from(*count).map_err(|_| invalid())?;
                let end = start.checked_add(count).ok_or_else(invalid)?;
                let mut offset = None;
                let mut limit = None;
                for (position, at) in text
                    .grapheme_indices(true)
                    .map(|(at, _)| at)
                    .chain(std::iter::once(text.len()))
                    .enumerate()
                {
                    if position == start {
                        offset = Some(at);
                    }
                    if position == end {
                        limit = Some(at);
                        break;
                    }
                }
                let (offset, limit) = (offset.ok_or_else(invalid)?, limit.ok_or_else(invalid)?);
                Ok(Value::String(text[offset..limit].to_owned().into()))
            }
            (Self::StringGraphemeAt, [Value::String(text), Value::Int32(index)]) => {
                let index = usize::try_from(*index).map_err(|_| {
                    Fault::coded(
                        crate::FaultCode::IndexOutOfRange,
                        "String index is out of range",
                    )
                })?;
                let character = text.graphemes(true).nth(index).ok_or_else(|| {
                    Fault::coded(
                        crate::FaultCode::IndexOutOfRange,
                        "String index is out of range",
                    )
                })?;
                Ok(Value::Char(character.to_owned()))
            }
            (
                Self::StringJoinParts,
                [
                    Value::ObjectReference(parts),
                    Value::Int32(count),
                    Value::String(separator),
                    Value::Int32(expected),
                ],
            ) => {
                let Value::Array {
                    element: Type::String,
                    elements,
                } = parts.reference.read()?
                else {
                    return Err(Fault::new("String joining requires a String array"));
                };
                if *count < 0
                    || *expected < 0
                    || elements.len() > 65536
                    || *count as usize > elements.len()
                {
                    return Err(Fault::new("Invalid string joining bounds"));
                }
                let mut length = separator
                    .len()
                    .checked_mul((*count as usize).saturating_sub(1))
                    .filter(|length| *length <= *expected as usize)
                    .ok_or_else(|| Fault::new("String joining byte count mismatch"))?;
                for element in &elements[..*count as usize] {
                    let Value::String(text) = element else {
                        return Err(Fault::new("String joining requires initialized strings"));
                    };
                    length = length
                        .checked_add(text.len())
                        .filter(|length| *length <= *expected as usize)
                        .ok_or_else(|| Fault::new("String joining byte count mismatch"))?;
                }
                if length != *expected as usize {
                    return Err(Fault::new("String joining byte count mismatch"));
                }
                let mut text = String::new();
                text.try_reserve_exact(length).map_err(|_| {
                    Fault::coded(
                        crate::FaultCode::NativeMemoryLimitExceeded,
                        "String joining allocation failed",
                    )
                })?;
                for (index, element) in elements[..*count as usize].iter().enumerate() {
                    if index != 0 {
                        text.push_str(separator);
                    }
                    let Value::String(part) = element else {
                        unreachable!()
                    };
                    text.push_str(part);
                }
                Ok(Value::String(text.into()))
            }
            (Self::StringReplaceLineEndings, [Value::String(text), Value::String(replacement)]) => {
                let invalid =
                    || Fault::coded(crate::FaultCode::RuntimeError, "String size overflow");
                let bytes = text.as_bytes();
                let mut length = bytes.len();
                let mut changed = false;
                let mut at = 0;
                while at < bytes.len() {
                    let width = crate::string_lines::newline_width(&bytes[at..]);
                    if width == 0 {
                        at += 1;
                        continue;
                    }
                    changed |= &bytes[at..at + width] != replacement.as_bytes();
                    length = length
                        .checked_sub(width)
                        .and_then(|n| n.checked_add(replacement.len()))
                        .ok_or_else(invalid)?;
                    at += width;
                }
                if !changed {
                    return Ok(Value::String(text.clone()));
                }
                let mut result = String::new();
                result.try_reserve_exact(length).map_err(|_| invalid())?;
                let mut start = 0;
                at = 0;
                while at < bytes.len() {
                    let width = crate::string_lines::newline_width(&bytes[at..]);
                    if width == 0 {
                        at += 1;
                        continue;
                    }
                    result.push_str(&text[start..at]);
                    result.push_str(replacement);
                    at += width;
                    start = at;
                }
                result.push_str(&text[start..]);
                Ok(Value::String(result.into()))
            }
            (
                Self::StringReplaceOrdinal,
                [Value::String(text), Value::String(old), Value::String(new)],
            ) => {
                let invalid = || Fault::coded(crate::FaultCode::RuntimeError, "Runtime error");
                if old.is_empty() {
                    return Err(invalid());
                }
                let count = text.matches(old.as_ref()).count();
                if count == 0 || old == new {
                    return Ok(Value::String(text.clone()));
                }
                let length = text
                    .len()
                    .checked_sub(count.checked_mul(old.len()).ok_or_else(invalid)?)
                    .and_then(|n| n.checked_add(count.checked_mul(new.len())?))
                    .ok_or_else(invalid)?;
                let mut result = String::new();
                result.try_reserve_exact(length).map_err(|_| invalid())?;
                let mut start = 0;
                for (at, _) in text.match_indices(old.as_ref()) {
                    result.push_str(&text[start..at]);
                    result.push_str(new);
                    start = at + old.len();
                }
                result.push_str(&text[start..]);
                Ok(Value::String(result.into()))
            }
            (Self::StringConcat, [Value::String(left), Value::String(right)]) => {
                let length = left
                    .len()
                    .checked_add(right.len())
                    .ok_or_else(|| Fault::new("string size overflow"))?;
                let mut value = String::new();
                value
                    .try_reserve_exact(length)
                    .map_err(|_| Fault::new("string allocation failed"))?;
                value.push_str(left);
                value.push_str(right);
                Ok(Value::String(value.into()))
            }
            (Self::StringCompareOrdinal, [Value::String(left), Value::String(right)]) => {
                // Valid UTF-8 byte order agrees with Unicode scalar order.
                let order = left.as_bytes().cmp(right.as_bytes());
                Ok(Value::Int32(match order {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }))
            }
            (Self::StringCompareCanonical, [Value::String(left), Value::String(right)]) => Ok(
                Value::Int32(crate::string_normalization::compare(left, right) as i32),
            ),
            (Self::CharCompareCanonical, [Value::Char(left), Value::Char(right)]) => Ok(
                Value::Int32(crate::string_normalization::compare(left, right) as i32),
            ),
            (Self::StringHashCanonical, [Value::String(value)]) => {
                Ok(Value::Int32(crate::string_normalization::hash(value)))
            }
            (Self::CharHashCanonical, [Value::Char(value)]) => {
                Ok(Value::Int32(crate::string_normalization::hash(value)))
            }
            (Self::StringCompareOrdinalIgnoreCase, [Value::String(left), Value::String(right)]) => {
                Ok(Value::Int32(
                    match crate::string_comparison::compare_ignore_case(left, right) {
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    },
                ))
            }
            (Self::StringHashOrdinal, [Value::String(value)]) => {
                Ok(Value::Int32(crate::string_comparison::hash_ordinal(value)))
            }
            (Self::StringHashOrdinalIgnoreCase, [Value::String(value)]) => Ok(Value::Int32(
                crate::string_comparison::hash_ignore_case(value),
            )),
            (Self::StringContainsCanonical, [Value::String(value), Value::String(pattern)]) => Ok(
                Value::Boolean(crate::string_search::contains(value, pattern)),
            ),
            (Self::StringStartsWithCanonical, [Value::String(value), Value::String(pattern)]) => {
                Ok(Value::Boolean(crate::string_search::starts_with(
                    value, pattern,
                )))
            }
            (Self::StringEndsWithCanonical, [Value::String(value), Value::String(pattern)]) => Ok(
                Value::Boolean(crate::string_search::ends_with(value, pattern)),
            ),
            (
                Self::StringContainsOrdinalIgnoreCase,
                [Value::String(value), Value::String(pattern)],
            ) => Ok(Value::Boolean(crate::string_search::contains_ignore_case(
                value, pattern,
            ))),
            (
                Self::StringStartsWithOrdinalIgnoreCase,
                [Value::String(value), Value::String(pattern)],
            ) => Ok(Value::Boolean(
                crate::string_search::starts_with_ignore_case(value, pattern),
            )),
            (
                Self::StringEndsWithOrdinalIgnoreCase,
                [Value::String(value), Value::String(pattern)],
            ) => Ok(Value::Boolean(crate::string_search::ends_with_ignore_case(
                value, pattern,
            ))),
            (Self::StringContainsOrdinal, [Value::String(value), Value::String(pattern)]) => {
                Ok(Value::Boolean(value.contains(pattern.as_str())))
            }
            (Self::StringStartsWithOrdinal, [Value::String(value), Value::String(pattern)]) => {
                Ok(Value::Boolean(value.starts_with(pattern.as_str())))
            }
            (Self::StringEndsWithOrdinal, [Value::String(value), Value::String(pattern)]) => {
                Ok(Value::Boolean(value.ends_with(pattern.as_str())))
            }
            (Self::StringGraphemeCount, [Value::String(value)]) => {
                let count = i32::try_from(value.graphemes(true).count())
                    .map_err(|_| Fault::new("String grapheme count exceeds Int32 range"))?;
                Ok(Value::Int32(count))
            }
            (Self::CharIsSingleGrapheme, [Value::String(value)]) => Ok(Value::Boolean(
                value
                    .graphemes(true)
                    .next()
                    .is_some_and(|first| first.len() == value.len()),
            )),
            (Self::CharCompareOrdinal, [Value::Char(left), Value::Char(right)]) => {
                Ok(Value::Int32(left.as_bytes().cmp(right.as_bytes()) as i32))
            }
            (Self::CharCompareOrdinalIgnoreCase, [Value::Char(left), Value::Char(right)]) => Ok(
                Value::Int32(crate::string_comparison::compare_ignore_case(left, right) as i32),
            ),
            (Self::CharFromString, [Value::String(value)]) => {
                let character = Value::Char(value.as_str().to_owned());
                character.initialized()?;
                Ok(character)
            }
            (Self::CharText, [Value::Char(value)]) => Ok(Value::String(value.clone().into())),
            (Self::StringGraphemes, [Value::String(value)]) => crate::reflection::array(
                "Char",
                value.graphemes(true).map(|g| Ok(Value::Char(g.into()))),
                limits,
            ),
            (Self::StringScalars, [Value::String(value)]) => crate::reflection::array(
                "UInt32",
                value.chars().map(|c| Ok(Value::UInt32(c as u32))),
                limits,
            ),
            (Self::StringByteCount, [Value::String(value)]) => {
                let length = i32::try_from(value.len())
                    .map_err(|_| Fault::new("UTF-8 byte count exceeds Int32 range"))?;
                Ok(Value::Int32(length))
            }
            (
                Self::StringSliceUtf8,
                [
                    Value::String(value),
                    Value::Int32(start),
                    Value::Int32(length),
                ],
            ) => {
                // Explicit internal statuses: 1 = OutOfRange, 2 = InvalidBoundary.
                let error = |status| Value::Erased(Box::new(Value::Byte(status)));
                let (Ok(start), Ok(length)) = (usize::try_from(*start), usize::try_from(*length))
                else {
                    return Ok(error(1));
                };
                let Some(end) = start.checked_add(length).filter(|end| *end <= value.len()) else {
                    return Ok(error(1));
                };
                let Some(slice) = value.get(start..end) else {
                    return Ok(error(2));
                };
                let mut result = String::new();
                result
                    .try_reserve_exact(slice.len())
                    .map_err(|_| Fault::new("string allocation failed"))?;
                result.push_str(slice);
                Ok(Value::Erased(Box::new(Value::String(result.into()))))
            }
            _ => Err(Fault::new("invalid native arguments")),
        }
    }
}

#[cfg(test)]
mod character_tests {
    #[test]
    fn bmp_categories_match_pinned_dotnet_10_probe() {
        // FNV-1a over Char.GetUnicodeCategory for all 65,536 UTF-16 units.
        // docs/experiments/character-classification-dotnet, SDK 10.0.100.
        let ranges = crate::char_categories::RANGES;
        assert_eq!(ranges.last().unwrap().0, 0x10FFFF);
        assert!(ranges.windows(2).all(|pair| pair[0].0 < pair[1].0));
        let mut hash = 14_695_981_039_346_656_037u64;
        for value in 0..=u16::MAX {
            let index = ranges.partition_point(|(end, _)| *end < u32::from(value));
            hash = (hash ^ u64::from(ranges[index].1)).wrapping_mul(1_099_511_628_211);
        }
        assert_eq!(hash, 0x9FB70257D9A6A292);
    }
}

#[cfg(test)]
mod task_callback_tests {
    #[test]
    fn timer_services_require_exact_internal_call_signatures() {
        for (signature, accepted) in [
            (
                "TimerStart(Int32 milliseconds, fn<Void> callback) -> Int64",
                true,
            ),
            ("TimerCancel(Int64 handle) -> Boolean", true),
            (
                "TimerStart(Int64 milliseconds, fn<Void> callback) -> Int64",
                false,
            ),
            (
                "TimerStart(Int32 milliseconds, fn<Int32> callback) -> Int64",
                false,
            ),
            (
                "TimerStart(Int32 milliseconds, fn<Void> callback) -> Int32",
                false,
            ),
            ("TimerCancel(Int32 handle) -> Boolean", false),
        ] {
            let source = format!(
                ".module Test\n.function neoCLR.Runtime.{signature}\n.methodimpl InternalCall\n.end\n"
            );
            assert_eq!(crate::assemble(&source).is_ok(), accepted, "{signature}");
        }
    }

    #[test]
    fn typed_queue_services_require_exact_generic_signatures() {
        for (signature, accepted) in [
            ("GetDefaultTaskQueue<T>() -> T", true),
            ("GetCurrentTaskQueue<T>() -> T", true),
            ("RegisterTaskQueue<T>(T queue) -> Void", true),
            ("GetDefaultTaskQueue<T>() -> Int32", false),
            ("GetDefaultTaskQueue<T, U>() -> T", false),
            ("GetDefaultTaskQueue() -> Int32", false),
            ("RegisterTaskQueue<T>(Int32 queue) -> Void", false),
            ("RegisterTaskQueue<T>(T queue) -> void", true),
            ("GetDefaultTaskQueue<T>() -> void", false),
            ("RegisterTaskQueue<T>(T queue) -> Int32", false),
        ] {
            let module = crate::assemble(&format!(
                ".module Test\n.function neoCLR.Runtime.{signature}\n.methodimpl InternalCall\n.end\n"
            ));
            assert_eq!(module.is_ok(), accepted, "{signature}");
        }
    }

    #[test]
    fn scheduler_destinations_require_exact_contracts() {
        for (signature, accepted) in [
            ("SchedulerCapture() -> Int64", true),
            (
                "SchedulerRegister(Int64 destination, fn<Void> callback) -> Int64",
                true,
            ),
            (
                "SchedulerRegister(Int64 destination, fn<Int32> callback) -> Int64",
                false,
            ),
            (
                "SchedulerRegister(Int64 destination, fn<Void> callback) -> Int32",
                false,
            ),
            ("SchedulerActivate(Int64 registration) -> Boolean", true),
            ("SchedulerCancel(Int64 registration) -> Boolean", true),
            ("SchedulerActivate(Int32 registration) -> Boolean", false),
            ("SchedulerCancel(Int64 registration) -> Int32", false),
            ("SchedulerCapture() -> Int32", false),
            ("SchedulerCapture(Int64 value) -> Int64", false),
            (
                "SchedulerSubmitTo(Int64 destination, fn<Void> callback) -> Int32",
                true,
            ),
            (
                "SchedulerSubmitTo(Int32 destination, fn<Void> callback) -> Int32",
                false,
            ),
            (
                "SchedulerSubmitTo(Int64 destination, fn<void> callback) -> Int32",
                false,
            ),
            (
                "SchedulerSubmitTo(Int64 destination, fn<Void> callback) -> Void",
                false,
            ),
        ] {
            assert_eq!(crate::assemble(&format!(".module Test\n.function neoCLR.Runtime.{signature}\n.methodimpl InternalCall\n.end\n")).is_ok(), accepted, "{signature}");
        }
    }

    #[test]
    fn scheduler_submission_requires_exact_unit_callback_and_status_result() {
        for (signature, accepted) in [
            ("fn<Void> callback) -> Int32", true),
            ("fn<Void> callback) -> Void", false),
            ("fn<void> callback) -> Int32", false),
            ("fn<Int32> callback) -> Int32", false),
            ("fn<Int32,Void> callback) -> Int32", false),
        ] {
            assert_eq!(crate::assemble(&format!(".module Test\n.function neoCLR.Runtime.SchedulerSubmit({signature}\n.methodimpl InternalCall\n.end\n")).is_ok(), accepted);
        }
    }

    #[test]
    fn scheduling_accepts_only_parameterless_unit_or_no_result_callbacks() {
        for (callback, accepted) in [
            ("fn<Void>", true),
            ("fn<void>", true),
            ("fn<Int32>", false),
            ("fn<Int32,Void>", false),
        ] {
            for result in ["Void", "void"] {
                let module = crate::assemble(&format!(
                    ".module Test\n.function neoCLR.Runtime.ScheduleTask({callback} callback) -> {result}\n.methodimpl InternalCall\n.end\n"
                ));
                assert_eq!(module.is_ok(), accepted, "{callback} -> {result}");
            }
        }
    }
}

#[cfg(test)]
mod reflection_signature_tests {
    #[test]
    fn generic_void_service_result_keeps_its_value_convention() {
        let module = crate::assemble(
            ".module Test\n.function neoCLR.Runtime.GetDefaultTaskQueue<T>() -> T\n.methodimpl InternalCall\n.end\n"
        ).unwrap();
        let mut function = module.functions[0].clone();
        function.generic_arguments = vec![crate::metadata::Type::Void];
        function.returns = crate::metadata::Type::Void;
        assert!(super::bind(&function).is_ok());
        function.no_result = true;
        assert!(super::bind(&function).is_err());
    }

    #[test]
    fn no_result_metadata_cannot_discard_a_service_result() {
        for signature in [
            "CharCategory(UInt32 scalar) -> Int32",
            "GetDefaultTaskQueue<T>() -> T",
        ] {
            let module = crate::assemble(&format!(
                ".module Test\n.function neoCLR.Runtime.{signature}\n.methodimpl InternalCall\n.end\n"
            )).unwrap();
            let mut function = module.functions[0].clone();
            function.no_result = true;
            let error = super::bind(&function)
                .err()
                .expect("invalid no-result metadata accepted");
            assert!(error.message.contains("requires a Void return"));
        }
    }

    #[test]
    fn no_result_service_execution_requires_an_admitted_control_signature() {
        for (signature, accepted) in [
            (
                "ReflectionPropertySet(RuntimeTypeHandle owner, Int32 token, System.Object receiver, System.Object value) -> void",
                true,
            ),
            (
                "ReflectionPropertySetCheck(RuntimeTypeHandle owner, Int32 token, System.Object receiver, System.Object value) -> void",
                false,
            ),
            (
                "ReflectionPropertyGet(RuntimeTypeHandle owner, Int32 token, System.Object receiver) -> void",
                false,
            ),
            ("WriteLine(String text) -> void", true),
            ("Fault(String message) -> void", true),
            ("Fault(Int32 message) -> void", false),
            ("NotifyWorker(Int32 worker,fn<Void> callback) -> void", true),
            (
                "NotifyWorker(Int32 worker,fn<Int32> callback) -> void",
                false,
            ),
            (
                "RegisterDefaultTaskQueue(System.Concurrency.Tasks.TaskQueue queue) -> void",
                true,
            ),
            (
                "RegisterDefaultTaskQueue(System.Object queue) -> void",
                false,
            ),
            ("ParseInt32(String text) -> void", false),
            ("Fail(String message) -> void", true),
            ("Fail(String message) -> Void", false),
            ("Fail(Int32 message) -> void", false),
            ("GCCollect() -> void", true),
            ("GCKeepAlive(System.Object value) -> void", true),
        ] {
            let result = crate::assemble(&format!(
                ".module System\n.type class System.Object\n.end\n.type class System.Concurrency.Tasks.TaskQueue\n.end\n.function neoCLR.Runtime.{signature}\n.methodimpl InternalCall\n.end\n"
            ));
            assert_eq!(result.is_ok(), accepted, "{signature}: {result:?}");
        }
    }
}

#[cfg(test)]
mod storage_snapshot_tests {
    use crate::metadata::Type;

    #[test]
    fn storage_names_requires_an_erased_payload_and_managed_string_array() {
        let mut module = crate::assemble(
            ".module Test\n.function neoCLR.Runtime.StorageKind(String path) -> Value\n.methodimpl InternalCall\n.end\n",
        ).unwrap();
        let function = &mut module.functions[0];
        function.name = "neoCLR.Runtime.StorageNames".into();
        for (parameter, result, accepted) in [
            (Type::Value, Type::ArrayRef(Box::new(Type::String)), true),
            (Type::Value, Type::Array(Box::new(Type::String)), false),
            (Type::String, Type::ArrayRef(Box::new(Type::String)), false),
            (Type::Value, Type::ArrayRef(Box::new(Type::Int32)), false),
        ] {
            function.parameters = vec![parameter];
            function.returns = result;
            for name in ["neoCLR.Runtime.StorageNames", "neoCLR.Runtime.DnsAddresses"] {
                function.name = name.into();
                assert_eq!(super::bind(function).is_ok(), accepted, "{name}");
            }
        }
    }
}

#[cfg(test)]
mod local_time_array_tests {
    #[test]
    fn local_time_managed_array_contract_and_execution() {
        for (result, accepted) in [
            ("Int32[]", true),
            ("arrayref<Int32>", true),
            ("arrayref<Int64>", false),
        ] {
            let source = format!(
                ".module Test\n.function neoCLR.Runtime.UnixTimeToLocal(Int64 ticks) -> {result}\n.methodimpl InternalCall\n.end\n"
            );
            let parsed = crate::assembler::parse_module(&source).unwrap();
            let assembled = super::bind(&parsed.functions[0]);
            assert_eq!(
                assembled.is_ok(),
                accepted,
                "{result}: {:?}",
                assembled.as_ref().err()
            );
        }
        let module = crate::assembler::parse_module(".module Test\n.entry Main\n.function neoCLR.Runtime.UnixTimeToLocal(Int64 ticks) -> arrayref<Int32>\n.methodimpl InternalCall\n.end\n.function Main() -> Int32\nldc.i8 0\ncall neoCLR.Runtime.UnixTimeToLocal(Int64)\nldlen\nconv.i4\nret\n.end\n").unwrap();
        let library = crate::assemble(".module System\n").unwrap();
        let program = crate::LoadedProgram::with_library(&module, &library).unwrap();
        program.verify().unwrap();
        assert_eq!(
            program.run(crate::Limits::default()).unwrap().value,
            crate::Value::Int32(8)
        );
    }
}
