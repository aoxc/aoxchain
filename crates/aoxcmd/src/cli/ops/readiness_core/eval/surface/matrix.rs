use super::*;

pub(in crate::cli::ops) fn load_full_surface_matrix(
    repo_root: &Path,
) -> (String, Option<FullSurfaceMatrixModel>, Vec<String>) {
    let matrix_path = repo_root
        .join("models")
        .join("full_surface_readiness_matrix_v1.yaml");
    let matrix_path_string = matrix_path.display().to_string();

    let raw = match fs::read_to_string(&matrix_path) {
        Ok(raw) => raw,
        Err(error) => {
            return (
                matrix_path_string,
                None,
                vec![format!("Unable to read canonical matrix: {error}")],
            );
        }
    };

    match serde_yaml::from_str::<FullSurfaceMatrixModel>(&raw) {
        Ok(model) => (matrix_path_string, Some(model), Vec::new()),
        Err(error) => (
            matrix_path_string,
            None,
            vec![format!("Unable to parse canonical matrix YAML: {error}")],
        ),
    }
}

pub(in crate::cli::ops) fn validate_full_surface_matrix(
    matrix: Option<&FullSurfaceMatrixModel>,
    surfaces: &[SurfaceReadiness],
    release_line: &str,
) -> (bool, Option<String>, u8, Vec<String>) {
    let Some(matrix) = matrix else {
        return (false, None, 0, Vec::new());
    };

    let mut warnings = Vec::new();
    if matrix.release_line != release_line {
        warnings.push(format!(
            "Matrix release line {} does not match runtime release line {}",
            matrix.release_line, release_line
        ));
    }

    for expected in &matrix.surfaces {
        if expected.required_evidence.is_empty() {
            warnings.push(format!(
                "Matrix surface {} is missing required_evidence entries",
                expected.id
            ));
        }
        if expected.verification_command.trim().is_empty() {
            warnings.push(format!(
                "Matrix surface {} is missing verification_command",
                expected.id
            ));
        }
        if expected.blocker.trim().is_empty() {
            warnings.push(format!(
                "Matrix surface {} is missing blocker text",
                expected.id
            ));
        }

        match surfaces
            .iter()
            .find(|surface| surface.surface == expected.id)
        {
            Some(surface) => {
                if surface.owner != expected.owner {
                    warnings.push(format!(
                        "Matrix owner mismatch for {}: matrix={} runtime={}",
                        expected.id, expected.owner, surface.owner
                    ));
                }
            }
            None => warnings.push(format!(
                "Matrix surface {} is not represented in runtime readiness output",
                expected.id
            )),
        }
    }

    for surface in surfaces {
        if !matrix
            .surfaces
            .iter()
            .any(|expected| expected.id == surface.surface)
        {
            warnings.push(format!(
                "Runtime surface {} is missing from canonical matrix",
                surface.surface
            ));
        }
    }

    (
        true,
        Some(matrix.release_line.clone()),
        matrix.surfaces.len() as u8,
        warnings,
    )
}
