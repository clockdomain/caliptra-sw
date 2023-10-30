// Licensed under the Apache-2.0 license

use caliptra_coverage::{
    calculator, collect_instr_pcs, get_bitvec_paths, uncovered_function_names, CoverageMap,
};

use caliptra_builder::firmware::ROM_WITH_UART;
use caliptra_coverage::get_tag_from_fw_id;

fn main() {
    let cov_path = std::env::var("CPTRA_COVERAGE_PATH").unwrap_or_else(|_| "".into());
    if cov_path.is_empty() {
        return;
    }

    let paths = get_bitvec_paths(cov_path.as_str()).unwrap();
    let tag = get_tag_from_fw_id(&ROM_WITH_UART);

    let info = caliptra_coverage::extract_function_info(&ROM_WITH_UART).unwrap();

    println!("{} coverage files found", paths.len());
    let instr_pcs = collect_instr_pcs(&ROM_WITH_UART).unwrap();
    println!("ROM instruction count = {}", instr_pcs.len());

    let cv = CoverageMap::new(paths);
    let bv = cv
        .map
        .get(&tag)
        .expect("Coverage data  not found for image");

    uncovered_function_names(info, &bv);

    println!(
        "Coverage for ROM_WITH_UART is {}%",
        (100 * calculator::coverage_from_bitmap(bv, &instr_pcs)) as f32 / instr_pcs.len() as f32
    );
}
