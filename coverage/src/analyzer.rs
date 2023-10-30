use caliptra_builder::{build_firmware_elf, FwId};
use goblin::elf::{Elf, Sym};

pub struct FunctionInfo {
    pub name: String,
    pc_start: usize,
    pc_end: usize,
}

impl FunctionInfo {
    // Helper method to get the inclusive PC range of a function.
    pub fn pc_range(&self) -> std::ops::Range<usize> {
        self.pc_start..self.pc_end
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
pub fn extract_function_info(
    id: &FwId<'static>,
) -> Result<Vec<FunctionInfo>, Box<dyn std::error::Error>> {
    let elf_bytes = build_firmware_elf(id)?;

    let mut function_info_vec: Vec<FunctionInfo> = Vec::new();

    // Parse the ELF file.
    let elf = Elf::parse(&elf_bytes)?;

    // Find the section index of the ".text" section.
    let mut text_section_index: Option<usize> = None;
    for (index, section) in elf.section_headers.iter().enumerate() {
        if let Some(section_name) = elf.strtab.get_at(section.sh_name) {
            if section_name == ".text" {
                text_section_index = Some(index);
                break;
            }
        }
    }

    match text_section_index {
        Some(index) => {
            println!("Found the .text section at index: {}", index);

            // Iterate through the symbols to find functions in the text section.
            for sym in &elf.syms {
                let Sym {
                    st_shndx,
                    st_value,
                    st_size,
                    st_name,
                    ..
                } = sym;

                // Check if the symbol is in the text section.
                if Some(st_shndx as usize) == text_section_index {
                    // Calculate the range of PCs for the function.
                    let pc_start = st_value as usize;
                    let pc_end = pc_start + st_size as usize;

                    // Get the function name.
                    if let Some(func_name) = elf.strtab.get_at(st_name as usize) {
                        // Create a FunctionInfo struct and add it to the vector.
                        let func_info = FunctionInfo {
                            name: func_name.to_string(),
                            pc_start,
                            pc_end,
                        };
                        function_info_vec.push(func_info);
                        // Print the function name and its PC range.
                        println!(
                            "Function: {} (PC Range: 0x{:x}-0x{:x})",
                            func_name, pc_start, pc_end
                        );
                    }
                }
            }
        }
        None => println!("The .text section was not found in the ELF file."),
    }
    Ok(function_info_vec)
}
