use engine_core::glam::Vec2;
use function::framework::world::World;
use function::render::render_data::Quad;
use function::render::render_scene::RenderScene;

pub fn extract(world: &World) -> RenderScene {
    let mut scene = RenderScene::new();

    for object in world.level().objects() {
        scene.push_quad(Quad::new(
            object.transform().position,
            Vec2::splat(1.0),
            [0.9, 0.5, 0.2, 1.0],
        ));
    }

    scene
}
