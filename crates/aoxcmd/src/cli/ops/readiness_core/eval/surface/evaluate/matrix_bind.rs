use super::*;

pub(super) fn matrix_validation_summary(
    matrix_model: Option<&FullSurfaceMatrixModel>,
    surfaces: &[SurfaceReadiness],
    release_line: &str,
) -> (bool, Option<String>, u8, Vec<String>) {
    validate_full_surface_matrix(matrix_model, surfaces, release_line)
}
