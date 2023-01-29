#[derive(Debug, Clone)]
pub enum Node {
    /// This usually proceeds after a DEFINE FUNCTION/MODULE.
    Block(Vec<Box<Node>>),
    Module {
        name: String,
        nodes: Vec<Box<Node>>,
    },
}
