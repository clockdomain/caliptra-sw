// Licensed under the Apache-2.0 license

use caliptra_builder::build_firmware_elf;
use caliptra_coverage::calculator;
use caliptra_coverage::collect_instr_pcs;
use caliptra_coverage::get_bitvec_paths;
use caliptra_coverage::CoverageMap;
use caliptra_coverage::CPTRA_COVERAGE_PATH;

use caliptra_builder::firmware::ROM_WITH_UART;
use caliptra_coverage::get_tag_from_fw_id;
use caliptra_coverage::uncovered_functions;
use caliptra_coverage::FunctionInfo;
use caliptra_coverage::Instruction;

pub fn report_uncovered_instructions_per_function(
    partially_covered_functions: &[FunctionInfo],
    instructions: &[Instruction],
) {
    for function in partially_covered_functions {
        let start_address = function.address;
        let end_address = start_address + function.size;

        // Filter instructions for the current function
        let uncovered_instructions = instructions
            .iter()
            .filter(|instruction| {
                instruction.address() >= start_address && instruction.address() < end_address
            })
            .collect::<Vec<_>>();
        // Print the report
        println!(
            "Function: {} (start: {}, size: {}), Uncovered Instructions: {:?}",
            function.function_name, start_address, function.size, uncovered_instructions
        );
    }
}

fn main() -> std::io::Result<()> {
    let cov_path = std::env::var(CPTRA_COVERAGE_PATH).unwrap_or_else(|_| "".into());
    if cov_path.is_empty() {
        return Ok(());
    }

    let paths = get_bitvec_paths(cov_path.as_str()).unwrap();
    if paths.is_empty() {
        println!("{} coverage files found", paths.len());
        return Ok(());
    }

    let tag = get_tag_from_fw_id(&ROM_WITH_UART).unwrap();

    println!("{} coverage files found", paths.len());
    let instr_pcs = collect_instr_pcs(&ROM_WITH_UART).unwrap();
    println!("ROM instruction count = {}", instr_pcs.len());

    let cv = CoverageMap::new(paths);
    let bv = cv
        .map
        .get(&tag)
        .expect("Coverage data  not found for image");

    let elf_bytes = build_firmware_elf(&ROM_WITH_UART)?;

    uncovered_functions(&elf_bytes, bv)?;

    println!(
        "Coverage for ROM_WITH_UART is {}%",
        (100 * calculator::coverage_from_bitmap(bv, &instr_pcs)) as f32 / instr_pcs.len() as f32
    );
    Ok(())
}
