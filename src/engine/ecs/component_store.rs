// T is anything struct type of component data
pub struct ComponentStore<T> {
    components: Vec<Option<T>>,
}

impl<T> ComponentStore<T> {
    pub fn new() -> Self {
        Self { components: Vec::new() }
    }

    pub fn insert(&mut self, entity_id: usize, component: T) {
        // If the vector is too small, grow it
        if entity_id >= self.components.len() {
            self.components.resize_with(entity_id + 1, || None);
        }
        self.components[entity_id] = Some(component);
    }
    
    pub fn get(&self, entity_id: usize) -> Option<&T> {
        self.components.get(entity_id)?.as_ref()
    }

    // Returns an itterator
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.components.iter().enumerate().filter_map(|(id, comp)| {
            comp.as_ref().map(|c| (id, c))
        })
    }
}