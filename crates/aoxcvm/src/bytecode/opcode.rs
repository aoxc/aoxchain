//! Canonical opcode definitions for the phase-3 execution core prototype.

/// Canonical opcode class used by verifier and metering policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeClass {
    Arithmetic,
    ControlFlow,
    System,
}

/// Canonical execution opcodes.
///
/// Design properties:
/// - stable wire assignment via `repr(u8)`,
/// - explicit reserved range handling in decoder (`0x80..=0xff`),
/// - fail-closed unknown opcode rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Nop = 0x00,
    PushI64 = 0x01,
    Add = 0x02,
    Sub = 0x03,
    Mul = 0x04,
    Div = 0x05,
    Mod = 0x06,
    Halt = 0x07,
    Revert = 0x08,
}

impl Opcode {
    /// Decodes a canonical opcode from wire value.
    pub const fn from_byte(byte: u8) -> Result<Self, OpcodeDecodeError> {
        match byte {
            0x00 => Ok(Self::Nop),
            0x01 => Ok(Self::PushI64),
            0x02 => Ok(Self::Add),
            0x03 => Ok(Self::Sub),
            0x04 => Ok(Self::Mul),
            0x05 => Ok(Self::Div),
            0x06 => Ok(Self::Mod),
            0x07 => Ok(Self::Halt),
            0x08 => Ok(Self::Revert),
            0x80..=0xff => Err(OpcodeDecodeError::Reserved(byte)),
            _ => Err(OpcodeDecodeError::Unknown(byte)),
        }
    }

    /// Encodes a canonical opcode to wire value.
    pub const fn to_byte(self) -> u8 {
        self as u8
    }

    /// Returns opcode class.
    pub const fn class(self) -> OpcodeClass {
        match self {
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod => OpcodeClass::Arithmetic,
            Self::Halt | Self::Revert => OpcodeClass::ControlFlow,
            Self::Nop | Self::PushI64 => OpcodeClass::System,
        }
    }

    /// Deterministic base gas cost used by executor.
    pub const fn base_gas(self) -> u64 {
        match self {
            Self::Nop => 1,
            Self::PushI64 => 2,
            Self::Add | Self::Sub => 3,
            Self::Mul => 5,
            Self::Div | Self::Mod => 8,
            Self::Halt | Self::Revert => 0,
        }
    }
}

/// Canonical opcode decoding errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeDecodeError {
    Unknown(u8),
    Reserved(u8),
}

#[cfg(test)]
mod tests {
    use super::{Opcode, OpcodeClass, OpcodeDecodeError};

    #[test]
    fn opcode_roundtrip_is_stable() {
        for opcode in [
            Opcode::Nop,
            Opcode::PushI64,
            Opcode::Add,
            Opcode::Sub,
            Opcode::Mul,
            Opcode::Div,
            Opcode::Mod,
            Opcode::Halt,
            Opcode::Revert,
        ] {
            assert_eq!(Opcode::from_byte(opcode.to_byte()), Ok(opcode));
        }
    }

    #[test]
    fn reserved_and_unknown_are_fail_closed() {
        assert_eq!(
            Opcode::from_byte(0x80),
            Err(OpcodeDecodeError::Reserved(0x80))
        );
        assert_eq!(
            Opcode::from_byte(0x7f),
            Err(OpcodeDecodeError::Unknown(0x7f))
        );
    }

    #[test]
    fn class_mapping_is_explicit() {
        assert_eq!(Opcode::Add.class(), OpcodeClass::Arithmetic);
        assert_eq!(Opcode::Halt.class(), OpcodeClass::ControlFlow);
        assert_eq!(Opcode::Nop.class(), OpcodeClass::System);
    }
}
