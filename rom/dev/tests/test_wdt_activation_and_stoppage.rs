use caliptra_builder::ImageOptions;
use caliptra_common::RomBootStatus::KatStarted;
use caliptra_hw_model::{Fuses, HwModel};

pub mod helpers;

#[test]
fn test_wdt_activation_and_stoppage() {
    let (mut hw, image_bundle) =
        helpers::build_hw_model_and_image_bundle(Fuses::default(), ImageOptions::default());

    hw.step_until_boot_status(KatStarted.into(), false);

    // Make sure the wdt1 timer is enabled.
    assert_eq!(!hw.soc_ifc().cptra_wdt_timer1_en().read().timer1_en(), true);

    // Upload the FW once ROM is at the right point
    hw.step_until(|m| m.soc_ifc().cptra_flow_status().read().ready_for_fw());
    hw.upload_firmware(&image_bundle.to_bytes().unwrap())
        .unwrap();

    // Keep going until we launch FMC
    hw.step_until_output_contains("[exit] Launching FMC")
        .unwrap();

    // Make sure the wdt1 timer is disabled.
    assert_eq!(hw.soc_ifc().cptra_wdt_timer1_en().read().timer1_en(), false);
}
