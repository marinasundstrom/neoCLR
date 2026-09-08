use neoclr::{
    ExecutionOptions, Limits, LoadedProgram,
    debugger::{Breakpoint, DebugCommand, DebugSnapshot, DebugValue, Debugger},
};
use std::{
    io::{self, BufRead, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const HELP: &str = "continue/c  pause/p  step/s (one IL instruction)  next/n (next source line, over calls)  out\nbt (call stack)  stack [frame index]  heap [object id]  memory (tracked native bytes)\nsource  il  gc  output  watch (toggle live display)\nbreak <function> <IL index>  break <line> (current source)  clear\ninput <text>  eof  status  quit/q\nExecution starts paused. Inspection is read-only; running snapshots normally refresh at most 10 times/sec.";

fn value(out: &mut impl Write, name: &str, v: &DebugValue, indent: usize) -> io::Result<()> {
    writeln!(
        out,
        "{:indent$}{name}: {} = {}{}",
        "",
        v.ty,
        v.value,
        if v.truncated { " [truncated]" } else { "" }
    )?;
    for (name, child) in &v.children {
        value(out, name, child, indent + 2)?;
    }
    Ok(())
}
fn overview(out: &mut impl Write, snapshot: &DebugSnapshot) -> io::Result<()> {
    writeln!(
        out,
        "[{}] snapshot #{}; {} frames; GC {}",
        snapshot.status,
        snapshot.revision,
        snapshot.frames.len(),
        snapshot.gc
    )?;
    for frame in &snapshot.frames {
        write!(
            out,
            "  #{} {} @ IL {}",
            frame.index,
            frame.function.escape_debug(),
            frame.instruction
        )?;
        if let Some(point) = &frame.source {
            write!(
                out,
                "  {}:{}:{}",
                point.document.escape_debug(),
                point.line,
                point.column
            )?;
        }
        writeln!(out)?;
    }
    if let Some(fault) = &snapshot.fault {
        writeln!(out, "Fault: {}", fault.escape_debug())?;
    }
    if let Some(result) = &snapshot.result {
        value(out, "result", result, 0)?;
    }
    if snapshot.truncated {
        writeln!(out, "[snapshot limits reached; some memory omitted]")?;
    }
    Ok(())
}
fn inspect(
    out: &mut impl Write,
    snapshot: &DebugSnapshot,
    command: &str,
    argument: Option<&str>,
) -> Result<(), String> {
    let index = argument
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| "expected numeric index".to_owned())
        })
        .transpose()?;
    let result = match command {
        "stack" => {
            let frame = snapshot
                .frames
                .iter()
                .find(|f| Some(f.index) == index)
                .or_else(|| {
                    if index.is_none() {
                        snapshot.frames.first()
                    } else {
                        None
                    }
                })
                .ok_or("frame not in snapshot")?;
            writeln!(
                out,
                "Frame #{} {}",
                frame.index,
                frame.function.escape_debug()
            )
            .map_err(|e| e.to_string())?;
            for (name, v) in frame.arguments.iter().chain(&frame.locals) {
                value(out, name, v, 2).map_err(|e| e.to_string())?;
            }
            for (i, v) in frame.evaluation_stack.iter().enumerate() {
                value(out, &format!("eval[{i}]"), v, 2).map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        "heap" => {
            if index.is_some_and(|id| !snapshot.heap.iter().any(|(i, _)| *i == id)) {
                return Err("heap object not in snapshot".into());
            }
            for (id, v) in &snapshot.heap {
                if index.is_none_or(|n| n == *id) {
                    value(out, &format!("heap#{id}"), v, 0).map_err(|e| e.to_string())?;
                }
            }
            Ok(())
        }
        "memory" => {
            for a in &snapshot.native {
                writeln!(
                    out,
                    "native#{} {} bytes={} [{}{}]",
                    a.id,
                    if a.frame_owned { "frame-owned" } else { "heap" },
                    a.size,
                    a.bytes
                        .iter()
                        .map(|b| b.map_or_else(|| "??".into(), |n| format!("{n:02x}")))
                        .collect::<Vec<_>>()
                        .join(" "),
                    if a.size > a.bytes.len() { " …" } else { "" }
                )
                .map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        "gc" => writeln!(out, "{}", snapshot.gc),
        "output" => {
            for line in &snapshot.output {
                writeln!(out, "{line}").map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        "il" => {
            if let Some(f) = snapshot.frames.first() {
                writeln!(
                    out,
                    "{} @ {}: {}",
                    f.function.escape_debug(),
                    f.instruction,
                    f.operation
                )
            } else {
                Ok(())
            }
        }
        _ => overview(out, snapshot),
    };
    result.map_err(|e| e.to_string())
}

pub fn run(
    program: LoadedProgram,
    document: &str,
    source: &str,
    arguments: Vec<String>,
) -> Result<Vec<String>, String> {
    let debugger = Debugger::new();
    let worker_debugger = debugger.clone();
    let worker = std::thread::spawn(move || {
        // SAFETY: like CLI run, debug executes explicitly selected trusted guest/native code.
        let result = unsafe {
            program.run_with_native(ExecutionOptions {
                debugger: Some(worker_debugger.clone()),
                arguments,
                console: Some(Arc::new(worker_debugger.clone())),
                limits: Limits {
                    instructions: usize::MAX,
                    ..Limits::default()
                },
                ..Default::default()
            })
        };
        if let Err(fault) = result {
            worker_debugger.launch_failure(&fault);
        }
    });
    let watch = Arc::new(AtomicBool::new(false));
    let alive = Arc::new(AtomicBool::new(true));
    let watcher = {
        let debugger = debugger.clone();
        let watch = watch.clone();
        let alive = alive.clone();
        std::thread::spawn(move || {
            let mut revision = 0;
            while alive.load(Ordering::Relaxed) {
                let snapshot = debugger.wait(revision, Duration::from_millis(250));
                if watch.load(Ordering::Relaxed) && snapshot.revision != revision {
                    let mut out = io::stdout().lock();
                    let _ = overview(&mut out, &snapshot);
                    let _ = inspect(&mut out, &snapshot, "stack", None);
                    let _ = inspect(&mut out, &snapshot, "heap", None);
                }
                revision = snapshot.revision;
            }
        })
    };
    println!("neoCLR debugger\n{HELP}");
    let initial = debugger.wait(0, Duration::from_secs(5));
    overview(&mut io::stdout().lock(), &initial).map_err(|e| e.to_string())?;
    let result = (|| {
        let input = io::stdin();
        let mut lines = input.lock().lines();
        loop {
            print!("neo debug> ");
            io::stdout().flush().map_err(|e| e.to_string())?;
            let Some(line) = lines.next() else {
                break;
            };
            let line = line.map_err(|e| e.to_string())?;
            let mut words = line.split_whitespace();
            let Some(command) = words.next() else {
                continue;
            };
            if matches!(command, "q" | "quit") {
                break;
            }
            let snapshot = debugger.snapshot();
            let action = match command {
                "c" | "continue" => Some(DebugCommand::Continue),
                "p" | "pause" => Some(DebugCommand::Pause),
                "s" | "step" => Some(DebugCommand::Step),
                "n" | "next" => Some(DebugCommand::Next),
                "out" => Some(DebugCommand::Out),
                "clear" => Some(DebugCommand::ClearBreakpoints),
                "break" => {
                    let args: Vec<_> = words.clone().collect();
                    let point = match args.as_slice() {
                        [line] => line.parse::<usize>().ok().filter(|n| *n > 0).map(|line| {
                            Breakpoint::Source {
                                document: snapshot
                                    .frames
                                    .first()
                                    .and_then(|f| f.source.as_ref())
                                    .map_or_else(|| document.to_owned(), |p| p.document.clone()),
                                line,
                            }
                        }),
                        [function, pc] => {
                            pc.parse::<usize>()
                                .ok()
                                .map(|instruction| Breakpoint::Instruction {
                                    function: (*function).into(),
                                    instruction,
                                })
                        }
                        _ => None,
                    };
                    if let Some(point) = point {
                        Some(DebugCommand::Break(point))
                    } else {
                        println!("break <line> or break <function> <IL index>");
                        continue;
                    }
                }
                "input" => {
                    let text = line
                        .trim_start()
                        .strip_prefix("input")
                        .unwrap()
                        .trim_start();
                    if let Err(e) = debugger.input(format!("{text}\n").as_bytes()) {
                        println!("{e}");
                    }
                    continue;
                }
                "eof" => {
                    debugger.end_input();
                    continue;
                }
                "watch" => {
                    let enabled = !watch.fetch_xor(true, Ordering::Relaxed);
                    println!("live display {}", if enabled { "on" } else { "off" });
                    continue;
                }
                "help" | "h" => {
                    println!("{HELP}");
                    continue;
                }
                "source" => {
                    if let Some(point) = snapshot.frames.first().and_then(|f| f.source.as_ref()) {
                        println!(
                            "{}:{}:{}",
                            point.document.escape_debug(),
                            point.line,
                            point.column
                        );
                        if point.document == document && document.ends_with(".neo") {
                            for (i, text) in source
                                .lines()
                                .enumerate()
                                .skip(point.line.saturating_sub(3))
                                .take(5)
                            {
                                println!(
                                    "{} {:4} {}",
                                    if i + 1 == point.line { ">" } else { " " },
                                    i + 1,
                                    text.escape_debug()
                                );
                            }
                        } else {
                            println!(
                                "Source text is not loaded; location comes from artifact metadata."
                            );
                        }
                    } else {
                        println!("No source mapping for this frame; use il.");
                    }
                    continue;
                }
                "bt" | "stack" | "heap" | "memory" | "gc" | "output" | "status" | "il" => None,
                _ => {
                    println!("Unknown command. Type help.");
                    continue;
                }
            };
            if let Some(action) = action {
                let wait = matches!(
                    action,
                    DebugCommand::Pause
                        | DebugCommand::Step
                        | DebugCommand::Next
                        | DebugCommand::Out
                );
                if let Err(e) = debugger.command(action) {
                    println!("{e}");
                    continue;
                }
                if wait {
                    let deadline = Instant::now() + Duration::from_secs(2);
                    let mut update = debugger.snapshot();
                    while (update.revision <= snapshot.revision
                        || !matches!(
                            update.status.as_str(),
                            "paused" | "completed" | "faulted" | "stopped" | "waiting for input"
                        ))
                        && Instant::now() < deadline
                    {
                        update = debugger.wait(
                            update.revision,
                            deadline.saturating_duration_since(Instant::now()),
                        );
                    }
                    overview(&mut io::stdout().lock(), &update).map_err(|e| e.to_string())?;
                }
            } else if let Err(error) =
                inspect(&mut io::stdout().lock(), &snapshot, command, words.next())
            {
                println!("{error}");
            }
        }
        Ok(Vec::new())
    })();
    let _ = debugger.command(DebugCommand::Stop);
    alive.store(false, Ordering::Relaxed);
    let _ = watcher.join();
    // Native calls are not asynchronously interruptible. Never hang the terminal on quit.
    if worker.is_finished() {
        let _ = worker.join();
    }
    result
}
