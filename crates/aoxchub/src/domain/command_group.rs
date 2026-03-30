#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandGroup {
    Diagnostics,
    NodeOperations,
    WorkspaceOperations,
    Packaging,
    Recovery,
}
