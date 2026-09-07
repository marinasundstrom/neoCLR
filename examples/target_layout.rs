//! Compare explicit data layouts without changing interpreter execution.
use neoclr::{LoadedProgram, assemble, assembler::parse_type, memory::TargetLayout};

fn main() -> Result<(), neoclr::Fault> {
    let module = assemble(
        ".module App\n.type Packet\n.field Tag Byte\n.field Link Packet*\n.field Count UInt64\n.end",
    )?;
    let program = LoadedProgram::new(&module)?;
    for (label, target) in [
        (
            "32-bit, align 4",
            TargetLayout {
                pointer_size: 4,
                pointer_alignment: 4,
                int64_alignment: 4,
                single_alignment: 4,
                double_alignment: 4,
            },
        ),
        (
            "64-bit, align 8",
            TargetLayout {
                pointer_size: 8,
                pointer_alignment: 8,
                int64_alignment: 8,
                single_alignment: 4,
                double_alignment: 8,
            },
        ),
    ] {
        let layout = program.layout_of(&parse_type("Packet")?, target)?;
        println!(
            "{label}: size {}, alignment {}, offsets {:?}",
            layout.size,
            layout.alignment,
            layout.fields.iter().map(|f| f.offset).collect::<Vec<_>>()
        );
    }
    Ok(())
}
