use std::{str, collections::HashMap, sync::Arc};

use crate::b_engine::entities::{DynamicWorld, SystemGroup};

/// This is a wrapper for the world and system groups held by the engine
pub struct Entities {
    pub worlds: HashMap<&'static str, Arc<DynamicWorld>>,
    /// Maximum of 16 system groups
    pub system_groups: HashMap<&'static str, SystemGroup>
}
impl Entities {
    pub fn new() -> Self{
        Self {  
            worlds: HashMap::new(),
            system_groups: HashMap::new()
        }
    }

    fn update_system_groups(&mut self) {
        for system in self.system_groups.iter_mut() {
            system.1.run_systems();
        }
    }

    //
    // WORLDS
    //
    pub fn add_world(&mut self, name: &'static str, world: Arc<DynamicWorld>) {
        self.worlds.insert(name, world);
    }
    pub fn get_world(&self, name: &'static str) -> Result<Arc<DynamicWorld>, String> {
        let w = self.worlds.get(name);
        if let Some(world) = w {
            Ok(Arc::clone(world))
        } else {
            let joined = format!("Failed to find world nameof {}", name);
            Err(joined)
        }
    }
    //
    // SYSTEM GROUPS
    //
    pub fn add_system_group(&mut self, name: &'static str, group: SystemGroup) {
        self.system_groups.insert(name, group);
    }
    pub fn get_system_group(&self, name: &'static str) -> Result<&SystemGroup, String> {
        let got = self.system_groups.get(name);
        if let Some(group) = got {
            Ok(group)
        } else {
            let joined = format!("Failed to find world nameof {}", name);
            Err(joined)
        }
    }
    pub fn get_system_group_mut(&mut self, name: &'static str) -> Result<&mut SystemGroup, String> {
        let got = self.system_groups.get_mut(name);
        if let Some(group) = got {
            Ok(group)
        } else {
            let joined = format!("Failed to find world nameof {}", name);
            Err(joined)
        }
    }

}