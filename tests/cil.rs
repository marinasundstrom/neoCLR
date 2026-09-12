use neoclr::cil::{MAX_CODE_BYTES, Operation as Op, decode};

#[test]
fn static_corpus_preserves_byte_offsets_and_tokens() {
    let body = decode(&[0x00, 0x72, 1, 0, 0, 0x70, 0x28, 2, 0, 0, 0x0a, 0x00, 0x2a]).unwrap();
    assert_eq!(
        body.iter().map(|i| i.offset).collect::<Vec<_>>(),
        [0, 1, 6, 11, 12]
    );
    assert_eq!(
        body.iter().map(|i| i.operation.clone()).collect::<Vec<_>>(),
        [
            Op::Nop,
            Op::LoadString(0x70000001),
            Op::Call(0x0a000002),
            Op::Nop,
            Op::Return
        ]
    );
    assert_eq!(
        decode(&[0x28, 1, 0, 0, 6, 0x2a]).unwrap()[0].operation,
        Op::Call(0x06000001)
    );
}

#[test]
fn integer_forms_preserve_signed_values() {
    for opcode in 0x15..=0x1e {
        assert_eq!(
            decode(&[opcode]).unwrap()[0].operation,
            Op::Int32(i32::from(opcode) - 0x16)
        );
    }
    for value in [i8::MIN, -1, 0, 42, i8::MAX] {
        assert_eq!(
            decode(&[0x1f, value as u8]).unwrap()[0].operation,
            Op::Int32(i32::from(value))
        );
    }
    for value in [i32::MIN, -1000, 0, i32::MAX] {
        let mut code = vec![0x20];
        code.extend(value.to_le_bytes());
        assert_eq!(decode(&code).unwrap()[0].operation, Op::Int32(value));
    }
}

#[test]
fn slot_encodings_normalize_without_losing_offsets() {
    let body = decode(&[
        0x02, 0x09, 0x0d, 0x0e, 255, 0x11, 254, 0x13, 253, 0xfe, 0x09, 0, 1, 0xfe, 0x0c, 1, 1,
        0xfe, 0x0e, 0xff, 0xff,
    ])
    .unwrap();
    assert_eq!(
        body.iter().map(|i| i.operation.clone()).collect::<Vec<_>>(),
        [
            Op::LoadArgument(0),
            Op::LoadLocal(3),
            Op::StoreLocal(3),
            Op::LoadArgument(255),
            Op::LoadLocal(254),
            Op::StoreLocal(253),
            Op::LoadArgument(256),
            Op::LoadLocal(257),
            Op::StoreLocal(65535)
        ]
    );
    assert_eq!(body.last().unwrap().offset, 17);
}

#[test]
fn truncated_operands_report_the_start_of_the_instruction() {
    for instruction in [
        vec![0x20, 1, 2, 3, 4],
        vec![0x28, 1, 0, 0, 6],
        vec![0x72, 1, 0, 0, 0x70],
        vec![0xfe, 0x09, 0, 1],
        vec![0x1f, 42],
        vec![0x0e, 1],
        vec![0x11, 1],
        vec![0x13, 1],
    ] {
        for length in 1..instruction.len() {
            let mut code = vec![0x00];
            code.extend(&instruction[..length]);
            let error = decode(&code).unwrap_err();
            assert_eq!(error.offset, 1);
            assert!(error.message.contains("truncated"));
        }
    }
}

#[test]
fn invalid_tokens_unsupported_code_and_limits_are_rejected() {
    for (opcode, token) in [
        (0x28, 0x06000000u32),
        (0x28, 0x04000001),
        (0x28, 0x2b000001),
        (0x72, 0x70000000),
        (0x72, 0x01000001),
    ] {
        let mut code = vec![opcode];
        code.extend(token.to_le_bytes());
        assert!(decode(&code).unwrap_err().message.contains("token"));
    }
    // Also reject unsupported instructions in unreachable bytes after ret.
    for code in [
        vec![0x2a, 0x2b, 0],
        vec![0xfe, 0x14],
        vec![0x6f],
        vec![0xff],
    ] {
        assert!(decode(&code).unwrap_err().message.contains("unsupported"));
    }
    assert!(decode(&[]).is_err());
    assert_eq!(
        decode(&vec![0x00; MAX_CODE_BYTES]).unwrap().len(),
        MAX_CODE_BYTES
    );
    assert!(decode(&vec![0x00; MAX_CODE_BYTES + 1]).is_err());
}

#[test]
fn dotnet_emitted_static_calls_decode_and_execute_in_neoclr() {
    use neoclr::metadata::Instruction as RuntimeOp;
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../docs/experiments/cil-decoder/result.json")).unwrap();
    let mut module = neoclr::assemble(".module Calls\n.entry Main\n.function Answer() -> Int32\nldc.i4 0\nret\n.end\n.function Main() -> Int32\ncall Answer()\nret\n.end").unwrap();
    let target = match &module.functions[1].body[0] {
        RuntimeOp::Call(target) => target.clone(),
        _ => unreachable!(),
    };
    let answer_token = fixture["Methods"][0]["Token"].as_u64().unwrap() as u32;
    // Test-only binding for this closed fixture, not a general metadata loader.
    for (function, method) in module
        .functions
        .iter_mut()
        .zip(fixture["Methods"].as_array().unwrap())
    {
        assert_eq!(function.name, method["Name"].as_str().unwrap());
        let bytes = method["Bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
            .collect::<Vec<_>>();
        function.body = decode(&bytes)
            .unwrap()
            .into_iter()
            .filter_map(|instruction| match instruction.operation {
                Op::Nop => None,
                Op::Int32(value) => Some(RuntimeOp::Int(value)),
                Op::Call(token) => {
                    assert_eq!(token, answer_token);
                    Some(RuntimeOp::Call(target.clone()))
                }
                Op::Return => Some(RuntimeOp::Return),
                other => panic!("unexpected fixture operation: {other:?}"),
            })
            .collect();
    }
    neoclr::verify(&module).unwrap();
    assert_eq!(
        neoclr::run(&module, neoclr::Limits::default())
            .unwrap()
            .value,
        neoclr::Value::Int32(fixture["Result"].as_i64().unwrap() as i32)
    );
}

#[test]
fn constructor_and_field_operands_preserve_standard_tokens() {
    let body = decode(&[0x73, 1, 0, 0, 6, 0x7b, 2, 0, 0, 4, 0x7d, 3, 0, 0, 0x0a]).unwrap();
    assert_eq!(
        body.iter().map(|i| i.operation.clone()).collect::<Vec<_>>(),
        [
            Op::Construct(0x06000001),
            Op::LoadField(0x04000002),
            Op::StoreField(0x0a000003)
        ]
    );
    for opcode in [0x73, 0x7b, 0x7d] {
        for len in 1..5 {
            assert!(
                decode(&[opcode, 1, 0, 0, 6][..len])
                    .unwrap_err()
                    .message
                    .contains("truncated")
            );
        }
        assert!(
            decode(&[opcode, 0, 0, 0, 0])
                .unwrap_err()
                .message
                .contains("token")
        );
    }
    assert!(decode(&[0x7d, 1, 0, 0, 6]).is_err());
    assert!(decode(&[0x73, 1, 0, 0, 4]).is_err());
}
