use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
};

use crate::ast::NodeId;

#[derive(Debug, Default)]
pub struct Tags {
    tags: HashMap<TypeId, HashMap<NodeId, Box<dyn Any>>>,
}

impl Tags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<T: 'static>(&self, node_id: NodeId) -> Option<&T> {
        self.tags
            .get(&TypeId::of::<T>())
            .and_then(|tags| tags.get(&node_id))
            .and_then(|tag| tag.downcast_ref())
    }

    pub fn get_mut<T: 'static>(&mut self, node_id: NodeId) -> Option<&mut T> {
        self.tags
            .get_mut(&TypeId::of::<T>())
            .and_then(|tags| tags.get_mut(&node_id))
            .and_then(|tag| tag.downcast_mut())
    }

    pub fn insert<T: 'static>(&mut self, node_id: NodeId, tag: T) {
        self.tags
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(node_id, Box::new(tag));
    }

    pub fn tag<T: 'static + Default>(&mut self, node_id: NodeId) -> &mut T {
        match self.tags.entry(TypeId::of::<T>()) {
            Entry::Occupied(tags) => tags.into_mut(),
            Entry::Vacant(tags) => tags.insert(HashMap::new()),
        }
        .entry(node_id)
        .or_insert_with(|| Box::new(T::default()))
        .downcast_mut()
        .unwrap()
    }

    pub fn tags<T: 'static>(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.tags
            .get(&TypeId::of::<T>())
            .into_iter()
            .flat_map(|tags| {
                tags.iter()
                    .map(|(node_id, tag)| (*node_id, tag.downcast_ref().unwrap()))
            })
    }
}
