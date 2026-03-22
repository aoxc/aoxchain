use crate::error::ContractError;

pub trait Validate {
    fn validate(&self) -> Result<(), ContractError>;
}
