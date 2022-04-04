use anyhow::bail;
use bevy::{prelude::Transform, reflect::TypeUuid};
use glam::{Quat, Vec3};
use serde::Deserialize;
use std::collections::HashSet;

const MAX_PLAYERS: u8 = 16;

#[derive(TypeUuid, Deserialize)]
#[uuid = "2f2f3f01-8184-4824-beab-50ed0d81550e"]
pub struct MapDescription {
    pub size: MapSize,
    max_players: u8,
    inactive_objects: Vec<InactiveObject>,
    active_objects: Vec<ActiveObject>,
}

impl MapDescription {
    pub fn inactive_objects(&self) -> &[InactiveObject] {
        self.inactive_objects.as_slice()
    }

    pub fn active_objects(&self) -> &[ActiveObject] {
        self.active_objects.as_slice()
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.max_players < 2 {
            bail!("Maximum number of players in the game must be at least 2.");
        }
        if self.max_players > MAX_PLAYERS {
            bail!(
                "Maximum number of players is {}, got: {}",
                MAX_PLAYERS,
                self.max_players
            );
        }

        if !self.size.0.is_finite() {
            bail!("Map size has to be finite, got: {}", self.size.0);
        }
        if self.size.0 <= 0. {
            bail!("Map size has to be positive, got: {}", self.size.0);
        }

        self.validate_positions(&self.active_objects)?;
        self.validate_positions(&self.inactive_objects)?;
        self.validate_players()?;
        Ok(())
    }

    fn validate_positions<'a, O>(
        &self,
        objects: impl IntoIterator<Item = &'a O>,
    ) -> anyhow::Result<()>
    where
        O: 'a + MapObject,
    {
        for object in objects {
            let x = object.position().position[0];
            let y = object.position().position[1];
            if x < 0. || x > self.size.0 || y < 0. || y > self.size.0 {
                bail!("An object is placed outside of the map: ({}, {})", x, y);
            }
        }

        Ok(())
    }

    fn validate_players(&self) -> anyhow::Result<()> {
        let mut encountered: HashSet<u8> = HashSet::with_capacity(self.max_players as usize);
        for object in &self.active_objects {
            if object.player >= self.max_players {
                bail!(
                    "Encountered object with player {} but the map has maximum players of {}",
                    object.player,
                    self.max_players
                );
            }
            encountered.insert(object.player);
        }
        if encountered.len() < self.max_players as usize {
            bail!(
                "All players must have at least one object on the map. Got players: {}",
                encountered
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        }
        Ok(())
    }
}

pub trait MapObject {
    fn model_name(&self) -> &'static str;
    fn position(&self) -> &ObjectPosition;
}

#[derive(Clone, Copy, Debug, TypeUuid, Deserialize)]
#[uuid = "bbf80d94-c4de-4c7c-9bdc-552ef25aff4e"]
pub struct MapSize(pub f32);

#[derive(Deserialize)]
pub struct ObjectPosition {
    position: [f32; 2],
    rotation: f32,
}

impl ObjectPosition {
    pub fn transform(&self) -> Transform {
        let translation = Vec3::new(self.position[0], 0., self.position[1]);
        let rotation = Quat::from_rotation_y(self.rotation);
        Transform {
            translation,
            rotation,
            ..Default::default()
        }
    }
}

#[derive(Copy, Clone, Deserialize)]
pub enum InactiveObjectType {
    Tree,
}

#[derive(Deserialize)]
pub struct InactiveObject {
    object_type: InactiveObjectType,
    position: ObjectPosition,
}

impl MapObject for InactiveObject {
    fn model_name(&self) -> &'static str {
        match self.object_type {
            InactiveObjectType::Tree => "tree01",
        }
    }

    fn position(&self) -> &ObjectPosition {
        &self.position
    }
}

#[derive(Copy, Clone, Deserialize)]
pub enum ActiveObjectType {
    Base,
    PowerHub,
    Attacker,
}

#[derive(Deserialize)]
pub struct ActiveObject {
    object_type: ActiveObjectType,
    position: ObjectPosition,
    player: u8,
}

impl ActiveObject {
    pub fn player(&self) -> u8 {
        self.player
    }
}

impl MapObject for ActiveObject {
    fn model_name(&self) -> &'static str {
        match self.object_type {
            ActiveObjectType::Base => "base",
            ActiveObjectType::PowerHub => "powerhub",
            ActiveObjectType::Attacker => "attacker",
        }
    }

    fn position(&self) -> &ObjectPosition {
        &self.position
    }
}
