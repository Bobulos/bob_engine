use std::collections::HashSet;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::PhysicalKey;

pub struct Input {
    /// Keys currently held down
    held: HashSet<PhysicalKey>,
    /// Keys that went down this frame only
    just_pressed: HashSet<PhysicalKey>,
    /// Keys that were released this frame
    just_released: HashSet<PhysicalKey>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            held: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }

    /// Call once at the START of each frame to clear per-frame state
    pub fn flush(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Call from App::window_event for every KeyboardInput event
    pub fn receive_input_from_app(&mut self, key_event: KeyEvent) {
        let key = key_event.physical_key;
        match key_event.state {
            ElementState::Pressed => {
                // repeat=true means the key is being held, not freshly pressed
                if !key_event.repeat {
                    self.just_pressed.insert(key);
                }
                self.held.insert(key);
            }
            ElementState::Released => {
                self.held.remove(&key);
                self.just_released.insert(key);
            }
        }
    }

    /// True only on the frame the key was first pressed
    pub fn get_key_pressed(&self, key: PhysicalKey) -> bool {
        self.just_pressed.contains(&key)
    }

    /// True every frame the key is held down
    pub fn get_key_down(&self, key: PhysicalKey) -> bool {
        self.held.contains(&key)
    }

    /// True only on the frame the key was released
    pub fn get_key_released(&self, key: PhysicalKey) -> bool {
        self.just_released.contains(&key)
    }
}