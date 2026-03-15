use crate::runtime::core::CoreRuntime;
use crate::runtime::node::NodeRuntime;
use crate::runtime::unity::UnityRuntime;

pub struct RuntimeHandles {
    pub core: CoreRuntime,
    pub unity: UnityRuntime,
    pub node: NodeRuntime,
}
