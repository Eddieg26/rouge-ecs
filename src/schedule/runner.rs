use super::graph;
use crate::{
    tasks::{barrier::JobBarrier, ScopedTaskPool},
    world::World,
};
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Sequential,
    Parallel,
}

pub trait ScheduleRunner: Send + Sync {
    fn run(&self, graph: &graph::SystemGraph, world: &World);
}

pub struct SequentialRunner;

impl ScheduleRunner for SequentialRunner {
    fn run(&self, graph: &graph::SystemGraph, world: &World) {
        for node in graph.nodes() {
            node.run(world);
        }
    }
}

pub struct ParallelRunner;

impl ScheduleRunner for ParallelRunner {
    fn run(&self, graph: &graph::SystemGraph, world: &World) {
        for row in graph.hierarchy() {
            let num_threads = row.len().min(
                std::thread::available_parallelism()
                    .unwrap_or(NonZeroUsize::new(1).unwrap())
                    .into(),
            );

            let pool = ScopedTaskPool::new(num_threads);

            let barrier = Arc::new(Mutex::new(JobBarrier::new(row.len())));

            for node in row {
                let barrier = barrier.clone();
                let node = &graph.nodes()[node.id()];

                pool.execute(move || {
                    node.run(world);

                    barrier.lock().unwrap().notify();
                });
            }

            barrier.lock().unwrap().wait(barrier.lock().unwrap());
        }
    }
}
