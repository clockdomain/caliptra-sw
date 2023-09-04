// Licensed under the Apache-2.0 license
use caliptra_builder::{FwId, ImageOptions, FMC_WITH_UART, ROM_WITH_UART};
use caliptra_common::RomBootStatus;
use caliptra_drivers::{
    pcr_log::{PcrLogEntry, PcrLogEntryId},
    FirmwareHandoffTable, PcrId,
};
use caliptra_hw_model::{BootParams, HwModel, InitParams};

use zerocopy::{AsBytes, FromBytes};

const TEST_CMD_READ_PCR_LOG: u32 = 0x1000_0000;
const TEST_CMD_READ_FHT: u32 = 0x1000_0001;
const TEST_CMD_TRIGGER_UPDATE_RESET: u32 = 0x1000_0002;
const TEST_CMD_READ_PCRS: u32 = 0x1000_0003;
const TEST_CMD_TRY_TO_RESET_PCRS: u32 = 0x1000_0004;

const RT_ALIAS_MEASUREMENT_COMPLETE: u32 = 0x400;
const RT_ALIAS_DERIVED_CDI_COMPLETE: u32 = 0x401;
const RT_ALIAS_KEY_PAIR_DERIVATION_COMPLETE: u32 = 0x402;
const RT_ALIAS_SUBJ_ID_SN_GENERATION_COMPLETE: u32 = 0x403;
const RT_ALIAS_SUBJ_KEY_ID_GENERATION_COMPLETE: u32 = 0x404;
const RT_ALIAS_CERT_SIG_GENERATION_COMPLETE: u32 = 0x405;
const RT_ALIAS_DERIVATION_COMPLETE: u32 = 0x406;

const PCR_COUNT: usize = 32;
const PCR_ENTRY_SIZE: usize = core::mem::size_of::<PcrLogEntry>();

#[test]
fn test_boot_status_reporting() {
    let rom = caliptra_builder::build_firmware_rom(&ROM_WITH_UART).unwrap();

    pub const MOCK_RT_WITH_UART: FwId = FwId {
        crate_name: "caliptra-fmc-mock-rt",
        bin_name: "caliptra-fmc-mock-rt",
        features: &["emu"],
        workspace_dir: None,
    };

    let image = caliptra_builder::build_and_sign_image(
        &FMC_WITH_UART,
        &MOCK_RT_WITH_UART,
        ImageOptions::default(),
    )
    .unwrap();

    let mut hw = caliptra_hw_model::new(BootParams {
        init_params: InitParams {
            rom: &rom,
            ..Default::default()
        },
        fw_image: Some(&image.to_bytes().unwrap()),
        ..Default::default()
    })
    .unwrap();

    hw.step_until_boot_status(RT_ALIAS_MEASUREMENT_COMPLETE, true);
    hw.step_until_boot_status(RT_ALIAS_DERIVED_CDI_COMPLETE, true);
    hw.step_until_boot_status(RT_ALIAS_KEY_PAIR_DERIVATION_COMPLETE, true);
    hw.step_until_boot_status(RT_ALIAS_SUBJ_ID_SN_GENERATION_COMPLETE, true);
    hw.step_until_boot_status(RT_ALIAS_SUBJ_KEY_ID_GENERATION_COMPLETE, true);
    hw.step_until_boot_status(RT_ALIAS_CERT_SIG_GENERATION_COMPLETE, true);
    hw.step_until_boot_status(RT_ALIAS_DERIVATION_COMPLETE, true);
}

#[test]
fn test_fht_info() {
    pub const MOCK_RT_WITH_UART: FwId = FwId {
        crate_name: "caliptra-fmc-mock-rt",
        bin_name: "caliptra-fmc-mock-rt",
        features: &["emu", "interactive_test_fmc"],
        workspace_dir: None,
    };
    let rom = caliptra_builder::build_firmware_rom(&ROM_WITH_UART).unwrap();
    let image = caliptra_builder::build_and_sign_image(
        &FMC_WITH_UART,
        &MOCK_RT_WITH_UART,
        ImageOptions::default(),
    )
    .unwrap();

    let mut hw = caliptra_hw_model::new(BootParams {
        init_params: InitParams {
            rom: &rom,
            ..Default::default()
        },
        fw_image: Some(&image.to_bytes().unwrap()),
        ..Default::default()
    })
    .unwrap();

    let result = hw.mailbox_execute(0x1000_0001, &[]);
    assert!(result.is_ok());

    let data = result.unwrap().unwrap();
    let fht = FirmwareHandoffTable::read_from_prefix(data.as_bytes()).unwrap();
    assert_eq!(fht.ldevid_tbs_size, 533);
    assert_eq!(fht.fmcalias_tbs_size, 745);
    assert_eq!(fht.ldevid_tbs_addr, 0x50003800);
    assert_eq!(fht.fmcalias_tbs_addr, 0x50003C00);
    assert_eq!(fht.pcr_log_addr, 0x50004400);
    assert_eq!(fht.fuse_log_addr, 0x50004800);
}

#[test]
fn test_pcr_log() {
    pub const MOCK_RT_WITH_UART: FwId = FwId {
        crate_name: "caliptra-fmc-mock-rt",
        bin_name: "caliptra-fmc-mock-rt",
        features: &["emu", "interactive_test_fmc"],
        workspace_dir: None,
    };
    let rom = caliptra_builder::build_firmware_rom(&ROM_WITH_UART).unwrap();
    let image = caliptra_builder::build_and_sign_image(
        &FMC_WITH_UART,
        &MOCK_RT_WITH_UART,
        ImageOptions::default(),
    )
    .unwrap();

    let mut hw = caliptra_hw_model::new(BootParams {
        init_params: InitParams {
            rom: &rom,
            ..Default::default()
        },
        fw_image: Some(&image.to_bytes().unwrap()),
        ..Default::default()
    })
    .unwrap();

    let result = hw.mailbox_execute(0x1000_0001, &[]);
    assert!(result.is_ok());

    let data = result.unwrap().unwrap();
    let fht = FirmwareHandoffTable::read_from_prefix(data.as_bytes()).unwrap();

    let pcr_entry_arr = hw.mailbox_execute(0x1000_0000, &[]).unwrap().unwrap();

    // Check PCR entry for RtTci.
    let mut pcr_log_entry_offset = (fht.pcr_log_index as usize) * PCR_ENTRY_SIZE;

    let pcr_log_entry =
        PcrLogEntry::read_from_prefix(pcr_entry_arr[pcr_log_entry_offset..].as_bytes()).unwrap();
    assert_eq!(pcr_log_entry.id, PcrLogEntryId::RtTci as u16);
    assert_eq!(
        pcr_log_entry.pcr_ids,
        1 << (caliptra_common::RT_FW_CURRENT_PCR as u8)
    );

    // Check PCR entry for Manifest digest.
    pcr_log_entry_offset += core::mem::size_of::<PcrLogEntry>();
    let pcr_log_entry =
        PcrLogEntry::read_from_prefix(pcr_entry_arr[pcr_log_entry_offset..].as_bytes()).unwrap();
    assert_eq!(pcr_log_entry.id, PcrLogEntryId::FwImageManifest as u16);
    assert_eq!(
        pcr_log_entry.pcr_ids,
        1 << (caliptra_common::RT_FW_CURRENT_PCR as u8)
    );

    // Check PCR entry for RtTci.
    pcr_log_entry_offset += core::mem::size_of::<PcrLogEntry>();

    let pcr_log_entry =
        PcrLogEntry::read_from_prefix(pcr_entry_arr[pcr_log_entry_offset..].as_bytes()).unwrap();
    assert_eq!(pcr_log_entry.id, PcrLogEntryId::RtTci as u16);
    assert_eq!(
        pcr_log_entry.pcr_ids,
        1 << (caliptra_common::RT_FW_JOURNEY_PCR as u8)
    );

    // Check PCR entry for Manifest digest.
    pcr_log_entry_offset += PCR_ENTRY_SIZE;
    let pcr_log_entry =
        PcrLogEntry::read_from_prefix(pcr_entry_arr[pcr_log_entry_offset..].as_bytes()).unwrap();
    assert_eq!(pcr_log_entry.id, PcrLogEntryId::FwImageManifest as u16);
    assert_eq!(
        pcr_log_entry.pcr_ids,
        1 << (caliptra_common::RT_FW_JOURNEY_PCR as u8)
    );

    hw.soc_ifc()
        .internal_fw_update_reset()
        .write(|w| w.core_rst(true));

    assert!(hw.upload_firmware(&image.to_bytes().unwrap()).is_ok());

    hw.step_until_boot_status(RT_ALIAS_DERIVATION_COMPLETE, true);

    let pcr_entry_arr = hw.mailbox_execute(0x1000_0000, &[]).unwrap().unwrap();

    // Check PCR entry for RtTci.
    let mut pcr_log_entry_offset =
        (fht.pcr_log_index as usize) * core::mem::size_of::<PcrLogEntry>();

    let pcr_log_entry =
        PcrLogEntry::read_from_prefix(pcr_entry_arr[pcr_log_entry_offset..].as_bytes()).unwrap();
    assert_eq!(pcr_log_entry.id, PcrLogEntryId::RtTci as u16);
    assert_eq!(
        pcr_log_entry.pcr_ids,
        1 << (caliptra_common::RT_FW_CURRENT_PCR as u8)
    );

    // Check PCR entry for Manifest digest.
    pcr_log_entry_offset += PCR_ENTRY_SIZE;
    let pcr_log_entry =
        PcrLogEntry::read_from_prefix(pcr_entry_arr[pcr_log_entry_offset..].as_bytes()).unwrap();
    assert_eq!(pcr_log_entry.id, PcrLogEntryId::FwImageManifest as u16);
    assert_eq!(
        pcr_log_entry.pcr_ids,
        1 << (caliptra_common::RT_FW_CURRENT_PCR as u8)
    );

    // Check PCR entry for RtTci.
    pcr_log_entry_offset += core::mem::size_of::<PcrLogEntry>();

    let pcr_log_entry =
        PcrLogEntry::read_from_prefix(pcr_entry_arr[pcr_log_entry_offset..].as_bytes()).unwrap();
    assert_eq!(pcr_log_entry.id, PcrLogEntryId::RtTci as u16);
    assert_eq!(
        pcr_log_entry.pcr_ids,
        1 << (caliptra_common::RT_FW_JOURNEY_PCR as u8)
    );

    // Check PCR entry for Manifest digest.
    pcr_log_entry_offset += PCR_ENTRY_SIZE;
    let pcr_log_entry =
        PcrLogEntry::read_from_prefix(pcr_entry_arr[pcr_log_entry_offset..].as_bytes()).unwrap();
    assert_eq!(pcr_log_entry.id, PcrLogEntryId::FwImageManifest as u16);
    assert_eq!(
        pcr_log_entry.pcr_ids,
        1 << (caliptra_common::RT_FW_JOURNEY_PCR as u8)
    );
}
