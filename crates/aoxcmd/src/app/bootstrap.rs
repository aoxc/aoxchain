use crate::config::loader::SettingsLoader;
use crate::error::app_error::AppError;
use crate::keys::{KeyBootstrapRequest, KeyManager, KeyPaths};
use crate::node::lifecycle::bootstrap_node;
use crate::runtime::context::RuntimeContext;
use crate::runtime::core::CoreRuntime;
use crate::runtime::handles::RuntimeHandles;
use crate::runtime::node::NodeRuntime;
use crate::runtime::unity::UnityRuntime;
use crate::services::wiring::wire_defaults;

use aoxcore::identity::ca::CertificateAuthority;

pub fn bootstrap(password: &str) -> Result<(RuntimeContext, RuntimeHandles), AppError> {
    if password.is_empty() {
        return Err(AppError::InvalidArgument(
            "password must not be empty".to_string(),
        ));
    }

    let settings = SettingsLoader::load_default();

    let key_paths = KeyPaths::new(settings.keys_dir(), &settings.key_name);
    let request = KeyBootstrapRequest::new(
        settings.chain.clone(),
        settings.role.clone(),
        settings.zone.clone(),
        password,
        settings.cert_validity_secs,
    );

    let manager = KeyManager::new(key_paths, request);
    let ca = CertificateAuthority::new(settings.ca_issuer.clone());
    let material = manager.load_or_create(&ca)?;

    let node = bootstrap_node()?;

    let _services = wire_defaults();

    let context = RuntimeContext {
        settings,
        key_summary: material.summary(),
    };

    let handles = RuntimeHandles {
        core: CoreRuntime::new("AOXC-MAIN"),
        unity: UnityRuntime::new(
            node.consensus.quorum.numerator,
            node.consensus.quorum.denominator,
        ),
        node: NodeRuntime::new(node),
    };

    Ok((context, handles))
}
