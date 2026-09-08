use neoclr::{Limits, LoadedProgram, Value, assemble, frontend};

const BASE: &str = "ldarg 0\nldflda 0\nldarg 1\nstobj Int32\nldvoid\nret";
const CHAIN: &str = "ldarg 0\nldarg 1\ncall instance Base::.ctor(Int32)\npop";
const OWN: &str = "ldarg 0\nldflda 1\nldc.i4 2\nstobj Int32";
fn program(base: &str, derived: &str) -> LoadedProgram {
    let text = format!(
        ".module App\n.entry Main\n.type abstract Base\n.field X Int32\n.method instance byref .ctor(Int32 n) -> Void\n{base}\n.end\n.end\n.type Child\n.extends Base\n.field Y Int32\n.method instance byref .ctor(Int32 n) -> Void\n{derived}\n.end\n.end\n.function Main() -> Child\nldc.i4 40\nnewobj instance Child::.ctor(Int32)\nret\n.end"
    );
    LoadedProgram::new(&assemble(&text).unwrap()).unwrap()
}
#[test]
fn source_frame_heap_abstract_base_roundtrip_and_graph() {
    let module =
        frontend::compile(include_str!("../examples/source/constructor-chaining.neo")).unwrap();
    let module = serde_json::from_str(&serde_json::to_string(&module).unwrap()).unwrap();
    let p = LoadedProgram::new(&module).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let graph = p
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap();
    assert!(
        graph
            .functions
            .iter()
            .any(|f| f.target.name == "Counter..ctor")
    );
}
#[test]
fn chained_construction_preserves_fields() {
    let p = program(BASE, &format!("{CHAIN}\n{OWN}\nldvoid\nret"));
    p.verify().unwrap();
    let Value::Object { fields, .. } = p.run(Limits::default()).unwrap().value else {
        panic!()
    };
    assert_eq!(fields, [Value::Int32(40), Value::Int32(2)]);
}
#[test]
fn invalid_initialization_faults_even_without_verification() {
    for body in [
        "ldvoid\nret".to_owned(),
        format!("{OWN}\n{CHAIN}\nldvoid\nret"),
        format!("{CHAIN}\nldvoid\nret"),
        format!("{CHAIN}\n{CHAIN}\n{OWN}\nldvoid\nret"),
        format!("{CHAIN}\nldarg 0\nldflda 0\nldc.i4 9\nstobj Int32\n{OWN}\nldvoid\nret"),
        format!("{CHAIN}\nldarg 0\nldfld 1\npop\n{OWN}\nldvoid\nret"),
        format!("{CHAIN}\nldarg 0\nref.type\npop\n{OWN}\nldvoid\nret"),
        format!(".local Child& escape\n{CHAIN}\nldarg 0\nstloc escape\n{OWN}\nldvoid\nret"),
    ] {
        let p = program(BASE, &body);
        assert!(p.verify().is_err(), "{body}");
        assert!(p.run(Limits::default()).is_err(), "{body}");
    }
}
#[test]
fn branch_initialization_intersects_all_paths() {
    let body = format!("{CHAIN}\nldc.i4 1\nldc.i4 1\nbeq missing\n{OWN}\nmissing:\nldvoid\nret");
    let p = program(BASE, &body);
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
    let p = program(
        BASE,
        &format!(
            "{CHAIN}\nldc.bool true\nbrtrue other\n{OWN}\nbr done\nother:\n{OWN}\ndone:\nldvoid\nret"
        ),
    );
    p.verify().unwrap();
    p.run(Limits::default()).unwrap();
}
#[test]
fn generic_abstract_base_and_same_type_delegation() {
    let text = ".module App
.entry Main
.type abstract Base<T>
.field Item T
.method instance byref .ctor(T value) -> Void
ldarg 0
ldflda 0
ldarg 1
stobj T
ldvoid
ret
.end
.end
.type Child<T>
.extends Base<T>
.method instance byref .ctor(T value) -> Void
ldarg 0
ldarg 1
call instance Base<T>::.ctor(T)
pop
ldvoid
ret
.end
.method instance byref .ctor(T value, Int32 unused) -> Void
ldarg 0
ldarg 1
call instance Child<T>::.ctor(T)
pop
ldvoid
ret
.end
.end
.function Main() -> Int32
ldc.i4 42
ldc.i4 0
newobj instance Child<Int32>::.ctor(Int32,Int32)
ldfld 0
ret
.end";
    let p = LoadedProgram::new(&assemble(text).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn constructor_cycles_fault_with_guest_trace() {
    let text = ".module App\n.entry Main\n.type A\n.method instance byref .ctor() -> Void\nldarg 0\ncall instance A::.ctor()\npop\nldvoid\nret\n.end\n.end\n.function Main() -> A\nnewobj instance A::.ctor()\nret\n.end";
    let p = LoadedProgram::new(&assemble(text).unwrap()).unwrap();
    let fault = p.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("cyclic"), "{fault:?}");
    assert!(fault.stack_trace.is_some());
}
#[test]
fn gc_traces_references_stored_during_unpublished_construction() {
    let source = "record Payload(N: int)
record Holder(Item: Payload&) {
    init() {
        this.Item = new Payload(42)
        new Payload(0)
        let pressure = new Payload(1)
    }
}
func Main() -> int {
    let holder = Holder()
    return holder.Item.N
}";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(
        p.run(Limits {
            heap_objects: 2,
            ..Limits::default()
        })
        .unwrap()
        .value,
        Value::Int32(42)
    );
}

#[test]
fn ordinary_calls_cannot_observe_a_partially_constructed_receiver() {
    let source = "record Counter(N: int) {
    init() { this.N = 42; this.Read() }
    readonly virtual func Read() -> int { return this.N }
}
func Main() -> int { let c = Counter(); return c.N }";
    let il = frontend::lower_to_il(source).unwrap();
    let p = LoadedProgram::new(&assemble(&il).unwrap()).unwrap();
    assert!(p.verify().is_err());
    let fault = p.run(Limits::default()).unwrap_err();
    assert!(fault.message.contains("construction"), "{fault:?}");
}

#[test]
fn managed_field_store_supports_reference_slots_and_honors_readonly() {
    let text = ".module App
.entry Main
.type A
.field N Int32
.end
.function Main() -> Int32
.local A a
ldc.i4 1
newobj A
stloc a
ldloca a
ldc.i4 42
stfld 0
pop
ldloc a
ldfld 0
ret
.end";
    let p = LoadedProgram::new(&assemble(text).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let source = "record A(N: int) {
    readonly func Bad() -> () { }
}
func Main() -> int { let a = A(1); a.Bad(); return 0 }";
    let il = frontend::lower_to_il(source).unwrap().replace(
        "ldvoid\nret",
        "ldarg 0\nldc.i4 42\nstfld 0\npop\nldvoid\nret",
    );
    let p = LoadedProgram::new(&assemble(&il).unwrap()).unwrap();
    assert!(p.verify().is_err());
    assert!(p.run(Limits::default()).is_err());
}

#[test]
fn constructor_parameters_keep_readonly_and_output_contracts() {
    let source = "record C(N: int) {
    init(readonly input: int&, out result: int&) {
        this.N = input
        result = input
    }
}
func Main() -> int {
    let n = 21
    var answer: int
    let c = C(&n, out answer)
    return c.N + answer
}";
    let p = LoadedProgram::new(&frontend::compile(source).unwrap()).unwrap();
    p.verify().unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
