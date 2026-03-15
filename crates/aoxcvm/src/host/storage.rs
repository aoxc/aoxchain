use crate::vm_kind::VmKind;

/// Builds a deterministic host storage key.
///
/// The storage layout is intentionally explicit:
/// `<vm-prefix>/<namespace>/<key>`.
pub fn compose_storage_key(vm: VmKind, namespace: &[u8], key: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(vm.as_prefix().len() + namespace.len() + key.len() + 2);
    out.extend_from_slice(vm.as_prefix());
    out.push(b'/');
    out.extend_from_slice(namespace);
    out.push(b'/');
    out.extend_from_slice(key);
    out
}
