use super::*;

pub(super) fn build(context: &SurfaceContext) -> SurfaceReadiness {
    build_surface(
        "desktop-wallet",
        "client-platform",
        vec![
            surface_check(
                "desktop-wallet-compat",
                has_desktop_wallet_compat_artifact(&context.closure_dir),
                format!(
                    "desktop wallet compatibility artifact at {}",
                    context
                        .closure_dir
                        .join("desktop-wallet-compat.json")
                        .display()
                ),
            ),
            surface_check(
                "production-audit",
                context.closure_dir.join("production-audit.json").exists(),
                format!(
                    "wallet release decisions rely on {}",
                    context.closure_dir.join("production-audit.json").display()
                ),
            ),
            surface_check(
                "rpc-integration-doc",
                context.frontend_rpc_doc.exists(),
                format!(
                    "expected integration guide at {}",
                    context.frontend_rpc_doc.display()
                ),
            ),
        ],
        vec![
            context
                .closure_dir
                .join("desktop-wallet-compat.json")
                .display()
                .to_string(),
            context.frontend_rpc_doc.display().to_string(),
        ],
    )
}
