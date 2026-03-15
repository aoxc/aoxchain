use crate::node::state::AOXCNode;

pub struct NodeRuntime {
    pub node: AOXCNode,
}

impl NodeRuntime {
    #[must_use]
    pub fn new(node: AOXCNode) -> Self {
        Self { node }
    }
}
