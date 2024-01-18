use crate::{
    system::System,
    world::{meta::AccessType, World},
};
use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    dependencies: Vec<NodeId>,
}

impl Node {
    pub fn run(&self, world: &World) {
        self.system.run(world);
    }

    pub fn reads(&self) -> &[AccessType] {
        self.system.reads()
    }

    pub fn writes(&self) -> &[AccessType] {
        self.system.writes()
    }

    pub fn dependencies(&self) -> &[NodeId] {
        &self.dependencies
    }
}

impl Node {
    pub fn new(system: System) -> Self {
        Self {
            system,
            dependencies: Vec::new(),
        }
    }

    pub fn add_dependency(&mut self, node_id: NodeId) {
        self.dependencies.push(node_id);
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

    pub fn add_system(&mut self, mut system: System) -> NodeId {
        let after_nodes = std::mem::take(system.afters_mut());
        let before_nodes = std::mem::take(system.befores_mut());
        let node = Node::new(system);

        let node_id = self.add_node(node);

        for after in after_nodes {
            let after_id = self.add_system(after);
            self.nodes[*node_id].add_dependency(after_id);
        }

        for before in before_nodes {
            let before_id = self.add_system(before);
            self.nodes[*before_id].add_dependency(node_id);
        }

        node_id
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);

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

    pub fn reads(&self) -> Vec<AccessType> {
        self.nodes
            .iter()
            .flat_map(|node| node.reads().to_vec())
            .collect()
    }

    pub fn writes(&self) -> Vec<AccessType> {
        self.nodes
            .iter()
            .flat_map(|node| node.writes().to_vec())
            .collect()
    }

    pub fn build(&mut self) {
        let mut dependency_graph = HashMap::<NodeId, HashSet<NodeId>>::new();
        for (i, node) in self.nodes.iter().enumerate() {
            dependency_graph.insert(NodeId::new(i), HashSet::new());
            for (j, other_node) in self.nodes.iter().enumerate() {
                if i == j
                    || dependency_graph
                        .get(&NodeId::new(j))
                        .and_then(|set| set.get(&NodeId::new(i)))
                        .is_some()
                {
                    continue;
                }

                let writes = node.writes();
                let reads = other_node.reads();

                if writes
                    .iter()
                    .any(|write| (*write) != AccessType::None && reads.contains(write))
                {
                    dependency_graph
                        .entry(NodeId::new(i))
                        .or_insert_with(HashSet::new)
                        .insert(NodeId::new(j));
                }
            }

            for dependency in node.dependencies() {
                dependency_graph
                    .entry(NodeId::new(i))
                    .or_insert_with(HashSet::new)
                    .insert(*dependency);
            }
        }

        let mut hierarchy = Vec::new();

        while !dependency_graph.is_empty() {
            let group = dependency_graph
                .keys()
                .filter_map(|node_id| {
                    dependency_graph
                        .iter()
                        .all(|(_, other_dependencies)| !other_dependencies.contains(node_id))
                        .then_some(*node_id)
                })
                .collect::<Vec<NodeId>>();

            for node_id in &group {
                dependency_graph.remove(node_id);
            }

            hierarchy.insert(0, group);
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
