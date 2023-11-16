use regex::Regex;
use std::io::{self, BufRead, BufReader};

pub struct FunctionInfo {
    address: String,
    function_name: String,
    size: usize,
}
pub struct Instruction(usize, String); 

pub fn from_objdump_output(output_bytes: &[u8]) -> io::Result<Vec<Instruction>> {
    let mut instructions = Vec::new();
    let mut is_disassembly = false;
    let re = regex::Regex::new(r"^\s*(?P<address>[0-9a-f]+):\s*(?P<instruction>[0-9a-f]+\s+.+)").unwrap();

    let reader = BufReader::new(output_bytes);

    for line in reader.lines() {
        let line = line?;
        if line.contains("Disassembly of section") {
            is_disassembly = true;
            continue;
        }

        if is_disassembly && re.is_match(&line) {
            if let Some(captures) = re.captures(&line) {
                let address = usize::from_str_radix(&captures["address"], 16).unwrap();
                let instruction = captures["instruction"].trim().to_string();
                instructions.push((address, instruction));
            }
        }
    }

    Ok(instructions)
}

pub fn parse_objdump_output(output_bytes: &[u8]) -> io::Result<Vec<FunctionInfo>> {
    let mut function_info = Vec::new();
    let mut is_disassembly = false;
    let re = Regex::new(r"^\s*(?P<address>[0-9a-f]+):\s*(?P<instruction>[0-9a-f]+\s+.+)").unwrap();

    let reader = BufReader::new(output_bytes);
    let mut start_address: Option<usize> = None;

    for line in reader.lines() {
        let line = line?;
        if line.contains("Disassembly of section") {
            is_disassembly = true;
            start_address = None;
            continue;
        }

        if is_disassembly && re.is_match(&line) {
            if let Some(captures) = re.captures(&line) {
                let address = &captures["address"];
                let instruction = &captures["instruction"];

                let current_address = usize::from_str_radix(address, 16).unwrap();

                if start_address.is_none() {
                    start_address = Some(current_address);
                } else if let Some(start_address) = start_address.take() {
                    let size = current_address - start_address;
                    function_info.push(FunctionInfo {
                        address: address.to_string(),
                        function_name: instruction.to_string(),
                        size,
                    });
                }
            }
        }
    }
    Ok(function_info)
}
