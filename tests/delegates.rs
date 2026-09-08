use neoclr::{Limits, LoadedProgram, Value, assemble, run, verify};

const DECL: &str = ".delegate Transform\n.method instance Invoke(Int32) -> Int32\n.end\n.end";
fn module(body: &str, extras: &str) -> neoclr::Module {
    assemble(&format!(
        ".module Test\n.entry Main\n{DECL}\n{extras}\n.function Main() -> Int32\n{body}\nret\n.end"
    ))
    .unwrap()
}
#[test]
fn static_closed_generic_binding_roundtrips_and_invokes() {
    let m = module(
        "delegate.bind Transform = Identity<Int32>(Int32)\nldc.i4 42\ncallvirt instance Transform::Invoke(Int32)",
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
    assert_eq!(main.delegate_invocations.len(), 1);
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
        ".local Transform callback\nldc.i4 40\nnewobj Counter\nheap.new\ndelegate.bind Transform = instance Counter::Add(Int32)\nstloc callback\nldc.i4 100\nnewobj Counter\nheap.new\npop\nldc.i4 200\nnewobj Counter\nheap.new\npop\nldloc callback\nldc.i4 2\ncall instance Transform::Invoke(Int32)",
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
        ".local Counter value\nldc.i4 40\nnewobj Counter\nstloc value\nldloca value\ndelegate.bind Transform = instance Counter::Add(Int32)\nldc.i4 2\ncall instance Transform::Invoke(Int32)",
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
fn incompatible_signature_and_invalid_default_are_rejected() {
    assert!(assemble(&format!(".module Bad\n{DECL}\n.function Wrong(Int32) -> Boolean\nldc.bool true\nret\n.end\n.function Main() -> Void\ndelegate.bind Transform = Wrong(Int32)\nret\n.end")).unwrap_err().message.contains("signature"));
    assert!(assemble(&format!(".module Bad\n{DECL}\n.function Main() -> Int32\n.local Transform d\nldloca d\ninitobj Transform\nldc.i4 0\nret\n.end")).is_err());
}

fn neo(source: &str) -> LoadedProgram {
    LoadedProgram::new(&neoclr::frontend::compile(source).unwrap()).unwrap()
}
#[test]
fn neo_declarations_method_groups_and_invocation() {
    let p = neo(include_str!("../examples/source/delegates.neo"));
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
            "delegate Reader() -> int
{declarations}
func Main() -> int {{ {setup}; let callback = Reader(receiver.Read); return callback() }}"
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
    let p = neo("delegate Setter(out value: int&) -> ()
delegate Reader(readonly value: int&) -> int
func Set(out value: int&) -> () { value = 42 }
func Read(readonly value: int&) -> int { return value }
func Main() -> int { var value: int; let set = Setter(Set); set(out value); let read = Reader(Read); return read(&value) }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn nominal_identity_and_reference_permissions_are_checked() {
    for source in [
        "delegate A(value: int) -> int
delegate B(value: int) -> int
func Identity(value: int) -> int { return value }
func Main() -> int { let b: B = A(Identity); return b(42) }",
        "delegate Reader() -> int
class Counter { func Read() -> int { return 42 } }
func Main() -> int { let c: readonly Counter& = new Counter(); let r = Reader(c.Read); return r() }",
        "delegate Setter(out value: int&) -> ()
func Ignore(value: int&) -> () {}
func Main() -> int { let s = Setter(Ignore); return 0 }",
        "delegate Reader() -> int
class Counter { func Read() -> int { return 42 } }
func Main() -> int { var c = Counter(); let r = Reader(c.Read); return r() }",
    ] {
        assert!(neoclr::frontend::compile(source).is_err(), "{source}");
    }
}

#[test]
fn func_family_unifies_void_results_and_reference_arguments() {
    let p = neo("func Increment(value: int&) -> () { value = value + 1 }
func Noop() -> () {}
func Main() -> int { var value = 41; let callback = System.Func<int&, void>(Increment); callback(&value); let noop = System.Func<void>(Noop); noop.Invoke(); return value }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
#[test]
fn managed_array_foreach_accepts_stack_and_heap_arrays() {
    let source = "import System.Console.*
func Print(value: int) -> () { WriteLine(value) }
func Main() -> int { var values: int[2] = [41, 42]; let callback = System.Func<int, void>(Print); System.Array.ForEach<int>(&values, callback); let heap = new int[1] { 43 }; System.Array.ForEach<int>(heap, callback); return 0 }";
    let p = neo(source);
    assert_eq!(p.run(Limits::default()).unwrap().output, ["41", "42", "43"]);
}
#[test]
fn copied_delegates_share_target_and_storage_through_gc() {
    let p = neo("delegate Reader() -> int
class Counter { var Value: int = 40; func Next() -> int { this.Value = this.Value + 1; return this.Value } }
class Holder { var Callback: Reader; init(callback: Reader) { this.Callback = callback } }
func Make() -> Holder { let c = new Counter(); return Holder(Reader(c.Next)) }
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
.delegate Reader
.method instance Invoke() -> Int32
.end
.end
.type Secret
.method private static Read() -> Int32
ldc.i4 42
ret
.end
.method static Create() -> Reader
delegate.bind Reader = Secret::Read()
ret
.end
.end
.function Main() -> Int32
call Secret::Create()
call instance Reader::Invoke()
ret
.end";
    let m = assemble(source).unwrap();
    verify(&m).unwrap();
    assert_eq!(run(&m, Limits::default()).unwrap().value, Value::Int32(42));
    assert!(
        assemble(&source.replace(
            "call Secret::Create()",
            "delegate.bind Reader = Secret::Read()"
        ))
        .is_err()
    );
}
#[test]
fn binding_rejects_unpublished_constructor_receiver() {
    let source = "delegate Reader() -> int
class Counter { var Value: int; init() { let callback = Reader(this.Read); this.Value = 42 }; func Read() -> int { return this.Value } }
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
delegate.bind Transform = instance Counter::Add(Int32)
ldloc receiver
delegate.bind Transform = instance Counter::Add(Int32)
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
fn func_maximum_bundled_arity_and_delegate_reference_access() {
    let p = neo("func Sum(a: int, b: int, c: int, d: int) -> int { return a+b+c+d }
func Main() -> int { var f = System.Func<int,int,int,int,int>(Sum); let r = &f; return r.Invoke(10,10,10,12) }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}

#[test]
fn wrong_nominal_callable_faults_even_without_verification() {
    let m = module(
        "delegate.bind Other = Identity(Int32)
ldc.i4 42
call instance Transform::Invoke(Int32)",
        ".delegate Other
.method instance Invoke(Int32) -> Int32
.end
.end
.function Identity(Int32) -> Int32
ldarg 0
ret
.end",
    );
    assert!(verify(&m).is_err());
    assert!(
        run(&m, Limits::default())
            .unwrap_err()
            .message
            .contains("nominal")
    );
}
#[test]
fn func_sample_and_void_generic_return_binding() {
    let p = neo(include_str!("../examples/source/func-callbacks.neo"));
    let r = p.run(Limits::default()).unwrap();
    assert_eq!(r.value, Value::Int32(42));
    assert_eq!(r.output, ["41", "42", "43"]);
    let p = neo("func Identity<T>(value: T) -> T { return value }
func Main() -> () { let callback = System.Func<void, void>(Identity<void>); callback(default(void)) }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Void);
}

#[test]
fn contextual_method_groups_bind_in_parameters_returns_and_slots() {
    let p = neo("delegate Transform(value: int) -> int
func Add(value: int) -> int { return value+1 }
func Identity<T>(value: T) -> T { return value }
func Apply(callback: Transform) -> int { return callback(41) }
func Make() -> Transform { return Add }
func Main() -> int { let f: System.Func<int,int> = Identity<int>; if f(42) == 42 { let copy: Transform = Make(); return Apply(Add) }; return 0 }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
    let p = neo("import System.Console.*
func Print(value: int) -> () { WriteLine(value) }
func Main() -> () { var values: int[1] = [42]; System.Array.ForEach<int>(&values, Print) }");
    assert_eq!(p.run(Limits::default()).unwrap().output, ["42"]);
}

#[test]
fn delegate_valued_expressions_and_bound_receiver_evaluate_once() {
    let source = "import System.Console.*
delegate Reader() -> int
class Counter { var Value: int = 41; func Next() -> int { this.Value = this.Value+1; return this.Value } }
class Holder { var Callback: Reader; init(callback: Reader) { this.Callback = callback } }
func Receiver() -> Counter& { WriteLine(1); return new Counter() }
func Make() -> Reader { return Receiver().Next }
func Main() -> int { let holder = Holder(Make()); holder.Callback(); return Make()() }";
    let r = neo(source).run(Limits::default()).unwrap();
    assert_eq!(r.value, Value::Int32(42));
    assert_eq!(r.output, ["1", "1"]);
}

#[test]
fn delegate_binding_honors_declaring_module_references() {
    let sources = [
        ".module App\n.references (System, Helpers)\n.entry Main\n.function Main() -> Int32\ndelegate.bind System.Func<Int32> = Read()\ncall instance System.Func<Int32>::Invoke()\nret\n.end",
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
        r#"func Main() -> () { var values: int[1] = [42]; System.Array.ForEach<int>(&values, System.Console.WriteLine); let print: System.Func<string,void> = System.Console.WriteLine; print("done") }"#,
    );
    assert_eq!(p.run(Limits::default()).unwrap().output, ["42", "done"]);
}

#[test]
fn readonly_composed_func_parameters_preserve_substituted_signatures() {
    let p = neo("func Read(readonly value: int&) -> int { return value }
func Identity<T>(value: T) -> T { return value }
func Main() -> int { var value = 42; let read: System.Func<readonly int&, int> = Read; let identity: System.Func<readonly int&, readonly int&> = Identity<readonly int&>; return read(identity(&value)) }");
    assert_eq!(p.run(Limits::default()).unwrap().value, Value::Int32(42));
}
