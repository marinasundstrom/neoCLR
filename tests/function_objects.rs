use neoclr::{Limits, LoadedProgram, Value, assemble, run, verify};

fn module(body: &str, extras: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n{extras}\n.function Main() -> Int32\n{body}\nret\n.end"
    ))
    .unwrap()
}
#[test]
fn static_closed_generic_binding_roundtrips_and_invokes() {
    let m = module(
        "function.bind fn<Int32,Int32> = Identity<Int32>(Int32)\nldc.i4 42\ncallvirt instance fn<Int32,Int32>::Invoke(Int32)",
        ".function Identity<T>(T) -> T\nldarg 0\nret\n.end",
    );
    verify(&m).unwrap();
    let m = neoclr::load(&serde_json::to_string(&m).unwrap()).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    let graph = LoadedProgram::new(&m)
        .unwrap()
        .analyze_reachability(
            &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
            100,
        )
        .unwrap();
    let main = &graph.functions[graph.roots[0]];
    assert_eq!(main.bindings.len(), 1);
    assert_eq!(main.function_invocations.len(), 1);
    assert!(main.calls.is_empty());
    assert_eq!(
        graph.functions[main.bindings[0].target]
            .target
            .generic_arguments,
        [neoclr::metadata::Type::Int32]
    );
}

const COUNTER: &str = ".type Counter\n.field Value Int32\n.method instance byref Add(Int32) -> Int32\nldarg 0\nldflda Counter::Value\nldarg 0\nldfld Counter::Value\nldarg 1\nadd\nstobj Int32\nldarg 0\nldfld Counter::Value\nret\n.end\n.end";

#[test]
fn heap_receiver_is_shared_and_retained_through_gc() {
    let m = module(
        ".local fn<Int32,Int32> callback\nldc.i4 40\nnewobj Counter\nheap.new\nfunction.bind fn<Int32,Int32> = instance Counter::Add(Int32)\nstloc callback\nldc.i4 100\nnewobj Counter\nheap.new\npop\nldc.i4 200\nnewobj Counter\nheap.new\npop\nldloc callback\nldc.i4 2\ncall instance fn<Int32,Int32>::Invoke(Int32)",
        COUNTER,
    );
    verify(&m).unwrap();
    let result = run(
        &m,
        Limits {
            heap_objects: 2,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(result.value, Value::Int32(42));
    assert!(result.heap.statistics().collections > 0);
}

#[test]
fn frame_capture_is_rejected_without_verifier() {
    let m = module(
        ".local Counter value\nldc.i4 40\nnewobj Counter\nstloc value\nldloca value\nfunction.bind fn<Int32,Int32> = instance Counter::Add(Int32)\nldc.i4 2\ncall instance fn<Int32,Int32>::Invoke(Int32)",
        COUNTER,
    );
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("frame-backed")
    );
}

#[test]
fn incompatible_signature_is_rejected() {
    assert!(assemble(&format!(".module Bad\n.function Wrong(Int32) -> Boolean\nldc.bool true\nret\n.end\n.function Main() -> Void\nfunction.bind fn<Int32,Int32> = Wrong(Int32)\nret\n.end")).unwrap_err().message.contains("signature"));
}

fn neo(source: &str) -> LoadedProgram {
    LoadedProgram::new(&neoclr::frontend::compile(source).unwrap()).unwrap()
}
#[test]
fn neo_declarations_method_groups_and_invocation() {
    let p = neo(include_str!("../examples/source/function-objects.neo"));
    let r = p.run(Limits::default()).unwrap();
    assert_eq!(r.value, Value::Int32(42));
    assert_eq!(r.output, ["42", "41"]);
}
#[test]
fn virtual_and_interface_binding_preserve_selected_receiver() {
    for (declarations, setup, expected) in [
        (
            "class Base { virtual func Read() -> int { return 1 } }; class Derived: Base { override func Read() -> int { return 42 } }",
            "let receiver: Base& = new Derived()",
            42,
        ),
        (
            "interface Readable { readonly func Read() -> int { return 42 } }; class Counter: Readable {}",
            "let receiver: readonly Readable& = new Counter()",
            42,
        ),
        (
            "interface Readable { readonly func Read() -> int }; class Counter: Readable { readonly func Readable.Read() -> int { return 42 } }",
            "let receiver: readonly Readable& = new Counter()",
            42,
        ),
    ] {
        let p = neo(&format!(
            "{declarations}
func Main() -> int {{ {setup}; let callback = fn<int>(receiver.Read); return callback() }}"
        ));
        assert_eq!(
            p.run(Limits::default()).unwrap().value,
            Value::Int32(expected)
        );
        let graph = p
            .analyze_reachability(
                &[neoclr::assembler::parse_function_ref("Main()").unwrap()],
                100,
            )
            .unwrap();
        assert!(!graph.functions[graph.roots[0]].bindings.is_empty());
    }
}
#[test]
fn output_and_readonly_arguments_use_ordinary_call_checks() {
    let p = neo("func Set(out value: int&) -> () { value = 42 }
func Read(readonly value: int&) -> int { return value }
func Main() -> int { var value: int; let set = fn<out int&,Void>(Set); set(out value); let read = fn<readonly int&,int>(Read); return read(&value) }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn reference_permissions_are_checked() {
    for source in [
        "class Counter { func Read() -> int { return 42 } }
func Main() -> int { let c: readonly Counter& = new Counter(); let r = fn<int>(c.Read); return r() }",
        "func Ignore(value: int&) -> () {}
func Main() -> int { let s = fn<out int&,Void>(Ignore); return 0 }",
        "class Counter { func Read() -> int { return 42 } }
func Main() -> int { var c = Counter(); let r = fn<int>(c.Read); return r() }",
    ] {
        assert!(neoclr::frontend::compile(source).is_err(), "{source}");
    }
}

#[test]
fn function_shapes_unifies_void_results_and_reference_arguments() {
    let p = neo("func Increment(value: int&) -> () { value = value + 1 }
func Noop() -> () {}
func Main() -> int { var value = 41; let callback = fn<int&, void>(Increment); callback(&value); let noop = fn<void>(Noop); noop.Invoke(); return value }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn managed_array_foreach_accepts_stack_and_heap_arrays() {
    let source = "import System.Console.*
func Print(value: int) -> () { WriteLine(value) }
func Main() -> int { var values: int[2] = [41, 42]; let callback = fn<int, void>(Print); System.Array.ForEach<int>(&values, callback); let heap = new int[1] { 43 }; System.Array.ForEach<int>(heap, callback); return 0 }";
    let p = neo(source);
    assert_eq!(p.run(Limits::default()).unwrap().output, ["41", "42", "43"]);
}
#[test]
fn copied_function_objects_share_target_and_storage_through_gc() {
    let p = neo("class Counter { var Value: int = 40; func Next() -> int { this.Value = this.Value + 1; return this.Value } }
class Holder { var Callback: fn<int>; init(callback: fn<int>) { this.Callback = callback } }
func Make() -> Holder { let c = new Counter(); return Holder(fn<int>(c.Next)) }
func Main() -> int { var holder = Make(); let copy = holder.Callback; for i in 0..<30 { let garbage = new Counter() }; holder.Callback.Invoke(); return copy() }");
    let r = p
        .run(Limits {
            heap_objects: 3,
            ..Limits::default()
        })
        .unwrap();
    assert_eq!(r.value, Value::Int32(42));
    assert!(r.heap.statistics().collections > 5);
}
#[test]
fn private_binding_transfers_capability_but_cannot_be_forged() {
    let source = ".module Test
.entry Main
.type Secret
.method private static Read() -> Int32
ldc.i4 42
ret
.end
.method static Create() -> fn<Int32>
function.bind fn<Int32> = Secret::Read()
ret
.end
.end
.function Main() -> Int32
call Secret::Create()
call instance fn<Int32>::Invoke()
ret
.end";
    let m = assemble(source).unwrap();
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    assert!(
        assemble(&source.replace(
            "call Secret::Create()",
            "function.bind Reader = Secret::Read()"
        ))
        .is_err()
    );
}
#[test]
fn binding_rejects_unpublished_constructor_receiver() {
    let source = "class Counter { var Value: int; init() { let callback = fn<int>(this.Read); this.Value = 42 }; func Read() -> int { return this.Value } }
func Main() -> int { let c = new Counter(); return 0 }";
    let il = neoclr::frontend::lower_to_il(source).unwrap();
    let m = assemble(&il).unwrap();
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("construction")
    );
}

#[test]
fn single_target_equality_uses_closed_method_and_receiver_identity() {
    let m = module(
        ".local Counter& receiver
ldc.i4 40
newobj Counter
heap.new
stloc receiver
ldloc receiver
function.bind fn<Int32,Int32> = instance Counter::Add(Int32)
ldloc receiver
function.bind fn<Int32,Int32> = instance Counter::Add(Int32)
ceq
brfalse Different
ldc.i4 42
ret
Different:
ldc.i4 0",
        COUNTER,
    );
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn four_parameter_function_and_function_reference_access() {
    let p = neo("func Sum(a: int, b: int, c: int, d: int) -> int { return a+b+c+d }
func Main() -> int { var f = fn<int,int,int,int,int>(Sum); let r = &f; return r.Invoke(10,10,10,12) }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn wrong_function_shape_faults_even_without_verification() {
    let m = module(
        "function.bind fn<Int32,Boolean> = IsAnswer(Int32)
ldc.i4 42
call instance fn<Int32,Int32>::Invoke(Int32)",
        ".function IsAnswer(Int32) -> Boolean
ldc.bool true
ret
.end",
    );
    assert!(verify(&m).is_err());
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("shape")
    );
}
#[test]
fn function_sample_and_void_generic_return_binding() {
    let p = neo(include_str!("../examples/source/func-callbacks.neo"));
    let r = p.run(Limits::default()).unwrap();
    assert_eq!(r.value, Value::Int32(42));
    assert_eq!(r.output, ["41", "42", "43"]);
    let p = neo("func Identity<T>(value: T) -> T { return value }
func Main() -> () { let callback = fn<void, void>(Identity<void>); callback(default(void)) }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Void);
}

#[test]
fn contextual_method_groups_bind_in_parameters_returns_and_slots() {
    let p = neo("func Add(value: int) -> int { return value+1 }
func Identity<T>(value: T) -> T { return value }
func Apply(callback: fn<int,int>) -> int { return callback(41) }
func Make() -> fn<int,int> { return Add }
func Main() -> int { let f: fn<int,int> = Identity<int>; if f(42) == 42 { let copy: fn<int,int> = Make(); return Apply(Add) }; return 0 }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let p = neo("import System.Console.*
func Print(value: int) -> () { WriteLine(value) }
func Main() -> () { var values: int[1] = [42]; System.Array.ForEach<int>(&values, Print) }");
    assert_eq!(p.run(Limits::default()).unwrap().output, ["42"]);
}

#[test]
fn function_valued_expressions_and_bound_receiver_evaluate_once() {
    let source = "import System.Console.*
class Counter { var Value: int = 41; func Next() -> int { this.Value = this.Value+1; return this.Value } }
class Holder { var Callback: fn<int>; init(callback: fn<int>) { this.Callback = callback } }
func Receiver() -> Counter& { WriteLine(1); return new Counter() }
func Make() -> fn<int> { return Receiver().Next }
func Main() -> int { let holder = Holder(Make()); holder.Callback(); return Make()() }";
    let r = neo(source).run(Limits::default()).unwrap();
    assert_eq!(r.value, Value::Int32(42));
    assert_eq!(r.output, ["1", "1"]);
}

#[test]
fn function_binding_honors_declaring_module_references() {
    let sources = [
        ".module App\n.references (System, Helpers)\n.entry Main\n.function Main() -> Int32\nfunction.bind fn<Int32> = Read()\ncall instance fn<Int32>::Invoke()\nret\n.end",
        ".module Helpers\n.function Read() -> Int32\nldc.i4 42\nret\n.end",
    ];
    let mut modules = neoclr::assembler::assemble_modules(&sources).unwrap();
    let p = LoadedProgram::with_modules(
        &modules[0],
        neoclr::library::system().unwrap(),
        &modules[1..],
    )
    .unwrap();
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    modules[0].references = Some(vec![neoclr::metadata::ModuleReference::Name(
        "System".into(),
    )]);
    assert!(
        LoadedProgram::with_modules(
            &modules[0],
            neoclr::library::system().unwrap(),
            &modules[1..]
        )
        .unwrap_err()
        .message
        .contains("does not reference")
    );
}

#[test]
fn expected_signature_selects_bundled_static_method_group_overload() {
    let p = neo(
        r#"func Main() -> () { var values: int[1] = [42]; System.Array.ForEach<int>(&values, System.Console.WriteLine); let print: fn<string,void> = System.Console.WriteLine; print("done") }"#,
    );
    assert_eq!(p.run(Limits::default()).unwrap().output, ["42", "done"]);
}

#[test]
fn readonly_composed_func_parameters_preserve_substituted_signatures() {
    let p = neo("func Read(readonly value: int&) -> int { return value }
func Identity<T>(value: T) -> T { return value }
func Main() -> int { var value = 42; let read: fn<readonly int&, int> = Read; let identity: fn<readonly int&, readonly int&> = Identity<readonly int&>; return read(identity(&value)) }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

fn class_with_function_constructor(body: &str) -> neoclr::Module {
    module(
        "function.bind fn<Int32,Int32> = Identity(Int32)\nnewobj instance Holder::.ctor(fn<Int32,Int32>)\nldc.i4 42\ncall instance Holder::Apply(Int32)",
        &format!(
            r#"
.type class Holder
.field private callback fn<Int32,Int32>
.method instance .ctor(fn<Int32,Int32> value) -> void
{body}
ret
.end
.method instance Apply(Int32 value) -> Int32
ldarg this
ldfld Holder::callback
ldarg value
callvirt instance fn<Int32,Int32>::Invoke(Int32)
ret
.end
.end
.function Identity(Int32 value) -> Int32
ldarg value
ret
.end
"#
        ),
    )
}

#[test]
fn class_constructor_can_initialize_a_function_field() {
    let m = class_with_function_constructor("ldarg this\nldarg value\nstfld Holder::callback");
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn class_constructor_must_initialize_a_function_field_before_return() {
    let m = class_with_function_constructor("");
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("uninitialized field")
    );
}

#[test]
fn class_constructor_cannot_read_a_function_field_before_assignment() {
    let m = class_with_function_constructor(
        "ldarg this\nldfld Holder::callback\npop\nldarg this\nldarg value\nstfld Holder::callback",
    );
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("uninitialized")
    );
}

#[test]
fn default_function_payload_is_null_and_cannot_be_invoked() {
    let source = format!(
        ".module Test\n.entry Main\n.type Holder\n.field Callback fn<Int32,Int32>\n.end\n.function Main() -> fn<Int32,Int32>\n.local Holder holder\nldloca holder\ninitobj Holder\nldloc holder\nldfld Holder::Callback\nret\n.end"
    );
    let m = assemble(&source).unwrap();
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap().value,
        Value::NullObjectReference(neoclr::assembler::parse_type("fn<Int32,Int32>").unwrap())
    );
    let m = module(
        ".local fn<Int32,Int32> callback\nldloca callback\ninitobj fn<Int32,Int32>\nldloc callback\nldc.i4 42\ncallvirt instance fn<Int32,Int32>::Invoke(Int32)",
        "",
    );
    verify(&m).unwrap();
    assert_eq!(
        run(&m, Limits::default()).unwrap_err().code,
        neoclr::FaultCode::NullReference
    );
}
