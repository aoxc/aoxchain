use super::catalog::CommandSpec;

pub fn render_preview(spec: &CommandSpec) -> String {
    let mut parts = vec![spec.program.to_string()];
    parts.extend(spec.args.iter().map(|s| s.to_string()));
    parts.join(" ")
}
