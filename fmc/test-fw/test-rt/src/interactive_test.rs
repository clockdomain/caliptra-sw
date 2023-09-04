// Licensed under the Apache-2.0 license

use caliptra_drivers::memory_layout::{FHT_ORG, PCR_LOG_ORG};
use caliptra_drivers::{cprintln, FirmwareHandoffTable};
use caliptra_drivers::{
    pcr_log::{PcrLogEntry, PcrLogEntryId},
    PcrBank, PcrId,
};
use caliptra_registers::pv::PvReg;
use ureg::RealMmioMut;

use core::convert::TryFrom;
use core::convert::TryInto;
use zerocopy::{AsBytes, FromBytes};
fn process_mailbox_command(mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>) {
    let cmd = mbox.cmd().read();
    cprintln!("[fmc-test-harness] Received command: 0x{:08X}", cmd);
    match cmd {
        0x1000_0000 => {
            read_pcr_log(mbox);
        }
        0x1000_0001 => {
            read_fht(mbox);
        }
        0x1000_0002 => {
            trigger_update_reset(mbox);
        }
        0x1000_0003 => {
            read_pcrs(mbox);
        }
        0x1000_0004 => {
            try_to_reset_pcrs(mbox);
        }
        _ => {}
    }
}

pub fn process_mailbox_commands() {
    let mut mbox = unsafe { caliptra_registers::mbox::MboxCsr::new() };
    let mbox = mbox.regs_mut();

    loop {
        if mbox.status().read().mbox_fsm_ps().mbox_execute_uc() {
            process_mailbox_command(&mbox);
        }
    }

    #[cfg(not(feature = "interactive_test_fmc"))]
    process_mailbox_command(&mbox);
}

fn swap_word_bytes_inplace(words: &mut [u32]) {
    for word in words.iter_mut() {
        *word = word.swap_bytes()
    }
}

fn read_pcrs(mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>) {
    let pcr_bank = unsafe { PcrBank::new(PvReg::new()) };
    const PCR_COUNT: usize = 32;
    for i in 0..PCR_COUNT {
        let pcr = pcr_bank.read_pcr(PcrId::try_from(i as u8).unwrap());
        let mut pcr_bytes: [u32; 12] = pcr.try_into().unwrap();

        swap_word_bytes_inplace(&mut pcr_bytes);
        send_to_mailbox(mbox, pcr.as_bytes(), false);
    }

    mbox.dlen().write(|_| (48 * PCR_COUNT).try_into().unwrap());
    mbox.status().write(|w| w.status(|w| w.data_ready()));
}

fn read_fht(mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>) {
    // Copy the FHT from DCCM
    let mut fht: [u8; core::mem::size_of::<FirmwareHandoffTable>()] =
        [0u8; core::mem::size_of::<FirmwareHandoffTable>()];

    let src = unsafe {
        let ptr = FHT_ORG as *mut u8;
        core::slice::from_raw_parts_mut(ptr, core::mem::size_of::<FirmwareHandoffTable>())
    };

    fht.copy_from_slice(src);

    send_to_mailbox(mbox, fht.as_bytes(), true);
}

fn send_to_mailbox(
    mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>,
    data: &[u8],
    update_mb_state: bool,
) {
    let data_len = data.len();
    let word_size = core::mem::size_of::<u32>();
    let remainder = data_len % word_size;
    let n = data_len - remainder;
    for idx in (0..n).step_by(word_size) {
        mbox.datain()
            .write(|_| u32::from_le_bytes(data[idx..idx + word_size].try_into().unwrap()));
    }

    if remainder > 0 {
        let mut last_word = data[n] as u32;
        for idx in 1..remainder {
            last_word |= (data[n + idx] as u32) << (idx << 3);
        }
        mbox.datain().write(|_| last_word);
    }

    if update_mb_state {
        mbox.dlen().write(|_| data_len as u32);
        mbox.status().write(|w| w.status(|w| w.data_ready()));
    }
}

// Returns a list of u8 values, 0 on success, 1 on failure:
//   - Whether PCR0 is locked
//   - Whether PCR1 is locked
//   - Whether PCR2 is unlocked
//   - Whether PCR3 is unlocked
fn try_to_reset_pcrs(mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>) {
    let mut pcr_bank = unsafe { PcrBank::new(PvReg::new()) };

    let res0 = pcr_bank.erase_pcr(PcrId::PcrId0);
    let res1 = pcr_bank.erase_pcr(PcrId::PcrId1);
    let res2 = pcr_bank.erase_pcr(PcrId::PcrId2);
    let res3 = pcr_bank.erase_pcr(PcrId::PcrId3);

    let ret_vals: [u8; 4] = [
        if res0.is_err() { 0 } else { 1 },
        if res1.is_err() { 0 } else { 1 },
        if res2.is_ok() { 0 } else { 1 },
        if res3.is_ok() { 0 } else { 1 },
    ];

    send_to_mailbox(mbox, &ret_vals, false);
    mbox.dlen().write(|_| ret_vals.len().try_into().unwrap());
    mbox.status().write(|w| w.status(|w| w.data_ready()));
}

fn trigger_update_reset(mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>) {
    mbox.status().write(|w| w.status(|w| w.cmd_complete()));
    const STDOUT: *mut u32 = 0x3003_0624 as *mut u32;
    unsafe {
        core::ptr::write_volatile(STDOUT, 1_u32);
    }
}

fn read_pcr_log(mbox: &caliptra_registers::mbox::RegisterBlock<RealMmioMut>) {
    let mut pcr_entry_count = 0;
    loop {
        let pcr_entry = get_pcr_entry(pcr_entry_count);
        if PcrLogEntryId::from(pcr_entry.id) == PcrLogEntryId::Invalid {
            break;
        }

        pcr_entry_count += 1;
        send_to_mailbox(mbox, pcr_entry.as_bytes(), false);
    }

    mbox.dlen().write(|_| {
        (core::mem::size_of::<PcrLogEntry>() * pcr_entry_count)
            .try_into()
            .unwrap()
    });
    mbox.status().write(|w| w.status(|w| w.data_ready()));
}

fn get_pcr_entry(entry_index: usize) -> PcrLogEntry {
    // Copy the pcr log entry from DCCM
    let mut pcr_entry: [u8; core::mem::size_of::<PcrLogEntry>()] =
        [0u8; core::mem::size_of::<PcrLogEntry>()];

    let src = unsafe {
        let offset = core::mem::size_of::<PcrLogEntry>() * entry_index;
        let ptr = (PCR_LOG_ORG as *mut u8).add(offset);
        core::slice::from_raw_parts_mut(ptr, core::mem::size_of::<PcrLogEntry>())
    };

    pcr_entry.copy_from_slice(src);
    PcrLogEntry::read_from_prefix(pcr_entry.as_bytes()).unwrap()
}
