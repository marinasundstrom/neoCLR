//! Logical runtime-service uses; no target ABI or ownership policy is implied.
use crate::{
    metadata::{Function, Instruction as Op},
    Fault,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuntimeService {
    NativeAllocation,
    FrameAllocation,
    PointerMemory,
    ManagedHeap,
    ParseInt32,
    ParseInt64,
    ParseNumber,
    FormatInt32,
    ConsoleOutput,
    NativeInterop,
    StringOperations,
    CharacterClassification,
    MathOperations,
    LocalClock,
    WallClock,
    /// Conversions through the pinned IANA database, without reading the clock.
    TimeZoneRules,
    ProcessEnvironment,
    PathOperations,
    FileInput,
    FileOutput,
    ConsoleInput,
    ValueStorage,
    TypeInspection,
    /// Dynamic construction and property calls; static reachability cannot enumerate targets.
    ReflectionExecution,
    InterfaceDispatch,
    SlotReferences,
    ManagedArrays,
    TaskDispatch,
    IsolatedWorkers,
    /// Native socket ownership and I/O; submissions also require TaskDispatch.
    SocketIo,
    /// Host name resolution; submissions also require TaskDispatch.
    NameResolution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceUse {
    pub service: RuntimeService,
    /// None identifies an import declaration; Some identifies an IL instruction.
    pub instruction: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingService {
    /// Index in the associated Reachability report, not a metadata token.
    pub function: usize,
    pub instruction: Option<usize>,
    pub service: RuntimeService,
}

pub(crate) fn uses(module: &crate::Module, function: &Function) -> Result<Vec<ServiceUse>, Fault> {
    if function.pinvoke.is_some() {
        return Ok(vec![ServiceUse {
            service: RuntimeService::NativeInterop,
            instruction: None,
        }]);
    }
    if function.is_internal_call() {
        let binding = crate::native::bind_in(module, function)?;
        let service = match &binding {
            crate::native::Binding::Socket(_) => RuntimeService::SocketIo,
            crate::native::Binding::Resolve(_) => RuntimeService::NameResolution,
            #[cfg(test)]
            crate::native::Binding::TestSocketReceive => RuntimeService::TaskDispatch,
            crate::native::Binding::StartWorker(_)
            | crate::native::Binding::JoinWorker
            | crate::native::Binding::JoinWorkerResult
            | crate::native::Binding::RequestWorkerCancellation
            | crate::native::Binding::NotifyWorker => RuntimeService::IsolatedWorkers,
            crate::native::Binding::GenericCurrentTaskQueue
            | crate::native::Binding::GenericDefaultTaskQueue
            | crate::native::Binding::GenericRegisterTaskQueue
            | crate::native::Binding::CurrentTaskQueue
            | crate::native::Binding::DefaultTaskQueue
            | crate::native::Binding::ScheduleTask
            | crate::native::Binding::DrainEntryTasks
            | crate::native::Binding::RegisterDefaultTaskQueue => RuntimeService::TaskDispatch,
            crate::native::Binding::GcCollect
            | crate::native::Binding::GcInfo(_)
            | crate::native::Binding::GcKeepAlive
            | crate::native::Binding::ObjectEquals
            | crate::native::Binding::ObjectReferenceEquals
            | crate::native::Binding::ObjectIdentityHash => RuntimeService::ManagedHeap,
            crate::native::Binding::NativeAllocate | crate::native::Binding::NativeFree => {
                return Ok([
                    RuntimeService::NativeAllocation,
                    RuntimeService::PointerMemory,
                ]
                .into_iter()
                .map(|service| ServiceUse {
                    service,
                    instruction: None,
                })
                .collect());
            }
            crate::native::Binding::NativeMultiplyChecked | crate::native::Binding::Fault => {
                return Ok(vec![]);
            }
            crate::native::Binding::EnvironmentArguments
            | crate::native::Binding::EnvironmentCurrentDirectory
            | crate::native::Binding::EnvironmentVariable
            | crate::native::Binding::SystemTimeZoneName
            | crate::native::Binding::SystemCultureName => RuntimeService::ProcessEnvironment,
            crate::native::Binding::PathCombine | crate::native::Binding::PathGetFileName => {
                RuntimeService::PathOperations
            }
            crate::native::Binding::TimeZoneExists
            | crate::native::Binding::TimeZoneOffset
            | crate::native::Binding::TimeZoneMapLocal
            | crate::native::Binding::TimeZoneDatabaseVersion => RuntimeService::TimeZoneRules,
            crate::native::Binding::UnixTimeToLocal => RuntimeService::LocalClock,
            crate::native::Binding::UnixTimeTicks => RuntimeService::WallClock,
            crate::native::Binding::Math(_) => RuntimeService::MathOperations,
            crate::native::Binding::ReflectionArray(_) | crate::native::Binding::StringSnapshot => {
                RuntimeService::ManagedArrays
            }
            crate::native::Binding::ReflectionConstruct
            | crate::native::Binding::ReflectionMember(_)
            | crate::native::Binding::ReflectionProperty(_) => RuntimeService::ReflectionExecution,
            crate::native::Binding::ReflectionConstructionCheck
            | crate::native::Binding::ReflectionMemberCheck(_)
            | crate::native::Binding::ReflectionAssignable
            | crate::native::Binding::ReflectionPropertyCheck(_)
            | crate::native::Binding::Reflection(_)
            | crate::native::Binding::AssemblyInfo(_)
            | crate::native::Binding::ExecutingAssembly
            | crate::native::Binding::ObjectTypeHandle
            | crate::native::Binding::TypeName
            | crate::native::Binding::TypeEquals
            | crate::native::Binding::TypeArgumentCount
            | crate::native::Binding::TypeArgument => RuntimeService::TypeInspection,
            crate::native::Binding::ConsoleReadByte => RuntimeService::ConsoleInput,
            // Closing an opaque handle may release an input or output resource.
            // Report both logical uses; this is analysis, not an access-control policy.
            crate::native::Binding::FileResource(crate::file_streams::Operation::Close) => {
                return Ok(vec![
                    ServiceUse {
                        service: RuntimeService::FileInput,
                        instruction: None,
                    },
                    ServiceUse {
                        service: RuntimeService::FileOutput,
                        instruction: None,
                    },
                ]);
            }
            crate::native::Binding::FileResource(operation) => match operation {
                crate::file_streams::Operation::OpenRead
                | crate::file_streams::Operation::Read
                | crate::file_streams::Operation::ReadInto
                | crate::file_streams::Operation::Seek
                | crate::file_streams::Operation::Position
                | crate::file_streams::Operation::List
                | crate::file_streams::Operation::Kind => RuntimeService::FileInput,
                _ => RuntimeService::FileOutput,
            },
            crate::native::Binding::WriteAllText => RuntimeService::FileOutput,
            crate::native::Binding::ReadAllText => RuntimeService::FileInput,
            crate::native::Binding::ParseInt32 => RuntimeService::ParseInt32,
            crate::native::Binding::ParseInt64 => RuntimeService::ParseInt64,
            crate::native::Binding::ParseNumber(_) => RuntimeService::ParseNumber,
            crate::native::Binding::NativeIntegerTo64 => return Ok(vec![]),
            crate::native::Binding::Int32ToString => RuntimeService::FormatInt32,
            crate::native::Binding::IntegerToString | crate::native::Binding::StringCasing(_) => {
                RuntimeService::StringOperations
            }
            crate::native::Binding::WriteLine
            | crate::native::Binding::ConsoleWriteBytes
            | crate::native::Binding::ConsoleFlush => RuntimeService::ConsoleOutput,
            crate::native::Binding::CharCategory => RuntimeService::CharacterClassification,
            crate::native::Binding::Utf8Encode
            | crate::native::Binding::Utf8Decode
            | crate::native::Binding::StringJoinParts
            | crate::native::Binding::StringFromChars
            | crate::native::Binding::StringGraphemeAt
            | crate::native::Binding::StringIntern
            | crate::native::Binding::StringConcat
            | crate::native::Binding::StringCompareOrdinal
            | crate::native::Binding::StringCompareOrdinalIgnoreCase
            | crate::native::Binding::StringHashOrdinalIgnoreCase
            | crate::native::Binding::StringContainsOrdinal
            | crate::native::Binding::StringStartsWithOrdinal
            | crate::native::Binding::StringEndsWithOrdinal
            | crate::native::Binding::StringGraphemeCount
            | crate::native::Binding::CharFromString
            | crate::native::Binding::CharText
            | crate::native::Binding::StringGraphemes
            | crate::native::Binding::StringScalars
            | crate::native::Binding::StringByteCount
            | crate::native::Binding::StringSliceUtf8 => RuntimeService::StringOperations,
        };
        let mut uses = vec![ServiceUse {
            service,
            instruction: None,
        }];
        if matches!(
            &binding,
            crate::native::Binding::ReflectionConstruct
                | crate::native::Binding::ReflectionMember(_)
                | crate::native::Binding::ReflectionProperty(_)
        ) {
            uses.extend(
                [
                    RuntimeService::TypeInspection,
                    RuntimeService::FrameAllocation,
                    RuntimeService::ManagedHeap,
                ]
                .map(|service| ServiceUse {
                    service,
                    instruction: None,
                }),
            );
        }
        if matches!(
            &binding,
            crate::native::Binding::NotifyWorker
                | crate::native::Binding::Resolve(
                    crate::name_resolution::Operation::Lookup
                        | crate::name_resolution::Operation::LookupUntil
                )
                | crate::native::Binding::Socket(
                    crate::socket_io::Operation::ConnectAddressesUntil
                        | crate::socket_io::Operation::ReceiveUntil
                        | crate::socket_io::Operation::SendUntil
                        | crate::socket_io::Operation::Accept
                        | crate::socket_io::Operation::Connect
                        | crate::socket_io::Operation::ConnectAddresses
                        | crate::socket_io::Operation::Receive
                        | crate::socket_io::Operation::Send
                )
        ) {
            uses.push(ServiceUse {
                service: RuntimeService::TaskDispatch,
                instruction: None,
            });
        }
        if matches!(
            &binding,
            crate::native::Binding::UnixTimeToLocal
                | crate::native::Binding::ReflectionMember(_)
                | crate::native::Binding::ReflectionMemberCheck(_)
                | crate::native::Binding::TimeZoneMapLocal
                | crate::native::Binding::EnvironmentArguments
                | crate::native::Binding::Utf8Encode
                | crate::native::Binding::Utf8Decode
                | crate::native::Binding::StringJoinParts
                | crate::native::Binding::StringFromChars
        ) {
            uses.push(ServiceUse {
                service: RuntimeService::ManagedArrays,
                instruction: None,
            });
        }
        if matches!(
            &binding,
            crate::native::Binding::AssemblyInfo(
                crate::assembly_info::Query::References
                    | crate::assembly_info::Query::Modules
                    | crate::assembly_info::Query::Types
                    | crate::assembly_info::Query::ModuleTypes
            )
        ) {
            uses.push(ServiceUse {
                service: RuntimeService::ManagedArrays,
                instruction: None,
            });
        }
        if let crate::native::Binding::Reflection(query) = &binding {
            use crate::reflection::Query;
            if !matches!(
                query,
                Query::Shape
                    | Query::DisplayName
                    | Query::ElementType
                    | Query::MetadataToken
                    | Query::Module
                    | Query::DeclaringType
            ) {
                uses.push(ServiceUse {
                    service: RuntimeService::ManagedArrays,
                    instruction: None,
                });
            }
            if matches!(query, Query::Properties | Query::ElementType) {
                uses.push(ServiceUse {
                    service: RuntimeService::ValueStorage,
                    instruction: None,
                });
            }
        }
        if function.returns == crate::metadata::Type::Value {
            uses.push(ServiceUse {
                service: RuntimeService::ValueStorage,
                instruction: None,
            });
        }
        return Ok(uses);
    }
    Ok(function
        .body
        .iter()
        .enumerate()
        .flat_map(|(instruction, op)| {
            instruction_services(op)
                .iter()
                .map(move |service| ServiceUse {
                    service: *service,
                    instruction: Some(instruction),
                })
        })
        .collect())
}

fn instruction_services(op: &Op) -> &'static [RuntimeService] {
    use RuntimeService::*;
    // Exhaustive so additions to IL require an explicit service classification.
    match op {
        Op::BindFunction { .. } => &[ValueStorage, SlotReferences],
        Op::ReserveArray(_) | Op::NewArray(_) | Op::NewValueArray(_) | Op::AllocateArray(_) => {
            &[ManagedArrays, ManagedHeap, SlotReferences]
        }
        Op::CreateArray(_) => &[ManagedArrays],
        Op::ArrayLength | Op::ArrayElement(_) | Op::StoreArrayElement(_) | Op::ArrayAddress(_) => {
            &[ManagedArrays, SlotReferences]
        }
        Op::Allocate(..) | Op::Free => &[NativeAllocation, PointerMemory],
        Op::AllocateLocal => &[FrameAllocation, PointerMemory],
        Op::PackValue(..) | Op::IsValue(..) | Op::UnpackValue(..) => &[ValueStorage],
        Op::LoadTypeToken(..) => &[TypeInspection],
        Op::ReferenceType => &[TypeInspection, SlotReferences],
        Op::BoxValue(_) | Op::UnboxAny(_) => &[ManagedHeap, ValueStorage, SlotReferences],
        Op::IsInstance(_) | Op::ReferenceIsNull | Op::CastClass(_) => {
            &[TypeInspection, SlotReferences]
        }
        Op::ReferenceEqual => &[SlotReferences],
        Op::LocalAddress(..) | Op::ArgumentAddress(..) | Op::Receiver { .. } => &[SlotReferences],
        Op::LoadObject(..)
        | Op::StoreObject(..)
        | Op::FieldAddress(..)
        | Op::InitializeObject(..) => &[PointerMemory, SlotReferences],
        Op::BorrowInterface(..) | Op::CallVirtual(..) | Op::CallSelf { .. } => {
            &[InterfaceDispatch, SlotReferences]
        }
        Op::PointerFromInt(..)
        | Op::PointerAdd
        | Op::CopyObject(..)
        | Op::CopyBlock
        | Op::InitializeBlock
        | Op::LoadIndirectInt8
        | Op::LoadIndirectUInt8
        | Op::LoadIndirectInt16
        | Op::LoadIndirectUInt16
        | Op::LoadIndirectUInt32
        | Op::LoadIndirectInt64
        | Op::LoadIndirectNative
        | Op::StoreIndirectInt8
        | Op::StoreIndirectInt16
        | Op::StoreIndirectInt64
        | Op::StoreIndirectNative
        | Op::LoadIndirectFloat32
        | Op::LoadIndirectFloat64
        | Op::StoreIndirectFloat32
        | Op::StoreIndirectFloat64
        | Op::LoadIndirectInt32
        | Op::StoreIndirectInt32 => &[PointerMemory],
        Op::HeapNew => &[ManagedHeap, SlotReferences],
        Op::Unaligned(..)
        | Op::Int(..)
        | Op::Int64(..)
        | Op::CheckedInt8
        | Op::CheckedUInt8
        | Op::CheckedInt16
        | Op::CheckedUInt16
        | Op::CheckedInt32
        | Op::CheckedUInt32
        | Op::CheckedInt64
        | Op::CheckedUInt64
        | Op::CheckedNativeInt
        | Op::CheckedNativeUInt
        | Op::CheckedInt8Unsigned
        | Op::CheckedUInt8Unsigned
        | Op::CheckedInt16Unsigned
        | Op::CheckedUInt16Unsigned
        | Op::CheckedInt32Unsigned
        | Op::CheckedUInt32Unsigned
        | Op::CheckedInt64Unsigned
        | Op::CheckedUInt64Unsigned
        | Op::CheckedNativeIntUnsigned
        | Op::CheckedNativeUIntUnsigned
        | Op::ConvertInt8
        | Op::ConvertUInt8
        | Op::ConvertInt16
        | Op::ConvertUInt16
        | Op::ConvertUInt32
        | Op::ConvertInt64
        | Op::ConvertUInt64
        | Op::Float32 { .. }
        | Op::Float64 { .. }
        | Op::ConvertFloat32
        | Op::ConvertFloat64
        | Op::ConvertFloatUnsigned
        | Op::CheckFinite
        | Op::Bool(..)
        | Op::String(..)
        | Op::Void
        | Op::Arg(..)
        | Op::StoreArg(..)
        | Op::Load(..)
        | Op::ResetLocal(..)
        | Op::Store(..)
        | Op::Dup
        | Op::Pop
        | Op::BitAnd
        | Op::BitOr
        | Op::BitXor
        | Op::BitNot
        | Op::Negate
        | Op::ShiftLeft
        | Op::ShiftRight
        | Op::ShiftRightUnsigned
        | Op::Remainder
        | Op::RemainderUnsigned
        | Op::Add
        | Op::Sub
        | Op::Mul
        | Op::AddChecked
        | Op::SubChecked
        | Op::MulChecked
        | Op::Divide
        | Op::AddCheckedUnsigned
        | Op::SubCheckedUnsigned
        | Op::MulCheckedUnsigned
        | Op::DivideUnsigned
        | Op::LessUnsigned
        | Op::ConvertNativeInt
        | Op::ConvertNativeUInt
        | Op::ConvertInt32
        | Op::Equal
        | Op::Greater
        | Op::GreaterUnsigned
        | Op::Less
        | Op::Branch(..)
        | Op::BranchTrue(..)
        | Op::BranchFalse(..)
        | Op::BranchEqual(..)
        | Op::BranchNotEqual(..)
        | Op::BranchGreater(..)
        | Op::BranchGreaterUnsigned(..)
        | Op::BranchLess(..)
        | Op::BranchLessUnsigned(..)
        | Op::BranchGreaterEqual(..)
        | Op::BranchGreaterEqualUnsigned(..)
        | Op::BranchLessEqual(..)
        | Op::BranchLessEqualUnsigned(..)
        | Op::Switch(..)
        | Op::Call(..)
        | Op::Construct(..)
        | Op::Return
        | Op::New(..)
        | Op::Field(..)
        | Op::SetField(..)
        | Op::SizeOf(..)
        | Op::AlignOf(..)
        | Op::NullPointer(..)
        | Op::PointerCast(..)
        | Op::Fault(..) => &[],
    }
}
