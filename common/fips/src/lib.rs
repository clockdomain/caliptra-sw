// Licensed under the Apache-2.0 license
#![no_std]

use caliptra_common::CommandHandler;
use caliptra_error::{CaliptraError, CaliptraResult};

#[derive(PartialEq, Eq)]
pub struct FipsModuleApi(pub u32);

impl FipsModuleApi {
    /// The status command.
    pub const STATUS: Self = Self(0x53544154); // "STAT"
    /// The self-test command.
    pub const SELF_TEST: Self = Self(0x53454C46); // "SELF"
    /// The shutdown command.
    pub const SHUTDOWN: Self = Self(0x5348444E); // "SHDN"
}
impl From<u32> for FipsModuleApi {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<FipsModuleApi> for u32 {
    fn from(value: FipsModuleApi) -> Self {
        value.0
    }
}

pub struct FipsManagement;

impl FipsManagement {
    pub fn status(&self) -> CaliptraResult<()> {
        Ok(())
    }
    pub fn self_test(&self) -> CaliptraResult<()> {
        Ok(())
    }
    pub fn shutdown(&self) -> CaliptraResult<()> {
        Ok(())
    }
}

impl CommandHandler for FipsManagement {
    fn handle_command(&self, command_id: u32) -> CaliptraResult<()> {
        match FipsModuleApi::from(command_id) {
            FipsModuleApi::STATUS => self.status(),
            FipsModuleApi::SELF_TEST => self.self_test(),
            FipsModuleApi::SHUTDOWN => self.shutdown(),
            _ => Err(CaliptraError::FIPS_COMMAND_NOT_IMPLEMENTED),
        }
    }
}
