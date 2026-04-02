//! bytecode subsystem for AOXC-VMachine-QX1 Kernel v1.
pub mod canonicalizer;
pub mod debug;
pub mod feature_gates;
pub mod format;
pub mod hashing;
pub mod header;
pub mod immediate;
pub mod linker;
pub mod loader;
pub mod module;
pub mod opcode;
pub mod package;
pub mod section;
pub mod validator;
pub mod verifier;
pub mod versioning;
