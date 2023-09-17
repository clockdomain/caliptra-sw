// Licensed under the Apache-2.0 license

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn parse_trace_file(trace_file_path: &str) -> HashSet<u32> {
    let mut unique_pcs = HashSet::new();

    // Open the trace file
    if let Ok(file) = File::open(trace_file_path) {
        let reader = BufReader::new(file);

        // Iterate through each line in the trace file
        for line in reader.lines() {
            if let Ok(line) = line {
                // Check if the line starts with "pc="
                if line.starts_with("pc=") {
                    // Extract the PC by splitting the line at '=' and parsing the hexadecimal value
                    if let Some(pc_str) = line.strip_prefix("pc=") {
                        match u32::from_str_radix(pc_str.trim_start_matches("0x"), 16) {
                            Ok(pc) => {
                                unique_pcs.insert(pc);
                            }
                            Err(_) => (),
                        }
                    }
                }
            }
        }
    }

    unique_pcs
}

#[test]
fn test_parse_trace_file() {
    // Create a temporary trace file for testing
    let temp_trace_file = "temp_trace.txt";
    let trace_data = vec![
        "SoC write4 *0x300300bc <- 0x0",
        "SoC write4 *0x30030110 <- 0x2625a00",
        "SoC write4 *0x30030114 <- 0x0",
        "SoC write4 *0x300300b8 <- 0x1",
        "pc=0x0",
        "pc=0x4",
        "pc=0x4",
        "pc=0x4",
        "pc=0x0",
    ];

    // Write the test data to the temporary trace file
    std::fs::write(temp_trace_file, trace_data.join("\n"))
        .expect("Failed to write test trace file");

    // Call the function to parse the test trace file
    let unique_pcs = parse_trace_file(temp_trace_file);

    // Define the expected unique PCs based on the test data
    let expected_pcs: HashSet<u32> = vec![0x0, 0x4].into_iter().collect();

    // Assert that the result matches the expected unique PCs
    assert_eq!(unique_pcs, expected_pcs);

    // Clean up: remove the temporary trace file
    std::fs::remove_file(temp_trace_file).expect("Failed to remove test trace file");
}
