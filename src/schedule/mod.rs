use crate::{
    storage::sparse::SparseMap,
    system::{IntoSystem, System},
    world::{resource::Resource, World},
};
use std::any::{Any, TypeId};

pub trait ScheduleLabel: 'static {
    const LABEL: &'static str;
}

pub struct DefaultLabel;

impl ScheduleLabel for DefaultLabel {
    const LABEL: &'static str = "default";
}

pub trait SchedulePhase: 'static {
    const PHASE: &'static str;
}

pub struct Schedule {
    systems: Vec<System>,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system<M>(&mut self, system: impl IntoSystem<M>) {
        self.systems.push(system.into_system());
    }

    pub fn append(&mut self, mut schedule: Schedule) {
        self.systems.append(&mut schedule.systems);
    }

    pub fn reads(&self) -> Vec<TypeId> {
        self.systems
            .iter()
            .flat_map(|system| system.reads().to_vec())
            .collect()
    }

    pub fn writes(&self) -> Vec<TypeId> {
        self.systems
            .iter()
            .flat_map(|system| system.writes().to_vec())
            .collect()
    }

    pub fn run(&self, world: &World) {
        for system in &self.systems {
            system.run(world);
        }
    }
}

pub struct Schedules {
    schedules: SparseMap<TypeId, SparseMap<TypeId, Schedule>>,
}

impl Schedules {
    pub fn new() -> Self {
        Self {
            schedules: SparseMap::new(),
        }
    }

    pub fn add_system<M>(
        &mut self,
        phase: impl SchedulePhase,
        label: impl ScheduleLabel,
        system: impl IntoSystem<M>,
    ) {
        let phase_id = phase.type_id();
        let label_id = label.type_id();

        if let Some(phase) = self.schedules.get_mut(&phase_id) {
            if let Some(schedule) = phase.get_mut(&label_id) {
                schedule.add_system(system);
            } else {
                let mut schedule = Schedule::new();
                schedule.add_system(system);
                phase.insert(label_id, schedule);
            }
        } else {
            let mut phase = SparseMap::new();
            let mut schedule = Schedule::new();
            schedule.add_system(system);
            phase.insert(label_id, schedule);
            self.schedules.insert(phase_id, phase);
        }
    }

    pub fn add_schedule(
        &mut self,
        phase: impl SchedulePhase,
        label: impl ScheduleLabel,
        schedule: Schedule,
    ) {
        let phase_id = phase.type_id();
        let label_id = label.type_id();

        if let Some(phase) = self.schedules.get_mut(&phase_id) {
            if let Some(found) = phase.get_mut(&label_id) {
                found.append(schedule);
            } else {
                phase.insert(label_id, schedule);
            }
        } else {
            let mut phase = SparseMap::new();
            phase.insert(label_id, schedule);
            self.schedules.insert(phase_id, phase);
        }
    }

    pub fn run<P: SchedulePhase>(&self, world: &World) {
        let phase_id = TypeId::of::<P>();

        if let Some(phase) = self.schedules.get(&phase_id) {
            for schedule in phase.values() {
                schedule.run(world);
            }
        }
    }

    pub fn clear(&mut self) {
        self.schedules.clear();
    }
}

pub struct GlobalSchedules(Schedules);

impl GlobalSchedules {
    pub fn new() -> Self {
        Self(Schedules::new())
    }
}

impl From<Schedules> for GlobalSchedules {
    fn from(schedules: Schedules) -> Self {
        Self(schedules)
    }
}

impl std::ops::Deref for GlobalSchedules {
    type Target = Schedules;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Resource for GlobalSchedules {}

impl std::ops::DerefMut for GlobalSchedules {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct SceneSchedules(Schedules);

impl SceneSchedules {
    pub fn new() -> Self {
        Self(Schedules::new())
    }
}

impl From<Schedules> for SceneSchedules {
    fn from(schedules: Schedules) -> Self {
        Self(schedules)
    }
}

impl std::ops::Deref for SceneSchedules {
    type Target = Schedules;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SceneSchedules {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Resource for SceneSchedules {}
