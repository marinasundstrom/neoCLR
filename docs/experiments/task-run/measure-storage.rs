//! Focused single-thread regression probe for the shared-storage prerequisite.
//! Compile against baseline/candidate release libraries; load outside timing.
use neoclr::{Limits, LoadedProgram, Value, assemble};
use std::time::Instant;

fn main() {
    let body = r#"
.module StorageCost
.entry Main
.type class Cell
.field Number Int32
.end
.function Main() -> Int32
.local Cell cell
.local Int32 count
ldc.i4 0
newobj Cell
stloc cell
ldc.i4 0
stloc count
again:
ldloc cell
ldloc count
stfld Cell::Number
ldloc cell
ldfld Cell::Number
ldc.i4 1
add
stloc count
ldloc count
ldc.i4 200000
clt
brtrue again
ldloc count
ret
.end
"#;
    let program = LoadedProgram::new(&assemble(body).unwrap()).unwrap();
    program.verify().unwrap();
    for iteration in 0..6 {
        let started = Instant::now();
        let result = program
            .run(Limits {
                instructions: 4_000_000,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(result.value, Value::Int32(200_000));
        println!("{iteration},{}", started.elapsed().as_secs_f64());
    }
}
