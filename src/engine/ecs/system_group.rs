use std::sync::{Arc,RwLock};
use std::thread;

use crate::b_engine::entities::{DynamicWorld, SystemBase};
pub struct SystemGroup {
    //systems: RwLock<Vec<Box<dyn SystemBase>>>,
    threading: SystemGroupThreading,
    systems: Arc<RwLock<Vec<Box<dyn SystemBase>>>>,
    world: Arc<DynamicWorld>
}

/// All systems registered to a system group run on the same thread
/// Multiple system groups can share a world
/// Avoid haveing too many system group because each one takes a thread
impl SystemGroup {
    pub fn new(world: Arc<DynamicWorld>, threading: SystemGroupThreading) -> Self {
        Self {
            threading: threading,
            systems: Arc::new(RwLock::new(Vec::new())),
            world,
        }
    }

    /// Registers a system and returns it's index in the register
    /// The systems will run in the order registered
    /// Calls on_start() for the system
    pub fn register_system(&mut self, system: Box<dyn SystemBase + Send + Sync>) -> usize {
        system.on_start(&self.world);
        let mut systems = self.systems.write().unwrap();
        systems.push(system);
        systems.len() - 1
    }
    pub fn destroy_system(&mut self, system_index: usize) {
        let mut systems = self.systems.write().unwrap();
        systems[system_index].on_destroy(&self.world);
        systems.remove(system_index);
    }

    pub fn start_systems(&self) {
        
    }
    pub fn destroy_systems(&self) {
        
    }
    /// Runs system on the main thread
    pub fn run_systems(&self) {
        for system in self.systems.read().unwrap().iter() {
            system.on_update(&self.world);
        }
    }
    /// Runs system on a worker thread
    pub fn run_systems_parrallel(&self) {
        let systems = Arc::clone(&self.systems);
        let world = Arc::clone(&self.world);

        thread::spawn(move || {
            let systems = systems.read().unwrap();
            for system in systems.iter() {
                system.on_update(&world);
            }
        });
    }

}

pub enum SystemGroupThreading {
    Main,
    Parallel,
}