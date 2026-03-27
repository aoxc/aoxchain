// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::ContractError;

pub trait Validate {
    fn validate(&self) -> Result<(), ContractError>;
}
