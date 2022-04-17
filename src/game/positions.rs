use super::{
    mapdescr::MapSize,
    tree::{Rectangle, Tree, TreeItem},
    MAX_ACTIVE_OBJECTS,
};
use bevy::{
    prelude::{
        App, Changed, Component, CoreStage, Entity, GlobalTransform,
        ParallelSystemDescriptorCoercion, Plugin, Query, ResMut,
    },
    transform::TransformSystem,
};
use glam::Vec2;

pub struct PositionPlugin;

impl Plugin for PositionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_tree.after(TransformSystem::TransformPropagate),
        );
    }
}

pub struct MovingEntitiesTree(Tree<Entity>);

impl MovingEntitiesTree {
    pub fn new(map_size: MapSize) -> Self {
        Self(Tree::with_capacity(
            MAX_ACTIVE_OBJECTS,
            Rectangle::new(Vec2::ZERO, Vec2::splat(map_size.0)),
        ))
    }

    pub fn insert(&mut self, entity: Entity, initial_position: Vec2) -> MovingTreeItem {
        MovingTreeItem(self.0.insert(entity, initial_position))
    }
}

#[derive(Component)]
pub struct MovingTreeItem(TreeItem<Entity>);

fn update_tree(
    // This resource has to be optional, because run criteria of the system
    // cannot easily be set to Game::Playing. The system is executed in a
    // different stage and states are hard to share across multiple stages.
    //
    // This should be solved by https://github.com/bevyengine/bevy/discussions/1375
    tree: Option<ResMut<MovingEntitiesTree>>,
    mut changed: Query<(&mut MovingTreeItem, &GlobalTransform), Changed<GlobalTransform>>,
) {
    let mut tree = match tree {
        Some(tree) => tree,
        None => return,
    };

    for (mut tree_item, transform) in changed.iter_mut() {
        let new_position = Vec2::new(transform.translation.x, transform.translation.z);
        tree.0.update_position(&mut tree_item.0, new_position);
    }
}
