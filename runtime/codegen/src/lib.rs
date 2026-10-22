//! Internal shared JIT/AOT backend and provisional synchronous hosting driver.
//! Interpreter-only builds do not depend on this crate.
mod bindings;
mod boxing;
mod checked_conversion;
mod class_dispatch;
mod compiler;
mod enum_queries;
mod fault_details;
mod function_invocation;
mod inspection;
mod jit;
mod limits;
mod linking;
mod load_context;
mod memory_views;
mod native_exports;
mod native_imports;
mod reflection_arrays;
mod reflection_collections;
mod reflection_construction;
mod reflection_constructors;
mod reflection_fields;
mod reflection_metadata;
mod reflection_properties;
mod reflection_snapshots;
mod runtime_context;
mod scalar_abi;
mod scalar_unions;
mod selection;
mod specialization;
mod string_projection;
mod type_initialization;

use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

/// Provisional backend hosting entry point shared with the command-line tool.
///
/// Arguments use the existing backend CLI contract, excluding the executable name.
/// Compilation and execution keep their existing stdout/stderr and file behavior.
/// A completed guest status is returned after native resources have been released;
/// the driver does not call `process::exit`. Native host failures may still abort
/// the process. A guest fault remains terminal for that invocation. This is not a stable embedding API or an interpreter fallback.
/// Calls are serialized because native helper state is not yet a concurrent-session
/// contract. Hosts must not reenter the driver from guest/native callbacks.
pub fn run(arguments: Vec<std::ffi::OsString>) -> Result<i32, Box<dyn std::error::Error>> {
    static HOST: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _session = HOST
        .lock()
        .map_err(|_| "backend hosting session poisoned")?;
    let mut args = arguments;
    let native_import_count = args.iter().filter(|a| *a == "--native-imports").count();
    if native_import_count > 1 {
        return Err("duplicate --native-imports option".into());
    }
    let native_imports = native_import_count == 1;
    args.retain(|a| a != "--native-imports");
    if native_imports
        && args.iter().any(|a| {
            a == "--jit-managed" || a == "--jit-run" || a == "--inspect" || a == "--closed-world"
        })
    {
        return Err("--native-imports requires scalar AOT emission or --jit-scalar, not managed JIT, inspection or closed-world profiles".into());
    }
    if args.first().is_some_and(|arg| arg == "--native-library") {
        return native_exports::run(args, native_imports);
    }
    if args.first().is_some_and(|arg| arg == "--jit-scalar") {
        let dependency_args = load_context::split(&mut args);
        if !dependency_args.is_empty() && !native_imports {
            return Err("scalar JIT load contexts require --native-imports".into());
        }
        let paths = load_context::parse(&dependency_args)?;
        if args.len() != 4 && !(args.len() == 6 && args[4] == "--repetitions") {
            return Err("usage: neoclr-codegen --jit-scalar <input.neoil|input.neox|input.dll> <root-name|@entry> <Int32 argument> [--repetitions 1..1000] [--native-imports] [--system <seed> --module <dependency> --object-root <dependency>]".into());
        }
        let repetitions = if args.len() == 6 {
            args[5]
                .to_str()
                .ok_or("invalid repetition count")?
                .parse::<usize>()?
        } else {
            1
        };
        if !(1..=1000).contains(&repetitions) {
            return Err("repetitions must be 1..1000".into());
        }
        let started = std::time::Instant::now();
        let input = read_module(Path::new(&args[1]))?;
        let dependencies = paths
            .modules
            .iter()
            .map(|path| read_module(Path::new(path)))
            .collect::<Result<Vec<_>, _>>()?;
        let object_root = load_context::object_root(&paths.modules, paths.object, &dependencies)?;
        let system = paths
            .system
            .map(|path| read_module(Path::new(path)))
            .transpose()?;
        let verification = if dependency_args.is_empty() {
            None
        } else {
            Some(load_context::verification(
                &input,
                system
                    .as_ref()
                    .ok_or("native-import load context requires explicit --system")?,
                &dependencies,
                object_root.as_ref(),
            )?)
        };
        let load_seconds = started.elapsed().as_secs_f64();
        let root = args[2].to_str().ok_or("root must be UTF-8")?;
        let root = if root == "@entry" { &input.entry } else { root };
        let argument: i32 = args[3].to_str().ok_or("argument must be Int32")?.parse()?;
        let mut result = jit::execute(
            &input,
            root,
            argument,
            repetitions,
            native_imports,
            verification.as_ref(),
        )?;
        result["loadSeconds"] = serde_json::json!(load_seconds);
        println!("{result}");
        return Ok(0);
    }
    let mut jit_arguments = vec![std::ffi::CString::new("neoclr-jit")?];
    while let Some(index) = args.iter().position(|a| a == "--jit-entry-argument") {
        let value = args
            .get(index + 1)
            .and_then(|v| v.to_str())
            .ok_or("--jit-entry-argument requires UTF-8 text")?;
        jit_arguments.push(std::ffi::CString::new(value)?);
        args.drain(index..=index + 1);
    }
    let jit_run = args.iter().any(|a| a == "--jit-run");
    let managed_jit = args
        .iter()
        .filter(|a| *a == "--jit-managed" || *a == "--jit-run")
        .count();
    if managed_jit > 1 {
        return Err("duplicate --jit-managed".into());
    }
    args.retain(|a| a != "--jit-managed" && *a != "--jit-run");
    if managed_jit == 0 && jit_arguments.len() != 1 {
        return Err("entry arguments require --jit-managed".into());
    }
    let mut target = if managed_jit == 1 {
        jit::host_target()?
    } else {
        compiler::Target::MacosArm64
    };
    if args.iter().filter(|a| *a == "--target").count() > 1 {
        return Err("duplicate --target option".into());
    }
    if let Some(index) = args.iter().position(|a| a == "--target") {
        let value = args
            .get(index + 1)
            .and_then(|a| a.to_str())
            .ok_or("--target requires a target triple")?;
        target = compiler::Target::parse(value)?;
        args.drain(index..=index + 1);
    }
    if managed_jit == 1 && target != jit::host_target()? {
        return Err("JIT target must match the executing host".into());
    }
    let windows_stack_count = args
        .iter()
        .filter(|a| *a == "--windows-stack-experiment")
        .count();
    if windows_stack_count > 1 {
        return Err("duplicate --windows-stack-experiment option".into());
    }
    let windows_stack = windows_stack_count == 1;
    args.retain(|a| a != "--windows-stack-experiment");
    let windows_heap_count = args
        .iter()
        .filter(|a| *a == "--windows-heap-experiment")
        .count();
    if windows_heap_count > 1 {
        return Err("duplicate --windows-heap-experiment option".into());
    }
    let windows_heap = windows_heap_count == 1;
    args.retain(|a| a != "--windows-heap-experiment");
    if windows_heap && windows_stack {
        return Err("select one Windows experiment".into());
    }
    let windows_console_count = args
        .iter()
        .filter(|a| *a == "--windows-console-experiment")
        .count();
    if windows_console_count > 1 {
        return Err("duplicate --windows-console-experiment option".into());
    }
    let windows_console = windows_console_count == 1;
    args.retain(|a| a != "--windows-console-experiment");
    if windows_console && (windows_stack || windows_heap) {
        return Err("select one Windows experiment".into());
    }
    let windows_http_count = args
        .iter()
        .filter(|a| *a == "--windows-http-experiment")
        .count();
    if windows_http_count > 1 {
        return Err("duplicate --windows-http-experiment option".into());
    }
    let windows_http = windows_http_count == 1;
    args.retain(|a| a != "--windows-http-experiment");
    if windows_http && (windows_console || windows_stack || windows_heap) {
        return Err("select one Windows experiment".into());
    }
    let probe_count = args.iter().filter(|a| *a == "--probe-stack-roots").count();
    if probe_count > 1 {
        return Err("duplicate --probe-stack-roots option".into());
    }
    let gc_count = args.iter().filter(|a| *a == "--native-gc").count();
    if gc_count > 1 {
        return Err("duplicate --native-gc option".into());
    }
    let native_gc = gc_count == 1;
    if args.iter().filter(|a| *a == "--gc").count() > 1 {
        return Err("duplicate --gc option".into());
    }
    if let Some(index) = args.iter().position(|a| a == "--gc") {
        let value = args
            .get(index + 1)
            .and_then(|a| a.to_str())
            .filter(|value| !value.starts_with("--"))
            .ok_or("Missing collector for --gc; available: mark-sweep")?;
        let collector = value.parse::<neoclr::GarbageCollector>()?;
        if !native_gc {
            return Err("--gc requires --native-gc".into());
        }
        match collector {
            neoclr::GarbageCollector::MarkSweep => {}
            _ => return Err("Collector is not supported by this native backend".into()),
        }
        args.drain(index..=index + 1);
    }

    let bootstrap_count = args
        .iter()
        .filter(|a| *a == "--native-host-bootstrap")
        .count();
    if bootstrap_count > 1 {
        return Err("duplicate --native-host-bootstrap option".into());
    }
    let native_host_bootstrap = bootstrap_count == 1;
    if native_host_bootstrap && !native_gc {
        return Err("--native-host-bootstrap requires --native-gc".into());
    }
    args.retain(|a| a != "--native-host-bootstrap");
    let stack_count = args
        .iter()
        .filter(|a| *a == "--native-stack-budget")
        .count();
    if stack_count > 1 {
        return Err("duplicate --native-stack-budget option".into());
    }
    let native_stack_budget = stack_count == 1;
    if native_stack_budget && !native_gc {
        return Err("--native-stack-budget requires --native-gc".into());
    }
    args.retain(|a| a != "--native-stack-budget");
    if native_gc && probe_count != 0 {
        return Err("--native-gc and --probe-stack-roots are distinct modes".into());
    }
    args.retain(|a| a != "--native-gc");
    let probe_stack_roots = probe_count == 1 || native_gc;
    args.retain(|a| a != "--probe-stack-roots");
    let compile_system_count = args.iter().filter(|a| *a == "--compile-system").count();
    if compile_system_count > 1 {
        return Err("duplicate --compile-system option".into());
    }
    let compile_system = compile_system_count == 1;
    args.retain(|a| a != "--compile-system");
    let bind_user_fault_count = args.iter().filter(|a| *a == "--bind-user-fault").count();
    if bind_user_fault_count > 1 {
        return Err("duplicate --bind-user-fault option".into());
    }
    let bind_user_fault = bind_user_fault_count == 1;
    args.retain(|a| a != "--bind-user-fault");
    if bind_user_fault && !compile_system {
        return Err("--bind-user-fault requires --compile-system".into());
    }
    let input_count = args
        .iter()
        .filter(|a| *a == "--bind-console-read-byte")
        .count();
    if input_count > 1 {
        return Err("duplicate --bind-console-read-byte option".into());
    }
    let bind_console_read_byte = input_count == 1;
    args.retain(|a| a != "--bind-console-read-byte");
    if bind_console_read_byte && !compile_system {
        return Err("--bind-console-read-byte requires --compile-system".into());
    }
    let output_count = args
        .iter()
        .filter(|a| *a == "--bind-console-write-line")
        .count();
    if output_count > 1 {
        return Err("duplicate --bind-console-write-line option".into());
    }
    let bind_console_write_line = output_count == 1;
    args.retain(|a| a != "--bind-console-write-line");
    if bind_console_write_line && !compile_system {
        return Err("--bind-console-write-line requires --compile-system".into());
    }
    let crypto_count = args.iter().filter(|a| *a == "--bind-cryptography").count();
    if crypto_count > 1 {
        return Err("duplicate --bind-cryptography option".into());
    }
    let bind_cryptography = crypto_count == 1;
    args.retain(|a| a != "--bind-cryptography");
    if bind_cryptography && !compile_system {
        return Err("--bind-cryptography requires --compile-system".into());
    }
    let tls_count = args.iter().filter(|a| *a == "--bind-tls").count();
    if tls_count > 1 {
        return Err("duplicate --bind-tls option".into());
    }
    let bind_tls = tls_count == 1;
    args.retain(|a| a != "--bind-tls");
    if bind_tls && !compile_system {
        return Err("--bind-tls requires --compile-system".into());
    }
    let stream_output_count = args
        .iter()
        .filter(|a| *a == "--bind-console-stream-output")
        .count();
    if stream_output_count > 1 {
        return Err("duplicate --bind-console-stream-output option".into());
    }
    let bind_console_stream_output = stream_output_count == 1;
    args.retain(|a| a != "--bind-console-stream-output");
    if bind_console_stream_output && !compile_system {
        return Err("--bind-console-stream-output requires --compile-system".into());
    }
    let format_count = args
        .iter()
        .filter(|a| *a == "--bind-int32-to-string")
        .count();
    if format_count > 1 {
        return Err("duplicate --bind-int32-to-string option".into());
    }
    let bind_int32_to_string = format_count == 1;
    args.retain(|a| a != "--bind-int32-to-string");
    if bind_int32_to_string && !compile_system {
        return Err("--bind-int32-to-string requires --compile-system".into());
    }
    let paths_count = args.iter().filter(|a| *a == "--bind-paths").count();
    if paths_count > 1 {
        return Err("duplicate --bind-paths option".into());
    }
    let bind_paths = paths_count == 1;
    args.retain(|a| a != "--bind-paths");
    if bind_paths && !compile_system {
        return Err("--bind-paths requires --compile-system".into());
    }
    let file_output_count = args.iter().filter(|a| *a == "--bind-file-output").count();
    if file_output_count > 1 {
        return Err("duplicate --bind-file-output option".into());
    }
    let bind_file_output = file_output_count == 1;
    args.retain(|a| a != "--bind-file-output");
    if bind_file_output && !compile_system {
        return Err("--bind-file-output requires --compile-system".into());
    }
    let file_input_count = args.iter().filter(|a| *a == "--bind-file-input").count();
    if file_input_count > 1 {
        return Err("duplicate --bind-file-input option".into());
    }
    let bind_file_input = file_input_count == 1;
    args.retain(|a| a != "--bind-file-input");
    if bind_file_input && !compile_system {
        return Err("--bind-file-input requires --compile-system".into());
    }
    let utf8_count = args.iter().filter(|a| *a == "--bind-utf8-text").count();
    if utf8_count > 1 {
        return Err("duplicate --bind-utf8-text option".into());
    }
    let bind_utf8_text = utf8_count == 1;
    args.retain(|a| a != "--bind-utf8-text");
    if bind_utf8_text && !compile_system {
        return Err("--bind-utf8-text requires --compile-system".into());
    }
    let culture_count = args
        .iter()
        .filter(|a| *a == "--bind-system-culture")
        .count();
    if culture_count > 1 {
        return Err("duplicate --bind-system-culture option".into());
    }
    let bind_system_culture = culture_count == 1;
    args.retain(|a| a != "--bind-system-culture");
    if bind_system_culture && !compile_system {
        return Err("--bind-system-culture requires --compile-system".into());
    }
    let zone_count = args
        .iter()
        .filter(|a| *a == "--bind-time-zone-rules")
        .count();
    if zone_count > 1 {
        return Err("duplicate --bind-time-zone-rules option".into());
    }
    let bind_time_zone_rules = zone_count == 1;
    args.retain(|a| a != "--bind-time-zone-rules");
    if bind_time_zone_rules && !compile_system {
        return Err("--bind-time-zone-rules requires --compile-system".into());
    }
    let character_count = args
        .iter()
        .filter(|a| *a == "--bind-character-text")
        .count();
    if character_count > 1 {
        return Err("duplicate --bind-character-text option".into());
    }
    let bind_character_text = character_count == 1;
    args.retain(|a| a != "--bind-character-text");
    if bind_character_text && !compile_system {
        return Err("--bind-character-text requires --compile-system".into());
    }
    let integer_count = args.iter().filter(|a| *a == "--bind-integer-text").count();
    if integer_count > 1 {
        return Err("duplicate --bind-integer-text option".into());
    }
    let bind_integer_text = integer_count == 1;
    args.retain(|a| a != "--bind-integer-text");
    if bind_integer_text && !compile_system {
        return Err("--bind-integer-text requires --compile-system".into());
    }
    let task_count = args.iter().filter(|a| *a == "--bind-task-queue").count();
    if task_count > 1 {
        return Err("duplicate --bind-task-queue option".into());
    }
    let bind_task_queue = task_count == 1;
    args.retain(|a| a != "--bind-task-queue");
    if bind_task_queue && (!compile_system || !native_gc) {
        return Err("--bind-task-queue requires --compile-system and --native-gc".into());
    }
    let accept_count = args.iter().filter(|a| *a == "--bind-socket-accept").count();
    if accept_count > 1 {
        return Err("duplicate --bind-socket-accept option".into());
    }
    let bind_socket_accept = accept_count == 1;
    args.retain(|a| a != "--bind-socket-accept");
    if bind_socket_accept && (!compile_system || !native_gc) {
        return Err("--bind-socket-accept requires --compile-system and --native-gc".into());
    }
    let client_count = args.iter().filter(|a| *a == "--bind-socket-client").count();
    if client_count > 1 {
        return Err("duplicate --bind-socket-client option".into());
    }
    let bind_socket_client = client_count == 1;
    args.retain(|a| a != "--bind-socket-client");
    if bind_socket_client && (!compile_system || !native_gc || !bind_socket_accept) {
        return Err(
            "--bind-socket-client requires --compile-system, --native-gc and --bind-socket-accept"
                .into(),
        );
    }
    let scheduler_count = args.iter().filter(|a| *a == "--bind-scheduler").count();
    if scheduler_count > 1 {
        return Err("duplicate --bind-scheduler option".into());
    }
    let bind_scheduler = scheduler_count == 1;
    args.retain(|a| a != "--bind-scheduler");
    if bind_scheduler && (!compile_system || !native_gc) {
        return Err("--bind-scheduler requires --compile-system and --native-gc".into());
    }
    let timer_count = args.iter().filter(|a| *a == "--bind-timers").count();
    if timer_count > 1 {
        return Err("duplicate --bind-timers option".into());
    }
    let bind_timers = timer_count == 1;
    let stack_trace_count = args.iter().filter(|a| *a == "--bind-stack-trace").count();
    if stack_trace_count > 1 {
        return Err("duplicate --bind-stack-trace".into());
    }
    let bind_stack_trace = stack_trace_count == 1;
    args.retain(|a| a != "--bind-stack-trace");
    if bind_stack_trace && (!compile_system || !native_gc) {
        return Err("--bind-stack-trace requires compiled System and native GC roots".into());
    }
    let runtime_async_count = args.iter().filter(|a| *a == "--bind-runtime-async").count();
    if runtime_async_count > 1 {
        return Err("duplicate --bind-runtime-async".into());
    }
    let bind_runtime_async = runtime_async_count == 1;
    args.retain(|a| a != "--bind-runtime-async");
    if bind_runtime_async && (!compile_system || !native_gc || !native_stack_budget) {
        return Err("--bind-runtime-async requires compiled System, GC and stack budget".into());
    }

    args.retain(|a| a != "--bind-timers");
    if bind_timers && (!compile_system || !native_gc) {
        return Err("--bind-timers requires --compile-system and --native-gc".into());
    }
    let transfer_count = args
        .iter()
        .filter(|a| *a == "--bind-socket-transfer")
        .count();
    if transfer_count > 1 {
        return Err("duplicate --bind-socket-transfer option".into());
    }
    let bind_socket_transfer = transfer_count == 1;
    args.retain(|a| a != "--bind-socket-transfer");
    if bind_socket_transfer && (!compile_system || !native_gc) {
        return Err("--bind-socket-transfer requires --compile-system and --native-gc".into());
    }
    let socket_count = args
        .iter()
        .filter(|a| *a == "--bind-socket-listener")
        .count();
    if socket_count > 1 {
        return Err("duplicate --bind-socket-listener option".into());
    }
    let bind_socket_listener = socket_count == 1;
    args.retain(|a| a != "--bind-socket-listener");
    if bind_socket_listener && !compile_system {
        return Err("--bind-socket-listener requires --compile-system".into());
    }
    let reference_count = args.iter().filter(|a| *a == "--reference-arena").count();
    if reference_count > 1 {
        return Err("duplicate --reference-arena option".into());
    }
    let reference_arena = reference_count == 1;
    args.retain(|a| a != "--reference-arena");
    if bind_socket_listener && !reference_arena {
        return Err("--bind-socket-listener requires --reference-arena".into());
    }
    if native_gc && !reference_arena {
        return Err("--native-gc requires --reference-arena".into());
    }
    if windows_console
        && (target != compiler::Target::WindowsX64
            || !compile_system
            || !native_gc
            || !native_stack_budget
            || !reference_arena)
    {
        return Err("Windows console experiment requires Windows x64, --compile-system, --reference-arena, --native-gc and --native-stack-budget".into());
    }
    if windows_console
        && (bind_paths
            || bind_file_input
            || bind_file_output
            || bind_socket_listener
            || bind_socket_accept
            || bind_socket_transfer
            || bind_socket_client)
    {
        return Err(
            "Windows console experiment does not support file, path or socket services".into(),
        );
    }
    if windows_http
        && (target != compiler::Target::WindowsX64
            || !compile_system
            || !native_gc
            || !native_stack_budget
            || !reference_arena
            || !bind_task_queue
            || !bind_socket_listener
            || !bind_socket_accept
            || !bind_socket_transfer)
    {
        return Err("Windows HTTP experiment requires Windows x64, --compile-system, --reference-arena, --native-gc, --native-stack-budget and task/listener/accept/transfer bindings".into());
    }
    if windows_http && (bind_paths || bind_file_input || bind_file_output) {
        return Err("Windows HTTP experiment does not support file or path services".into());
    }
    if (windows_stack || windows_heap)
        && (target != compiler::Target::WindowsX64
            || !native_gc
            || !native_stack_budget
            || !reference_arena
            || compile_system)
    {
        return Err("Windows stack/heap experiments require Windows x64, --native-gc, --native-stack-budget and --reference-arena, without --compile-system".into());
    }
    if reference_arena && !compile_system && !windows_stack && !windows_heap {
        return Err("--reference-arena requires --compile-system".into());
    }
    if bind_console_stream_output && !reference_arena {
        return Err("--bind-console-stream-output requires --reference-arena".into());
    }
    if bind_file_input && !reference_arena {
        return Err("--bind-file-input requires --reference-arena".into());
    }
    if bind_file_output && !reference_arena {
        return Err("--bind-file-output requires --reference-arena".into());
    }
    if bind_paths && !reference_arena {
        return Err("--bind-paths requires --reference-arena".into());
    }
    if bind_time_zone_rules && !reference_arena {
        return Err("--bind-time-zone-rules requires --reference-arena".into());
    }
    if bind_system_culture && !reference_arena {
        return Err("--bind-system-culture requires --reference-arena".into());
    }
    if bind_utf8_text && !reference_arena {
        return Err("--bind-utf8-text requires --reference-arena".into());
    }
    if bind_cryptography && !reference_arena {
        return Err("--bind-cryptography requires --reference-arena".into());
    }
    if bind_tls && (!reference_arena || !native_gc) {
        return Err("--bind-tls requires --reference-arena and --native-gc".into());
    }
    let fault_details_count = args.iter().filter(|a| *a == "--fault-details").count();
    if fault_details_count > 1 {
        return Err("duplicate --fault-details option".into());
    }
    let fault_details = fault_details_count == 1
        || bind_user_fault
        || bind_console_read_byte
        || bind_console_write_line
        || bind_console_stream_output
        || bind_int32_to_string
        || bind_system_culture
        || bind_character_text
        || bind_integer_text
        || reference_arena;
    args.retain(|a| a != "--fault-details");
    if probe_stack_roots && !fault_details {
        return Err(
            "--probe-stack-roots requires --fault-details or an existing context-enabled binding"
                .into(),
        );
    }
    let mut reflection_roots = None;
    if args.iter().filter(|a| *a == "--reflection-roots").count() > 1 {
        return Err("duplicate --reflection-roots option".into());
    }
    if let Some(i) = args.iter().position(|a| a == "--reflection-roots") {
        let path = args
            .get(i + 1)
            .ok_or("--reflection-roots requires a JSON file")?;
        reflection_roots = Some(serde_json::from_slice(&fs::read(path)?)?);
        args.drain(i..=i + 1);
        if !compile_system || !reference_arena {
            return Err(
                "--reflection-roots requires --compile-system and --reference-arena".into(),
            );
        }
    }
    let dependency_args = load_context::split(&mut args);
    let load_context::Paths {
        modules: module_paths,
        system: system_path,
        object: object_path,
    } = load_context::parse(&dependency_args)?;
    if compile_system && system_path.is_none() {
        return Err("--compile-system requires explicit --system".into());
    }
    let inspect = args.first().is_some_and(|a| a == "--inspect");
    let inspect_closed = inspect && args.get(3).is_some_and(|a| a == "--closed-world");
    let closed = args.first().is_some_and(|a| a == "--closed-world");
    if managed_jit == 1 && !closed {
        return Err("--jit-managed requires --closed-world".into());
    }
    if target == compiler::Target::WindowsX64
        && (inspect || closed && !windows_console && !windows_http)
    {
        return Err("Windows x64 currently supports only scalar/literal-console emission; inspection and closed-world profiles remain macOS ARM64-only".into());
    }
    if !(args.len() == 3
        || (args.len() == 4 && (args[3] == "--console" || closed || inspect_closed)))
    {
        return Err(
            "usage: neoclr-codegen <input.neoil|input.neox|input.dll> <root-name|@entry> <output.o> [--console]; or --inspect <input> <root-name|@entry> [--closed-world]; or --closed-world <input> <root-name|@entry> <output.o>; --gc mark-sweep explicitly selects the included collector and requires --native-gc; closed-world modes accept trailing --module <library>, --system <seed>, --object-root <dependency> pairs; --reflection-roots <file.json> admits explicit source-identity construction, metadata and module-function retention (schemas 1–5) with --compile-system and --reference-arena; --compile-system opts supplied System managed bodies into closed-world selection; --bind-tls binds exact TLS ownership services with --compile-system, --reference-arena and --native-gc plus a matching TLS host adapter; --bind-cryptography binds exact byte-array hash/MAC/random services with --compile-system and --reference-arena plus a matching host adapter; --bind-user-fault binds exact supplied neoCLR.Runtime.Fault/Fail services to UserFault with details; --bind-console-read-byte binds the exact supplied input service to a linked C adapter; --bind-console-write-line binds the exact supplied output service to a linked UTF-8 adapter; --bind-console-stream-output binds raw byte Write/Flush with --reference-arena; --bind-int32-to-string binds formatting with caller-owned text arena ABI v4; --bind-paths binds exact lexical Unix path services; --bind-file-output binds bounded blocking UTF-8 file output; --bind-file-input binds bounded strict UTF-8 file input; --bind-utf8-text binds UTF-8 encoding/decoding, concatenation, ordinal predicates, byte counts and scalar-boundary slices with --reference-arena; --bind-system-culture binds a host preference snapshot with --reference-arena; --bind-time-zone-rules binds pinned IANA lookup/offset/version services with --reference-arena; --bind-character-text binds exact UTF-8 grapheme character services; --bind-integer-text binds primitive numeric/Boolean parsing, signed/unsigned 64-bit formatting and native-width conversion services; --bind-scheduler binds private deferred runnable submission with --native-gc; --bind-timers binds private one-shot scheduling with --native-gc and a polled host scope; --bind-socket-transfer binds Receive/Send/TransferResult and request-deadline services with --native-gc; --bind-socket-client binds bounded DNS/outbound connect with --native-gc and --bind-socket-accept; --bind-task-queue binds exact closed TaskQueue services with --native-gc; --bind-socket-accept binds Accept/ConnectResult/Cancel with --native-gc; --bind-socket-listener binds Listen/LocalPort/Close to an explicit host socket scope (requires --reference-arena); --reference-arena admits bounded invocation-owned reference objects in ABI v4; --fault-details exports ABI v3 with caller-owned diagnostics; --probe-stack-roots adds a read-only pre-operation spill callback (requires a context-enabled profile, not a collector); --native-host-bootstrap exports an owned fn<Void> root handle instead of an ordinary entry (requires --native-gc); --native-gc enables experimental nonmoving collection with --reference-arena and a matching statically linked GC adapter; --native-stack-budget opts into guarded recursion with --native-gc and the matching macOS ARM64 stack adapter; --native-imports opts macOS ARM64 scalar AOT or --jit-scalar into trusted free numeric/pointer C imports (AOT links runtime/native/interop/imports.c)"
                .into(),
        );
    }
    if inspect && !(args.len() == 3 || inspect_closed) {
        return Err(
            "usage: neoclr-codegen --inspect <input> <root-name|@entry> [--closed-world]".into(),
        );
    }
    if closed && args.len() != 4 {
        return Err(
            "usage: neoclr-codegen --closed-world <input> <root-name|@entry> <output.o>".into(),
        );
    }
    let shifted = inspect || closed;
    let input_path = &args[usize::from(shifted)];
    if !(dependency_args.is_empty() || closed || inspect_closed || native_imports) {
        return Err("load-context inputs require closed-world emission, inspection or scalar --native-imports".into());
    }
    let load_started = std::time::Instant::now();
    let input = read_module(Path::new(input_path))?;
    let dependencies = module_paths
        .iter()
        .map(|path| read_module(Path::new(path)))
        .collect::<Result<Vec<_>, _>>()?;
    let object_root = load_context::object_root(&module_paths, object_path, &dependencies)?;
    let context = system_path
        .map(|path| {
            read_module(Path::new(path)).map(|system| linking::RuntimeContext {
                system,
                object_root,
                reflection_roots,
                compile_system,
                bind_user_fault,
                bind_console_read_byte,
                bind_console_write_line,
                bind_console_stream_output,
                bind_cryptography,
                bind_tls,
                bind_int32_to_string,
                bind_utf8_text,
                bind_paths,
                bind_file_output,
                bind_file_input,
                bind_character_text,
                bind_system_culture,
                bind_time_zone_rules,
                bind_integer_text,
                bind_socket_listener,
                bind_socket_accept,
                bind_task_queue,
                bind_socket_transfer,
                bind_timers,
                bind_gc_collect: native_gc,
                bind_runtime_async,
                bind_stack_trace,
                bind_scheduler,
                bind_socket_client,
                reference_arena,
            })
        })
        .transpose()?;
    let load_seconds = load_started.elapsed().as_secs_f64();
    let root = args[1 + usize::from(shifted)]
        .to_str()
        .ok_or("root name must be UTF-8")?;
    let root = if root == "@entry" { &input.entry } else { root };
    if inspect {
        println!(
            "{}",
            serde_json::to_string_pretty(&inspection::report(
                &input,
                root,
                inspect_closed,
                &dependencies,
                context.as_ref(),
                fault_details,
                probe_stack_roots,
                native_gc,
                native_stack_budget,
                native_host_bootstrap
            ))?
        );
        return Ok(0);
    }
    let selection_started = std::time::Instant::now();
    let selection = if closed {
        Some(if dependencies.is_empty() {
            selection::prepare(&input, root)?
        } else {
            linking::prepare(&input, &dependencies, root, context.as_ref())?
        })
    } else {
        None
    };
    let selection_seconds = selection_started.elapsed().as_secs_f64();
    let compile_input = selection.as_ref().map_or(&input, |(module, _)| module);
    let details = fault_details
        .then(|| fault_details::Options::from_report(selection.as_ref().map(|(_, r)| r)));
    let details = details.map(|mut d| {
        d.windows_stack_experiment = windows_stack || windows_heap;
        d.windows_console_experiment = windows_console;
        d.windows_http_experiment = windows_http;
        d.windows_heap_experiment = windows_heap;
        if windows_stack || windows_heap {
            d.reference_arena = true;
        }
        d.probe_stack_roots = probe_stack_roots;
        d.native_gc = native_gc;
        d.native_host_bootstrap = native_host_bootstrap;
        d.native_stack_budget = native_stack_budget;
        d
    });
    if managed_jit == 1 {
        if !closed || native_host_bootstrap {
            return Err(
                "--jit-managed requires closed-world compilation without bootstrap bindings".into(),
            );
        }
        let details = details
            .as_ref()
            .ok_or("--jit-managed requires explicit GC/context bindings")?;
        let mut result = jit::execute_managed(
            compile_input,
            root,
            details,
            Path::new(&args[3]),
            &jit_arguments,
            target,
        )?;
        if jit_run {
            // execute_managed has released all native resources before returning.
            return Ok(result["guestExitCode"]
                .as_i64()
                .ok_or("missing JIT guest status")? as i32);
        }
        result["metadataLoadSeconds"] = serde_json::json!(load_seconds);
        result["selectionAndVerificationSeconds"] = serde_json::json!(selection_seconds);
        eprintln!("JIT_RESULT {result}");
        return Ok(0);
    }
    let import_verification = if native_imports && !dependency_args.is_empty() {
        let context = context
            .as_ref()
            .ok_or("native-import load context requires explicit --system")?;
        Some(load_context::verification(
            &input,
            &context.system,
            &dependencies,
            context.object_root.as_ref(),
        )?)
    } else {
        None
    };
    let object = compiler::compile_for_target_with_imports(
        compile_input,
        root,
        !closed && args.len() == 4,
        details.as_ref(),
        target,
        native_imports,
        import_verification.as_ref(),
    )?;
    // Do not clobber an existing artifact, including on failed compilation.
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2 + usize::from(closed)])?;
    output.write_all(&object)?;
    if let Some((_, report)) = selection {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(0);
    }
    println!(
        "Emitted {} object; C export: {}",
        target.triple(),
        if native_host_bootstrap {
            "neoclr_bootstrap_callback_v1 with an owned callback handle"
        } else if reference_arena {
            "neoclr_entry_v4 with caller-owned collector context"
        } else if fault_details {
            "neoclr_entry_v3 with caller-owned fault details"
        } else {
            "neoclr_entry_v2"
        }
    );
    Ok(0)
}

fn read_module(input_path: &Path) -> Result<neoclr::Module, Box<dyn std::error::Error>> {
    // Reuse the runtime's native CIL decoder; never translate binary metadata back
    // through text or interpret CLI projection bodies as neoCLR instructions.
    let mut bytes = Vec::new();
    fs::File::open(input_path)?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("input exceeds 16 MiB AOT profile limit".into());
    }
    let input = if bytes.starts_with(b"NEOX") {
        neoclr::metadata_container::decode_envelope(&bytes).map_err(|error| error.to_string())?
    } else if bytes.starts_with(b"MZ") {
        neoclr::metadata_container::decode(&bytes).map_err(|error| error.to_string())?
    } else if input_path
        .extension()
        .is_some_and(|extension| extension == "neoil")
    {
        neoclr::assemble(std::str::from_utf8(&bytes)?).map_err(|error| error.to_string())?
    } else {
        return Err("expected neoCLR CIL in NEOX, PE/#Neo or .neoil source".into());
    };
    Ok(input)
}
