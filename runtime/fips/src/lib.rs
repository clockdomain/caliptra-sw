#![no_std]
use core::convert::TryFrom;
use caliptra_error::CaliptraResult;
use caliptra_error::CaliptraError;

use caliptra_registers::mbox::enums::MboxStatusE;
use caliptra_drivers::KeyVault;
/// ROM Verification Environemnt
pub struct FipsEnv<'a> {
    pub key_vault: &'a mut KeyVault,
}

#[derive(PartialEq, Eq)]
pub struct FipsModuleApi(pub u32);

impl FipsModuleApi {
    /// The status command.
    pub const STATUS: Self = Self(0x5354_4154); // "STAT"
    /// The self-test command.
    pub const SELF_TEST: Self = Self(0x5345_4C46); // "SELF"
    /// The shutdown command.
    pub const SHUTDOWN: Self = Self(0x5348_444E); // "SHDN"
}

impl TryFrom<u32> for FipsModuleApi {
    type Error = CaliptraError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x5354_4154 => Ok(Self(0x5354_4154)), // "STAT"
            0x5345_4C46 => Ok(Self(0x5345_4C46)), // "SELF"
            0x5348_444E => Ok(Self(0x5348_444E)), // "SHDN"
            _ => Err(CaliptraError::FIPS_COMMAND_NOT_IMPLEMENTED),
        }
    }
}

pub trait FipsManagement {
    fn status(&self, fips_env: &FipsEnv) -> CaliptraResult<MboxStatusE>;
    fn self_test(&self, fips_env: &FipsEnv) -> CaliptraResult<MboxStatusE>;
    fn shutdown(&self, fips_env: &FipsEnv) -> CaliptraResult<MboxStatusE>;
}
