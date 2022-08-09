use std::convert::TryInto;

use esp_idf_sys as _;

use rand_core::{CryptoRng, Error, RngCore};
pub struct HRNG;

impl CryptoRng for HRNG {}
impl RngCore for HRNG {
    fn next_u32(&mut self) -> u32 {
        unsafe { esp_idf_sys::esp_random() }
    }

    fn next_u64(&mut self) -> u64 {
        unsafe { esp_idf_sys::esp_random().try_into().unwrap() }
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        unsafe {
            esp_idf_sys::esp_fill_random(
                dest.as_ptr() as *mut core::ffi::c_void,
                dest.len().try_into().unwrap(),
            );
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        Ok(self.fill_bytes(dest))
    }
}
