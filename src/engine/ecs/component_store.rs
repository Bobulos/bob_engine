/// Stores components of type `T` densely indexed by entity ID.
pub struct ComponentStore<T> {
    components: Vec<Option<T>>,
}

impl<T> ComponentStore<T> {
    pub fn new() -> Self {
        Self { components: Vec::new() }
    }

    pub fn insert(&mut self, entity_id: usize, component: T) {
        if entity_id >= self.components.len() {
            self.components.resize_with(entity_id + 1, || None);
        }
        self.components[entity_id] = Some(component);
    }

    pub fn remove(&mut self, entity_id: usize) {
        if let Some(slot) = self.components.get_mut(entity_id) {
            *slot = None;
        }
    }

    pub fn get(&self, entity_id: usize) -> Option<&T> {
        self.components.get(entity_id)?.as_ref()
    }

    pub fn get_mut(&mut self, entity_id: usize) -> Option<&mut T> {
        self.components.get_mut(entity_id)?.as_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.components
            .iter()
            .enumerate()
            .filter_map(|(id, c)| c.as_ref().map(|c| (id, c)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.components
            .iter_mut()
            .enumerate()
            .filter_map(|(id, c)| c.as_mut().map(|c| (id, c)))
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }
}