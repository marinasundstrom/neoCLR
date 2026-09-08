//! Cooperative, read-only inspection. Snapshots contain no guest references or host addresses.
use crate::{CancellationToken, Fault};
use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Default, Serialize)]
pub struct DebugValue {
    pub ty: String,
    pub value: String,
    pub children: Vec<(String, DebugValue)>,
    pub truncated: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct DebugFrame {
    /// Frame index counted from the entry frame; valid within this snapshot.
    pub index: usize,
    pub function: String,
    pub instruction: usize,
    pub operation: String,
    pub source: Option<crate::metadata::SequencePoint>,
    pub arguments: Vec<(String, DebugValue)>,
    pub locals: Vec<(String, DebugValue)>,
    pub evaluation_stack: Vec<DebugValue>,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct DebugSnapshot {
    pub revision: u64,
    pub status: String,
    pub frames: Vec<DebugFrame>,
    pub heap: Vec<(usize, DebugValue)>,
    pub native: Vec<NativeAllocation>,
    pub gc: serde_json::Value,
    pub output: Vec<String>,
    pub result: Option<DebugValue>,
    pub fault: Option<String>,
    pub truncated: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct NativeAllocation {
    pub id: usize,
    pub frame_owned: bool,
    pub size: usize,
    /// None means guest-uninitialized. Never inspect untracked native addresses.
    pub bytes: Vec<Option<u8>>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Breakpoint {
    Instruction {
        function: String,
        instruction: usize,
    },
    Source {
        document: String,
        line: usize,
    },
}
#[derive(Debug, Clone)]
pub enum DebugCommand {
    Pause,
    Continue,
    Step,
    Next,
    Out,
    Stop,
    Break(Breakpoint),
    ClearBreakpoints,
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Paused,
    Running,
    Step,
    Stepping,
    Next(usize, Option<(String, usize)>),
    Out(usize),
    Stopped,
    Finished,
}
#[derive(Debug)]
struct State {
    mode: Mode,
    snapshot: DebugSnapshot,
    breakpoints: Vec<Breakpoint>,
    input: VecDeque<u8>,
    eof: bool,
    attached: bool,
    last_publish: Instant,
}
#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}
#[derive(Debug, Clone)]
pub struct Debugger {
    shared: Arc<Shared>,
}
impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}
impl Debugger {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    mode: Mode::Paused,
                    snapshot: DebugSnapshot {
                        status: "starting".into(),
                        ..Default::default()
                    },
                    breakpoints: vec![],
                    input: VecDeque::new(),
                    eof: false,
                    attached: false,
                    last_publish: Instant::now(),
                }),
                changed: Condvar::new(),
            }),
        }
    }
    pub fn snapshot(&self) -> DebugSnapshot {
        self.shared.state.lock().unwrap().snapshot.clone()
    }
    /// Wait at most timeout for a newer snapshot; useful for terminal or future UI hosts.
    pub fn wait(&self, revision: u64, timeout: Duration) -> DebugSnapshot {
        let state = self.shared.state.lock().unwrap();
        let (state, _) = self
            .shared
            .changed
            .wait_timeout_while(state, timeout, |s| s.snapshot.revision <= revision)
            .unwrap();
        state.snapshot.clone()
    }
    pub fn command(&self, command: DebugCommand) -> Result<(), Fault> {
        let mut s = self.shared.state.lock().unwrap();
        if matches!(s.mode, Mode::Finished | Mode::Stopped) {
            return Err(Fault::new("debug execution has ended"));
        }
        let depth = s.snapshot.frames.first().map_or(0, |f| f.index + 1);
        let source = s
            .snapshot
            .frames
            .first()
            .and_then(|f| f.source.as_ref())
            .map(|p| (p.document.clone(), p.line));
        match command {
            DebugCommand::Pause => s.mode = Mode::Paused,
            DebugCommand::Continue => s.mode = Mode::Running,
            DebugCommand::Step | DebugCommand::Next | DebugCommand::Out
                if s.mode != Mode::Paused || s.snapshot.status != "paused" =>
            {
                return Err(Fault::new("pause before stepping"));
            }
            DebugCommand::Step => s.mode = Mode::Step,
            DebugCommand::Next => s.mode = Mode::Next(depth, source),
            DebugCommand::Out => s.mode = Mode::Out(depth),
            DebugCommand::Stop => s.mode = Mode::Stopped,
            DebugCommand::Break(point) => {
                if s.breakpoints.len() >= 256 {
                    return Err(Fault::new("breakpoint limit exceeded"));
                }
                if !s.breakpoints.contains(&point) {
                    s.breakpoints.push(point);
                }
            }
            DebugCommand::ClearBreakpoints => s.breakpoints.clear(),
        }
        match s.mode {
            Mode::Running | Mode::Step | Mode::Next(..) | Mode::Out(_) => {
                s.snapshot.status = "running".into()
            }
            Mode::Stopped => s.snapshot.status = "stop requested".into(),
            Mode::Paused if s.snapshot.status != "paused" => {
                s.snapshot.status = "pause requested".into()
            }
            _ => (),
        }
        self.shared.changed.notify_all();
        Ok(())
    }
    /// Record a hosting/loader failure before any execution snapshot was published.
    pub fn launch_failure(&self, fault: &Fault) {
        let mut s = self.shared.state.lock().unwrap();
        if s.snapshot.revision == 0 {
            s.snapshot.revision = 1;
            s.snapshot.status = "faulted".into();
            s.snapshot.fault = Some(fault.to_string());
            s.mode = Mode::Finished;
            self.shared.changed.notify_all();
        }
    }
    pub fn input(&self, bytes: &[u8]) -> Result<(), Fault> {
        let mut s = self.shared.state.lock().unwrap();
        if s.input.len().saturating_add(bytes.len()) > 65536 {
            return Err(Fault::new("debug input limit exceeded"));
        }
        s.input.extend(bytes);
        self.shared.changed.notify_all();
        Ok(())
    }
    pub fn end_input(&self) {
        let mut s = self.shared.state.lock().unwrap();
        s.eof = true;
        self.shared.changed.notify_all();
    }
    pub(crate) fn begin(&self) -> Result<(), Fault> {
        let mut s = self.shared.state.lock().unwrap();
        if s.attached {
            return Err(Fault::new(
                "debugger is already attached; use a new debugger for each execution",
            ));
        }
        s.attached = true;
        Ok(())
    }
    pub(crate) fn checkpoint(
        &self,
        location: (&str, usize, usize),
        source: Option<&crate::metadata::SequencePoint>,
        cancellation: Option<&CancellationToken>,
        before_host_call: bool,
        capture: impl Fn() -> DebugSnapshot,
    ) -> Result<(), Fault> {
        let (function, pc, depth) = location;
        let mut s = self.shared.state.lock().unwrap();
        let line = source.map(|p| (p.document.clone(), p.line));
        let hit = s.breakpoints.iter().any(|b| match b {
            Breakpoint::Instruction {
                function: f,
                instruction,
            } => f == function && *instruction == pc,
            Breakpoint::Source { document, line } => source
                .is_some_and(|p| &p.document == document && p.line == *line && p.instruction == pc),
        });
        if s.mode == Mode::Stepping
            || hit
            || matches!(&s.mode, Mode::Next(d, start) if depth < *d || (depth == *d && (line != *start || start.is_none())))
            || matches!(s.mode, Mode::Out(d) if depth < d)
        {
            s.mode = Mode::Paused;
        }
        if s.mode == Mode::Paused
            || before_host_call
            || s.last_publish.elapsed() >= Duration::from_millis(100)
        {
            let mut snapshot = capture();
            snapshot.revision = s.snapshot.revision + 1;
            snapshot.status = if s.mode == Mode::Paused {
                "paused"
            } else {
                "running"
            }
            .into();
            s.snapshot = snapshot;
            s.last_publish = Instant::now();
            self.shared.changed.notify_all();
        }
        while s.mode == Mode::Paused {
            if cancellation.is_some_and(CancellationToken::is_cancelled) {
                return Err(Fault::new("execution cancelled"));
            }
            s = self
                .shared
                .changed
                .wait_timeout(s, Duration::from_millis(50))
                .unwrap()
                .0;
        }
        if s.mode == Mode::Stopped {
            return Err(Fault::new("debug execution stopped"));
        }
        if s.mode == Mode::Step {
            s.mode = Mode::Stepping;
        }
        Ok(())
    }
    pub(crate) fn finish(&self, mut snapshot: DebugSnapshot, fault: Option<String>) {
        let mut s = self.shared.state.lock().unwrap();
        snapshot.revision = s.snapshot.revision + 1;
        snapshot.status = if s.mode == Mode::Stopped {
            "stopped"
        } else if fault.is_some() {
            "faulted"
        } else {
            "completed"
        }
        .into();
        snapshot.fault = fault;
        s.snapshot = snapshot;
        s.mode = Mode::Finished;
        self.shared.changed.notify_all();
    }
}
impl crate::Console for Debugger {
    fn read_byte(&self) -> std::io::Result<Option<u8>> {
        let mut s = self.shared.state.lock().unwrap();
        while s.input.is_empty() && !s.eof && !matches!(s.mode, Mode::Stopped | Mode::Finished) {
            s.snapshot.status = "waiting for input".into();
            s.snapshot.revision += 1;
            self.shared.changed.notify_all();
            s = self.shared.changed.wait(s).unwrap();
        }
        if matches!(s.mode, Mode::Stopped | Mode::Finished) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "debug execution stopped",
            ));
        }
        Ok(s.input.pop_front())
    }
    fn write_line(&self, _: &str) -> std::io::Result<()> {
        Ok(())
    }
}
