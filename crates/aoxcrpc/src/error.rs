/// RPC subsystem errors.
#[derive(Debug)]
pub enum RpcError {
    InvalidRequest,
    MethodNotFound,
    InternalError,
}
