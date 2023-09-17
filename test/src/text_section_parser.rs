pub fn get_instr_pcs(text_section: &[u8]) -> Vec<u32> {
    let mut index = 0_usize;
    let mut instr_count = 0_usize;

    let mut instr_pcs = Vec::<u32>::new();

    while index < text_section.len() {
        let instruction = &text_section[index..index + 2];
        let instruction = u16::from_le_bytes([instruction[0], instruction[1]]);

        match instruction & 0b11 {
            0 | 1 | 2 => {
                index += 2;
            }
            _ => {
                index += 4;
            }
        }
        instr_pcs.push(index as u32);
    }
    instr_pcs
}
