use std::str::FromStr;

use ic_web3_rs::types::H160;

use anyhow::Result;

use crate::errors::UtilsError;

#[inline]
pub fn from_h160(h160: &H160) -> String {
    format!("0x{}", hex::encode(h160.as_bytes()))
}

#[inline]
pub fn to_h160(address: &str) -> Result<H160, UtilsError> {
    H160::from_str(address).map_err(|err| UtilsError::InvalidAddressFormat(err.to_string()))
}

#[inline]
pub fn normalize(address: &str) -> Result<String, UtilsError> {
    let h160 = to_h160(address)?;
    Ok(from_h160(&h160))
}
