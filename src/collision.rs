use bevy::{prelude::*, sprite::collide_aabb::collide};

#[derive(Debug, Component, Default, Clone, Copy)]
pub struct Collider;

#[derive(Debug, Copy, Clone)]
pub struct CollisionEvent {
    pub other: Entity,
}

#[derive(Debug, Component, Default, Clone)]
pub struct CollisionHolder {
    pub events: Vec<CollisionEvent>,
}

#[derive(Debug, Resource)]
pub struct CollisionTimer(pub Timer);

// TODO should be a quad tree thing I forget the name the one with the squares
pub fn collision_detection_system(
    mut collision_q: Query<(Entity, &Transform, &mut CollisionHolder), With<Collider>>,
    colliders_q: Query<(Entity, &Transform), With<Collider>>,
) {
    for (entity, transform, mut col) in &mut collision_q {
        let size = transform.scale.truncate();

        col.events.clear();
        for (other_entity, other_trans) in &colliders_q {
            if entity == other_entity {
                continue;
            }

            let collision: Option<bevy::sprite::collide_aabb::Collision> = collide(
                transform.translation,
                size,
                other_trans.translation,
                other_trans.scale.truncate(),
            );
            if collision.is_none() {
                continue;
            }

            col.events.push(CollisionEvent {
                other: other_entity,
            });
        }
    }
}
