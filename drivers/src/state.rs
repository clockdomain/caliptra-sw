/*++

Licensed under the Apache-2.0 license.

File Name:

    state.rs

Abstract:

    File contains Device state related API

--*/

use caliptra_registers::soc_ifc;
use caliptra_registers::soc_ifc::enums::DeviceLifecycleE;

/// Device Life Cycle State
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Lifecycle {
    /// Unprovisioned
    Unprovisioned = 0x0,

    /// Manufacturing
    Manufacturing = 0x1,

    /// Production
    Production = 0x2,

    /// Unknown
    Unknown = 0x3,
}

impl From<DeviceLifecycleE> for Lifecycle {
    /// Converts to this type from the input type.
    fn from(value: DeviceLifecycleE) -> Self {
        match value {
            DeviceLifecycleE::DeviceUnprovisioned => Lifecycle::Unprovisioned,
            DeviceLifecycleE::DeviceManufacturing => Lifecycle::Manufacturing,
            DeviceLifecycleE::DeviceProduction => Lifecycle::Production,
            DeviceLifecycleE::Reserved2 => Lifecycle::Unknown,
        }
    }
}

/// Device State
#[derive(Default, Debug)]
pub struct DeviceState {}

impl DeviceState {
    /// Retrieve the device lifecycle state
    pub fn lifecycle(&self) -> Lifecycle {
        let soc_ifc_regs = soc_ifc::RegisterBlock::soc_ifc_reg();
        soc_ifc_regs
            .cptra_security_state()
            .read()
            .device_lifecycle()
            .into()
    }

    /// Check if device is locked for debug
    pub fn debug_locked(&self) -> bool {
        let soc_ifc_regs = caliptra_registers::soc_ifc::RegisterBlock::soc_ifc_reg();
        soc_ifc_regs.cptra_security_state().read().debug_locked()
    }
}
