use super::*;

#[derive(serde::Serialize)]
struct ApiContract<'a> {
    release_line: &'a str,
    admission_controls: Vec<&'a str>,
    rest_endpoints: Vec<&'a str>,
    json_rpc_methods: Vec<&'a str>,
    compatibility_contract: Vec<&'a str>,
}

pub fn cmd_api_contract(args: &[String]) -> Result<(), AppError> {
    let contract = ApiContract {
        release_line: AOXC_Q_RELEASE_LINE,
        admission_controls: vec![
            "idempotency key required on write paths",
            "request signature envelope validated before admission",
            "adaptive rate limiting with rejection telemetry",
            "strict request/response schema compatibility gates",
        ],
        rest_endpoints: vec![
            "/health",
            "/status",
            "/metrics",
            "/chain/status",
            "/block/latest",
            "/block/{height}",
            "/tx/{hash}",
            "/tx/{hash}/receipt",
            "/account/{id}",
            "/consensus/status",
            "/network/peers",
            "/vm/status",
            "/state/root",
            "/rpc/status",
            "/faucet/status",
            "/faucet/claim",
            "/faucet/history/{account_id}",
            "/faucet/balance",
            "/faucet/config",
            "/faucet/enable",
            "/faucet/disable",
            "/faucet/ban",
            "/faucet/unban",
            "/faucet/config/update",
        ],
        json_rpc_methods: vec![
            "status",
            "getLatestBlock",
            "getBlockByHeight",
            "getBlockByHash",
            "getTxByHash",
            "getReceiptByHash",
            "getAccount",
            "getBalance",
            "getStateRoot",
            "getConsensusStatus",
            "getNetworkStatus",
            "getPeers",
            "getVmStatus",
        ],
        compatibility_contract: vec![
            "read methods are backward compatible across minor releases",
            "write method schema expansion is additive-only within release line",
            "breaking changes require release-line bump and compatibility matrix update",
        ],
    };

    emit_serialized(&contract, output_format(args))
}
