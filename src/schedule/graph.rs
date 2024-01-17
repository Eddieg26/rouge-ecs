use crate::{system::System, world::World};
use std::any::TypeId;

pub struct NodeId(usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

impl std::ops::Deref for NodeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for NodeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Node {
    system: System,
}

impl Node {
    pub fn run(&self, world: &World) {
        self.system.run(world);
    }

    pub fn reads(&self) -> &[TypeId] {
        self.system.reads()
    }

    pub fn writes(&self) -> &[TypeId] {
        self.system.writes()
    }
}

pub struct SystemGraph {
    nodes: Vec<Node>,
    hierarchy: Vec<Vec<NodeId>>,
}

impl SystemGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            hierarchy: Vec::new(),
        }
    }

    pub fn add_system(&mut self, system: System) -> NodeId {
        let id = NodeId(self.nodes.len());

        self.nodes.push(Node { system });

        id
    }

    pub fn append(&mut self, other: &mut Self) {
        let offset = self.nodes.len();

        self.nodes.append(&mut other.nodes);

        for parents in &mut other.hierarchy {
            for parent in parents {
                parent.0 += offset;
            }
        }
    }

    pub fn reads(&self) -> Vec<TypeId> {
        self.nodes
            .iter()
            .flat_map(|node| node.reads().to_vec())
            .collect()
    }

    pub fn writes(&self) -> Vec<TypeId> {
        self.nodes
            .iter()
            .flat_map(|node| node.writes().to_vec())
            .collect()
    }

    pub fn build(&mut self) {
        let mut hierarchy = Vec::new();

        for node in &self.nodes {
            let mut reads = node.reads().to_vec();
            let mut writes = node.writes().to_vec();

            for node in self.nodes.iter() {
                if reads.iter().any(|read| node.writes().contains(read)) {
                    reads.extend(node.reads());
                }

                if writes.iter().any(|write| node.reads().contains(write)) {
                    writes.extend(node.writes());
                }
            }

            reads.sort();
            reads.dedup();

            writes.sort();
            writes.dedup();

            let mut parents = Vec::new();

            for (id, node) in self.nodes.iter().enumerate() {
                if reads.iter().any(|read| node.writes().contains(read)) {
                    parents.push(NodeId(id));
                }
            }

            hierarchy.push(parents);
        }

        self.hierarchy = hierarchy;
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn hierarchy(&self) -> &[Vec<NodeId>] {
        &self.hierarchy
    }
}
