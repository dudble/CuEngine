use engine_core::glam::Vec2;
use resource::scene::{ComponentInstanceRes, LevelRes};
use serde::Deserialize;

use super::component::{Component, Mover};
use super::transform::Transform2D;
use super::world::World;

#[derive(Deserialize)]
struct MoverData {
    velocity: [f32; 2],
    min_x: f32,
    max_x: f32,
}

fn component_from_instance(instance: &ComponentInstanceRes) -> Option<Box<dyn Component>> {
    match instance.type_name.as_str() {
        "mover" => {
            let data = instance.data.clone()?;
            let data: MoverData = data.try_into().ok()?;
            Some(Box::new(Mover::new(
                Vec2::new(data.velocity[0], data.velocity[1]),
                data.min_x,
                data.max_x,
            )))
        }
        other => {
            engine_core::log_warn!("unknown component type: {other}");
            None
        }
    }
}

pub fn world_from_level(level: &LevelRes) -> World {
    let mut world = World::new();

    for object_res in &level.objects {
        let object = world.level_mut().create_object(object_res.name.clone());
        *object.transform_mut() = Transform2D::new(Vec2::new(
            object_res.position[0],
            object_res.position[1],
        ));

        for instance in &object_res.components {
            if let Some(component) = component_from_instance(instance) {
                object.add_component(component);
            }
        }
    }

    world
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_mover_from_instance() {
        let data: toml::Value =
            toml::from_str("velocity = [3.0, 0.0]\nmin_x = -4.0\nmax_x = 4.0").expect("value");
        let instance = ComponentInstanceRes {
            type_name: "mover".to_string(),
            data: Some(data),
        };

        assert!(component_from_instance(&instance).is_some());
    }

    #[test]
    fn ignores_unknown_component() {
        let instance = ComponentInstanceRes {
            type_name: "unknown".to_string(),
            data: None,
        };

        assert!(component_from_instance(&instance).is_none());
    }
}
