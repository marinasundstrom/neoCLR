use neoclr::{Limits, LoadedProgram, Value, assembler::parse_function_ref};

fn library() -> String {
    let output = std::process::Command::new("python3")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["-c", "import runpy; m=runpy.run_path('docs/experiments/raven-target/collection_library.py'); print(m['build'](m['ROOT'] / 'runtime/System.neoil'))"])
        .output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn load(source: &str) -> LoadedProgram {
    let library = neoclr::assemble(&library()).unwrap();
    let module = neoclr::assembler::read_modules(
        &[neoclr::assembler::ModuleInput::Source(source)],
        &library,
    )
    .unwrap()
    .remove(0);
    let program = LoadedProgram::with_library(&module, &library).unwrap();
    program.verify().unwrap();
    program
}

fn call(program: &LoadedProgram, name: &str, args: &[i32]) -> i32 {
    let result = program
        .resolve_function(&parse_function_ref(name).unwrap())
        .unwrap()
        .invoke(
            args.iter().copied().map(Value::Int32).collect(),
            Limits::default(),
        )
        .unwrap()
        .value;
    let Value::Int32(value) = result else {
        panic!("Expected int: {result:?}")
    };
    value
}

#[test]
fn hebrew_calendar_matches_dotnet_complete_years() {
    let mut source = String::from(".module CalendarComparison\n");
    source.push_str(".function Create(Int32 year,Int32 month,Int32 day) -> Int32\n.local System.Result<System.Time.Date,System.Time.InvalidDateError> result\n.local System.Time.Date date\ncall System.Time.Calendar::get_Hebrew()\nldarg year\nldarg month\nldarg day\ncall instance System.Time.Calendar::CreateDate(Int32,Int32,Int32)\nstloc result\nldloca result\nldloca date\ncall instance System.Result<System.Time.Date,System.Time.InvalidDateError>::TryGetOutput(System.Time.Date&)\nbrfalse failed\nldloca date\ncall instance System.Time.Date::get_DayNumber()\nret\nfailed:\nldc.i4 -1\nret\n.end\n");
    for (name, parameters, arguments) in [
        (
            "GetDaysInMonth",
            "Int32 year,Int32 month",
            "ldarg year\nldarg month",
        ),
        ("GetMonthsInYear", "Int32 year", "ldarg year"),
    ] {
        let sig = if name == "GetDaysInMonth" {
            "Int32,Int32"
        } else {
            "Int32"
        };
        source.push_str(&format!(".function {name}({parameters}) -> Int32\n.local System.Result<Int32,System.Time.InvalidDateError> result\n.local Int32 value\ncall System.Time.Calendar::get_Hebrew()\n{arguments}\ncall instance System.Time.Calendar::{name}({sig})\nstloc result\nldloca result\nldloca value\ncall instance System.Result<Int32,System.Time.InvalidDateError>::TryGetOutput(Int32&)\nbrfalse failed\nldloc value\nret\nfailed:\nldc.i4 -1\nret\n.end\n"));
    }
    for name in ["GetYear", "GetMonth", "GetDay"] {
        source.push_str(&format!(".function {name}(Int32 number) -> Int32\n.local System.Result<System.Time.Date,System.Time.InvalidDateError> created\n.local System.Time.Date date\n.local System.Result<Int32,System.Time.InvalidDateError> result\n.local Int32 value\nldarg number\ncall System.Time.Date::FromDayNumber(Int32)\nstloc created\nldloca created\nldloca date\ncall instance System.Result<System.Time.Date,System.Time.InvalidDateError>::TryGetOutput(System.Time.Date&)\npop\ncall System.Time.Calendar::get_Hebrew()\nldloc date\ncall instance System.Time.Calendar::{name}(System.Time.Date)\nstloc result\nldloca result\nldloca value\ncall instance System.Result<Int32,System.Time.InvalidDateError>::TryGetOutput(Int32&)\nbrfalse failed\nldloc value\nret\nfailed:\nldc.i4 -1\nret\n.end\n"));
    }
    source.push_str(
        r#"
.function CheckYear(Int32 year,Int32 start,Int32 count,Int32 mask) -> Int32
.local Int32 month
.local Int32 number
.local Int32 days
.local Int32 date
.local Int32 day
ldarg year
call GetMonthsInYear(Int32)
ldarg count
bne.un failed
ldarg year
ldc.i4 1
ldc.i4 1
call Create(Int32,Int32,Int32)
ldarg start
bne.un failed
ldarg start
stloc number
ldc.i4 1
stloc month
month_loop:
ldarg mask
ldloc month
ldc.i4 1
sub
shr
ldc.i4 1
and
ldc.i4 29
add
stloc days
ldarg year
ldloc month
call GetDaysInMonth(Int32,Int32)
ldloc days
bne.un failed
ldarg year
ldc.i4 5344
beq check_days
ldarg year
ldc.i4 5999
beq check_days
ldarg year
ldc.i4 5770
blt next_month
ldarg year
ldc.i4 5789
bgt next_month
check_days:
ldc.i4 1
stloc day
day_loop:
ldloc number
ldloc day
add
ldc.i4 1
sub
stloc date
ldarg year
ldloc month
ldloc day
call Create(Int32,Int32,Int32)
ldloc date
bne.un failed
ldloc date
call GetYear(Int32)
ldarg year
bne.un failed
ldloc date
call GetMonth(Int32)
ldloc month
bne.un failed
ldloc date
call GetDay(Int32)
ldloc day
bne.un failed
ldloc day
ldc.i4 1
bne.un next_month
ldloc days
stloc day
br day_loop
next_month:
ldloc number
ldloc days
add
stloc number
ldloc month
ldc.i4 1
add
stloc month
ldloc month
ldarg count
ble month_loop
ldc.i4 1
ret
failed:
ldc.i4 0
ret
.end
"#,
    );
    let program = load(&source);
    let rows: Vec<Vec<i32>> =
        serde_json::from_str(include_str!("fixtures/hebrew-calendar.json")).unwrap();
    assert_eq!(rows.len(), 656);
    let check = program
        .resolve_function(&parse_function_ref("CheckYear(Int32,Int32,Int32,Int32)").unwrap())
        .unwrap();
    for row in &rows {
        let result = check
            .invoke(
                vec![
                    Value::Int32(row[0]),
                    Value::Int32(row[1]),
                    Value::Int32(row.len() as i32 - 2),
                    Value::Int32(
                        row[2..]
                            .iter()
                            .enumerate()
                            .fold(0, |bits, (index, days)| bits | ((days - 29) << index)),
                    ),
                ],
                Limits {
                    instructions: 1_000_000,
                    ..Limits::default()
                },
            )
            .unwrap()
            .value;
        assert_eq!(result, Value::Int32(1), "Hebrew year {}", row[0]);
    }
    assert_eq!(call(&program, "GetYear(Int32)", &[rows[0][1] - 1]), -1);
    let last = rows.last().unwrap();
    assert_eq!(
        call(
            &program,
            "GetYear(Int32)",
            &[last[1] + last[2..].iter().sum::<i32>()]
        ),
        -1
    );
}

#[test]
fn calendar_policies_cannot_be_constructed_or_mutated_by_guest_il() {
    let library = neoclr::assemble(&library()).unwrap();
    for body in [
        "ldc.i4 2\nnewobj instance System.Globalization.Culture::.ctor(Int32)\npop",
        "call System.Globalization.Culture::get_Invariant()\nldc.i4 1\nstfld System.Globalization.Culture::kind\npop",
    ] {
        let source =
            format!(".module InvalidCalendar\n.function Main() -> noresult\n{body}\nret\n.end\n");
        let error = neoclr::assembler::read_modules(
            &[neoclr::assembler::ModuleInput::Source(&source)],
            &library,
        )
        .and_then(|modules| {
            let program = LoadedProgram::with_library(&modules[0], &library)?;
            program.verify()
        })
        .unwrap_err();
        assert!(error.to_string().contains("access denied"), "{error}");
    }
}
