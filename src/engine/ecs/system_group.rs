use crate::b_engine::system_bootstrap;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::b_engine::entities::{DynamicWorld, SystemBase};
pub struct SystemGroup {
    //systems: RwLock<Vec<Box<dyn SystemBase>>>,
    threading: SystemGroupThreading,
    update_order: Vec<i16>,
    systems: Arc<RwLock<Vec<Box<dyn SystemBase>>>>,
    world: Arc<DynamicWorld>,
}

/// All systems registered to a system group run on the same thread
/// Multiple system groups can share a world
/// Avoid haveing too many system group because each one takes a thread
impl SystemGroup {
    pub fn new(world: Arc<DynamicWorld>, threading: SystemGroupThreading) -> Self {
        Self {
            update_order: Vec::new(),
            threading: threading,
            systems: Arc::new(RwLock::new(Vec::new())),
            world,
        }
    }
    pub fn update(&mut self) {
        match self.threading {
            SystemGroupThreading::Main => self.run_systems(),
            SystemGroupThreading::Parallel => self.run_systems_parrallel(),
        }
    }
    /// Registers a system and returns it's index in the group
    /// Update order dictates when the system starts
    /// also dictates the update order of the system
    pub fn register_system(
        &mut self,
        mut system: Box<dyn SystemBase + Send + Sync>,
        order: i16,
    ) -> usize {
        let mut systems = self.systems.write().unwrap();
        systems.push(system);
        self.update_order.push(order);
        systems.len() - 1
    }
    pub fn destroy_system(&mut self, system_index: usize) {
        let mut systems = self.systems.write().unwrap();
        systems[system_index].on_destroy(&self.world);
        systems.remove(system_index);
    }
    fn sort_systems(&mut self) {
        for (i, order) in self.update_order.iter().enumerate() {
            let value = *order as i16;
        }
    }
    pub fn start_systems(&self) {
        for system in self.systems.write().unwrap().iter_mut() {
            system.on_start(&self.world);
        }
    }
    pub fn destroy_systems(&self) {
        for system in self.systems.write().unwrap().iter_mut() {
            system.on_destroy(&self.world);
        }
    }
    /// Runs system on the main thread
    pub fn run_systems(&mut self) {
        //println!("Running system group with {} systems", self.systems.read().unwrap().len());
        for system in self.systems.write().unwrap().iter_mut() {
            system.on_update(&self.world);
        }
    }
    /// Runs system on a worker thread
    pub fn run_systems_parrallel(&self) {
        let systems = Arc::clone(&self.systems);
        let world = Arc::clone(&self.world);

        thread::spawn(move || {
            let mut systems = systems.write().unwrap();
            for system in systems.iter_mut() {
                system.on_update(&world);
            }
        });
    }
}

pub enum SystemGroupThreading {
    Main,
    Parallel,
}
