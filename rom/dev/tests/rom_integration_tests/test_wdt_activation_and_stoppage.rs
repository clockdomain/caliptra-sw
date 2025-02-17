// Licensed under the Apache-2.0 license

use caliptra_builder::{
    firmware::FMC_WITH_UART,
    firmware::{self, APP_WITH_UART},
    ImageOptions,
};
use caliptra_common::RomBootStatus::{self, KatStarted};
use caliptra_hw_model::{DeviceLifecycle, HwModel, SecurityState};

#[test]
fn test_wdt_activation_and_stoppage() {
    let security_state = *SecurityState::default()
        .set_debug_locked(true)
        .set_device_lifecycle(DeviceLifecycle::Unprovisioned);

    // Build the image we are going to send to ROM to load
    let image_bundle = caliptra_builder::build_and_sign_image(
        &FMC_WITH_UART,
        &APP_WITH_UART,
        ImageOptions::default(),
    )
    .unwrap();

    let rom =
        caliptra_builder::build_firmware_rom(&caliptra_builder::firmware::ROM_WITH_UART).unwrap();
    let mut hw = caliptra_hw_model::new(caliptra_hw_model::BootParams {
        init_params: caliptra_hw_model::InitParams {
            rom: &rom,
            security_state,
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();

    if cfg!(feature = "fpga_realtime") {
        // timer1_restart is only high for a few cycles; the realtime model
        // timing is too imprecise that sort of check.
        hw.step_until(|m| m.ready_for_fw());
    } else {
        // Ensure we are starting to count from zero.
        hw.step_until(|m| m.soc_ifc().cptra_wdt_timer1_ctrl().read().timer1_restart());
    }

    // Make sure the wdt1 timer is enabled.
    assert!(hw.soc_ifc().cptra_wdt_timer1_en().read().timer1_en());

    // Upload the FW once ROM is at the right point
    hw.step_until(|m| m.soc_ifc().cptra_flow_status().read().ready_for_fw());
    hw.upload_firmware(&image_bundle.to_bytes().unwrap())
        .unwrap();

    // Keep going until we launch FMC
    hw.step_until_output_contains("[exit] Launching FMC")
        .unwrap();

    // Make sure the wdt1 timer is enabled.
    assert!(hw.soc_ifc().cptra_wdt_timer1_en().read().timer1_en());
}

#[test]
fn test_wdt_not_enabled_on_debug_part() {
    let security_state = *SecurityState::default()
        .set_debug_locked(false)
        .set_device_lifecycle(DeviceLifecycle::Unprovisioned);

    let rom = caliptra_builder::build_firmware_rom(&firmware::ROM_WITH_UART).unwrap();
    let mut hw = caliptra_hw_model::new(caliptra_hw_model::BootParams {
        init_params: caliptra_hw_model::InitParams {
            rom: &rom,
            security_state,
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();

    // Confirm security state is as expected.
    assert!(!hw.soc_ifc().cptra_security_state().read().debug_locked());

    hw.step_until_boot_status(RomBootStatus::CfiInitialized.into(), false);
    hw.step_until_boot_status(KatStarted.into(), false);

    // Make sure the wdt1 timer is disabled.
    assert!(!hw.soc_ifc().cptra_wdt_timer1_en().read().timer1_en());
}

#[test]
fn test_rom_wdt_timeout() {
    const WDT_EXPIRED: u32 = 0x0105000C;

    let security_state = *SecurityState::default()
        .set_debug_locked(true)
        .set_device_lifecycle(DeviceLifecycle::Unprovisioned);

    let rom = caliptra_builder::build_firmware_rom(&firmware::ROM_WITH_UART).unwrap();
    let mut hw = caliptra_hw_model::new(caliptra_hw_model::BootParams {
        init_params: caliptra_hw_model::InitParams {
            rom: &rom,
            security_state,
            wdt_timeout_cycles: 1_000_000,
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();

    hw.step_until(|m| m.soc_ifc().cptra_fw_error_fatal().read() == WDT_EXPIRED);
}
