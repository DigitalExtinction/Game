use bevy::{
    ecs::system::SystemParam,
    prelude::{Component, Query, Transform},
};
use de_map::size::MapBounds;
use parry3d::{
    math::Isometry,
    na::{DMatrix, Vector3},
    query::{Ray, RayCast, RayIntersection},
    shape::HeightField,
};

#[derive(Component)]
pub struct Terrain {
    heightfield: HeightField,
}

impl Terrain {
    pub fn flat(bounds: MapBounds) -> Self {
        let size = bounds.size();
        let heightfield = HeightField::new(
            DMatrix::from_row_slice(2, 2, &[0., 0., 0., 0.]),
            Vector3::new(size.x, 1., size.y),
        );

        Self { heightfield }
    }

    fn cast_ray(&self, m: &Isometry<f32>, ray: &Ray, max_toi: f32) -> Option<RayIntersection> {
        self.heightfield
            .cast_ray_and_get_normal(m, ray, max_toi, true)
    }
}

#[derive(SystemParam)]
pub struct TerrainCollider<'w, 's> {
    terrains: Query<'w, 's, (&'static Terrain, &'static Transform)>,
}

impl<'w, 's> TerrainCollider<'w, 's> {
    pub fn cast_ray(&self, ray: &Ray, max_toi: f32) -> Option<RayIntersection> {
        self.terrains
            .iter()
            .filter_map(|(terrain, transform)| {
                let isometry = Isometry::new(
                    transform.translation.into(),
                    transform.rotation.to_scaled_axis().into(),
                );

                terrain.cast_ray(&isometry, ray, max_toi)
            })
            .min_by(|a, b| {
                a.toi
                    .partial_cmp(&b.toi)
                    .expect("partial_cmp between two terrain intersection ToI failed.")
            })
    }

    pub fn cast_ray_bidir(&self, ray: &Ray, max_toi: f32) -> Option<RayIntersection> {
        self.cast_ray(ray, max_toi)
            .or_else(|| self.cast_ray_negdir(ray, max_toi))
    }

    fn cast_ray_negdir(&self, ray: &Ray, max_toi: f32) -> Option<RayIntersection> {
        self.cast_ray(&Ray::new(ray.origin, -ray.dir), max_toi)
            .map(|intersection| {
                RayIntersection::new(
                    -intersection.toi,
                    -intersection.normal,
                    intersection.feature,
                )
            })
    }
}

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    use glam::{Vec2, Vec3};

    use super::*;

    #[test]
    fn test_cast_ray_bidir() {
        let mut world = World::default();

        world
            .spawn()
            .insert(Terrain::flat(MapBounds::new(Vec2::new(100., 200.))))
            .insert(Transform::from_translation(10000. * Vec3::ONE));
        world
            .spawn()
            .insert(Terrain::flat(MapBounds::new(Vec2::new(100., 200.))))
            .insert(Transform::from_xyz(-17., 3.2, -22.));

        fn help_system(mut commands: Commands, terrain: TerrainCollider) {
            let ray = Ray::new(Vec3::new(0., 10., 0.).into(), Vec3::new(2., -1., 1.).into());
            let intersection = terrain.cast_ray_bidir(&ray, f32::INFINITY).unwrap();
            commands.insert_resource(Vec3::from(ray.point_at(intersection.toi)));
        }

        let mut stage = SystemStage::parallel();
        stage.add_system(help_system);
        stage.run(&mut world);

        let intersection = *world.get_resource::<Vec3>().unwrap();
        assert!(Vec3::new(13.6, 3.2, 6.8).distance(intersection) < 0.00001);
    }
}
