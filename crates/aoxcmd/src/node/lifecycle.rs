use crate::node::state::{self, AOXCNode, NodeInitError};

pub fn bootstrap_node() -> Result<AOXCNode, NodeInitError> {
    state::setup()
}
