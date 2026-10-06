//! Explicit runtime binding registry. A name alone never activates host dispatch.
use crate::{
    Fault, Value,
    metadata::{Function, Type},
};
use unicode_segmentation::UnicodeSegmentation;

pub(crate) enum Binding {
    GcCollect,
    GcInfo(u8),
    GcKeepAlive,
    Socket(crate::socket_io::Operation),
    Resolve(crate::name_resolution::Operation),
    #[cfg(test)]
    TestSocketReceive,
    SystemCultureName,
    TimeZoneExists,
    TimeZoneOffset,
    TimeZoneMapLocal,
    TimeZoneDatabaseVersion,
    SystemTimeZoneName,
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
    ReflectionMember(u8),
    ReflectionMemberCheck(u8),
    ReflectionAssignable,
    ReflectionArray(u8),
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
    ScheduleTask,
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
    NativeIntegerTo64,
    WriteLine,
    Fault,
    CharCategory,
    Utf8Encode,
    Utf8Decode,
    StringConcat,
    StringIntern,
    StringFromChars,
    StringGraphemeAt,
    StringGraphemeCount,
    CharFromString,
    CharText,
    StringGraphemes,
    StringScalars,
    StringByteCount,
    StringCompareOrdinal,
    StringCompareOrdinalIgnoreCase,
    StringHashOrdinalIgnoreCase,
    StringContainsOrdinal,
    StringStartsWithOrdinal,
    StringEndsWithOrdinal,
    StringSliceUtf8,
    FileResource(crate::file_streams::Operation),
    StringSnapshot,
    ReadAllText,
    WriteAllText,
    ConsoleReadByte,
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
                if !(name.starts_with("System.Introspection.") || name == "System.Option") {
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
            "neoCLR.Runtime.ExecutingAssembly" | "neoCLR.Runtime.ReflectionArrayCreate"
        );
    if relevant {
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
            || function.no_result
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
    if function.no_result
        && !matches!(
            function.name.as_str(),
            "neoCLR.Runtime.ReflectionPropertySet"
                | "neoCLR.Runtime.GCCollect"
                | "neoCLR.Runtime.GCKeepAlive"
        )
    {
        return Err(Fault::new(
            "native service does not support no-result execution",
        ));
    }
    if let Some((operation, arity)) = crate::math::Operation::binding(&function.name) {
        if function.parameters != vec![Type::Double; arity] || function.returns != Type::Double {
            return Err(Fault::new("math binding signature mismatch"));
        }
        return Ok(Binding::Math(operation));
    }
    if let Some((query, count, result)) = crate::assembly_info::Query::binding(&function.name) {
        if function.parameters != vec![Type::String; count]
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
        ("neoCLR.Runtime.Fault", [Type::String]) => (Binding::Fault, Type::Void),
        ("neoCLR.Runtime.WriteLine", [Type::String]) => (Binding::WriteLine, Type::Void),
        ("neoCLR.Runtime.CharCategory", [Type::UInt32]) => (Binding::CharCategory, Type::Int32),
        ("neoCLR.Runtime.Utf8Encode", [Type::String]) => {
            (Binding::Utf8Encode, Type::Array(Box::new(Type::Byte)))
        }
        ("neoCLR.Runtime.Utf8Decode", [Type::ArrayRef(element)]) if **element == Type::Byte => {
            (Binding::Utf8Decode, Type::Value)
        }
        ("neoCLR.Runtime.StringFromChars", [Type::ArrayRef(element)])
            if **element == Type::Char =>
        {
            (Binding::StringFromChars, Type::String)
        }
        ("neoCLR.Runtime.StringGraphemeAt", [Type::String, Type::Int32]) => {
            (Binding::StringGraphemeAt, Type::Char)
        }
        ("neoCLR.Runtime.StringIntern", [Type::String]) => (Binding::StringIntern, Type::String),
        ("neoCLR.Runtime.StringConcat", [Type::String, Type::String]) => {
            (Binding::StringConcat, Type::String)
        }
        ("neoCLR.Runtime.StringCompareOrdinal", [Type::String, Type::String]) => {
            (Binding::StringCompareOrdinal, Type::Int32)
        }
        ("neoCLR.Runtime.StringCompareOrdinalIgnoreCase", [Type::String, Type::String]) => {
            (Binding::StringCompareOrdinalIgnoreCase, Type::Int32)
        }
        ("neoCLR.Runtime.StringHashOrdinalIgnoreCase", [Type::String]) => {
            (Binding::StringHashOrdinalIgnoreCase, Type::Int32)
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
        ("neoCLR.Runtime.ConsoleFlush", [Type::Boolean]) => (Binding::ConsoleFlush, Type::Value),
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
        ("neoCLR.Runtime.ScheduleTask", [callback])
            if *callback == crate::assembler::parse_type("fn<Void>")?
                || *callback == crate::assembler::parse_type("fn<noresult Void>")? =>
        {
            (Binding::ScheduleTask, Type::Void)
        }
        ("neoCLR.Runtime.DrainEntryTasks", []) => (Binding::DrainEntryTasks, Type::Void),
        ("neoCLR.Runtime.DefaultTaskQueue", []) => (
            Binding::DefaultTaskQueue,
            Type::from_name("System.Tasks.TaskQueue"),
        ),
        ("neoCLR.Runtime.RegisterDefaultTaskQueue", [queue])
            if queue == &Type::from_name("System.Tasks.TaskQueue") =>
        {
            (Binding::RegisterDefaultTaskQueue, Type::Void)
        }
        ("neoCLR.Runtime.CurrentTaskQueue", []) => (
            Binding::CurrentTaskQueue,
            Type::from_name("System.Tasks.TaskQueue"),
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
    if function.returns != returns {
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
        let console = options.console.as_deref();
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
            (Self::NativeIntegerTo64, [Value::IntPtr(number)]) => Ok(Value::Int64(*number as i64)),
            (Self::NativeIntegerTo64, [Value::UIntPtr(number)]) => {
                Ok(Value::UInt64(*number as u64))
            }
            (Self::Int32ToString, [Value::Int32(number)]) => {
                Ok(Value::String(number.to_string().into()))
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
            (Self::StringCompareOrdinalIgnoreCase, [Value::String(left), Value::String(right)]) => {
                Ok(Value::Int32(
                    match crate::string_comparison::compare_ignore_case(left, right) {
                        std::cmp::Ordering::Less => -1,
                        std::cmp::Ordering::Equal => 0,
                        std::cmp::Ordering::Greater => 1,
                    },
                ))
            }
            (Self::StringHashOrdinalIgnoreCase, [Value::String(value)]) => Ok(Value::Int32(
                crate::string_comparison::hash_ignore_case(value),
            )),
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
    fn typed_queue_services_require_exact_generic_signatures() {
        for (signature, accepted) in [
            ("GetDefaultTaskQueue<T>() -> T", true),
            ("GetCurrentTaskQueue<T>() -> T", true),
            ("RegisterTaskQueue<T>(T queue) -> Void", true),
            ("GetDefaultTaskQueue<T>() -> Int32", false),
            ("GetDefaultTaskQueue<T, U>() -> T", false),
            ("GetDefaultTaskQueue() -> Int32", false),
            ("RegisterTaskQueue<T>(Int32 queue) -> Void", false),
            ("RegisterTaskQueue<T>(T queue) -> noresult", false),
        ] {
            let module = crate::assemble(&format!(
                ".module Test\n.function neoCLR.Runtime.{signature}\n.methodimpl InternalCall\n.end\n"
            ));
            assert_eq!(module.is_ok(), accepted, "{signature}");
        }
    }

    #[test]
    fn scheduling_accepts_only_parameterless_unit_or_no_result_callbacks() {
        for (callback, accepted) in [
            ("fn<Void>", true),
            ("fn<noresult Void>", true),
            ("fn<Int32>", false),
            ("fn<Int32,Void>", false),
        ] {
            let module = crate::assemble(&format!(
                ".module Test\n.function neoCLR.Runtime.ScheduleTask({callback} callback) -> Void\n.methodimpl InternalCall\n.end\n"
            ));
            assert_eq!(module.is_ok(), accepted, "{callback}");
        }
    }
}

#[cfg(test)]
mod reflection_signature_tests {
    #[test]
    fn no_result_service_execution_requires_an_admitted_control_signature() {
        for (signature, accepted) in [
            (
                "ReflectionPropertySet(RuntimeTypeHandle owner, Int32 token, System.Object receiver, System.Object value) -> noresult",
                true,
            ),
            (
                "ReflectionPropertySetCheck(RuntimeTypeHandle owner, Int32 token, System.Object receiver, System.Object value) -> noresult",
                false,
            ),
            (
                "ReflectionPropertyGet(RuntimeTypeHandle owner, Int32 token, System.Object receiver) -> noresult",
                false,
            ),
            ("WriteLine(String text) -> noresult", false),
            ("GCCollect() -> noresult", true),
            ("GCKeepAlive(System.Object value) -> noresult", true),
        ] {
            let result = crate::assemble(&format!(
                ".module Test\n.type class System.Object\n.end\n.function neoCLR.Runtime.{signature}\n.methodimpl InternalCall\n.end\n"
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
