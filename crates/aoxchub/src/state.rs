use dioxus::prelude::*;

use crate::services::rpc_client::RpcClient;

pub fn provide_global_state() {
    use_context_provider(RpcClient::from_env);
    use_context_provider(|| Signal::new(0_u64));
}
