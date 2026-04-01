use crate::state::overlay::StateOverlay;

pub fn rollback(overlay: &mut StateOverlay) {
    overlay.pending.writes.clear();
    overlay.pending.deletes.clear();
}
