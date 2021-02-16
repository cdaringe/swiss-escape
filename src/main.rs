#[macro_use]
extern crate lazy_static;

use bevy::{math::vec3, prelude::*};
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::rapier::{dynamics::RigidBodyBuilder, geometry::ColliderSet};
use bevy_rapier2d::rapier::geometry::ColliderBuilder;
use bevy_rapier2d::render::RapierRenderPlugin;
use bevy_rapier2d::{
    na::{Vector, Vector2},
    physics::{RapierConfiguration, RapierPhysicsPlugin},
    rapier::{
        dynamics::{RigidBody, RigidBodySet},
        geometry::Collider,
    },
};
struct Ball {}

lazy_static! {
    static ref BOUNDS: Vec2 = Vec2::new(900.0, 600.0);
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut physics_config: ResMut<RapierConfiguration>,
) {
    commands.spawn(Camera2dBundle::default());

    // Static rigid-body with a cuboid shape.
    let ramp_1 = RigidBodyBuilder::new_static().rotation(2.).mass(10.);
    let ramp_collider1 = ColliderBuilder::cuboid(100.0, 10.0);
    commands.spawn((ramp_1, ramp_collider1));

    let ball_radius: f32 = 10.;
    let ball_body_builder = RigidBodyBuilder::new_dynamic()
        .translation(0.0, 50.0)
        .user_data(1);
    let ball_collider = ColliderBuilder::ball(ball_radius);
    let texture_handle = asset_server.load("img/marbles/hazers.png");
    commands
        .spawn((ball_body_builder, ball_collider))
        .with_children(|x| {
            let mut bundle = SpriteBundle::default();
            bundle.sprite = Sprite {
                // this is confusing. this size is 2x the size of the parent. the size
                // of the parent is apparently the radius. so, _scale_ to 2x the parent
                // size?
                size: Vec2::new(2., 2.),
                resize_mode: SpriteResizeMode::Manual,
            };
            bundle.transform = Transform::from_translation(vec3(0., 0., 10.));
            bundle.material = materials.add(texture_handle.into());
            x.spawn(bundle);
        })
        .with(Ball {});

    // walls
    let wall_material = materials.add(Color::rgb(0.8, 0.8, 0.8).into());
    let wall_thickness = 10.0;

    let left_wall = RigidBodyBuilder::new_static().translation(-BOUNDS.x / 2.0, 0.);
    let left_wall_collider = ColliderBuilder::cuboid(wall_thickness, BOUNDS.y);
    commands.spawn((left_wall, left_wall_collider));

    let bottom_wall = RigidBodyBuilder::new_static().translation(0., -BOUNDS.y / 2.);
    let bottom_wall_collider = ColliderBuilder::cuboid(BOUNDS.x, wall_thickness);
    commands.spawn((bottom_wall, bottom_wall_collider));

    let right_wall = RigidBodyBuilder::new_static().translation(BOUNDS.x / 2., 0.);
    let right_wall_collider = ColliderBuilder::cuboid(wall_thickness, BOUNDS.y);
    commands.spawn((right_wall, right_wall_collider));

    commands
        // left
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(vec3(-BOUNDS.x / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, BOUNDS.y + wall_thickness)),
            ..Default::default()
        })
        // right
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(vec3(BOUNDS.x / 2.0, 0.0, 0.0)),
            sprite: Sprite::new(Vec2::new(wall_thickness, BOUNDS.y + wall_thickness)),
            ..Default::default()
        })
        // bottom
        .spawn(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_translation(vec3(0.0, -BOUNDS.y / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(BOUNDS.x + wall_thickness, wall_thickness)),
            ..Default::default()
        })
        // top
        .spawn(SpriteBundle {
            material: wall_material,
            transform: Transform::from_translation(vec3(0.0, BOUNDS.y / 2.0, 0.0)),
            sprite: Sprite::new(Vec2::new(BOUNDS.x + wall_thickness, wall_thickness)),
            ..Default::default()
        });

    // physics
    physics_config.gravity = Vector::y() * -1000.;
}

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "marble bros".to_string(),
            width: BOUNDS[0],
            height: BOUNDS[1],
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(RapierPhysicsPlugin)
        .add_plugin(RapierRenderPlugin)
        .add_startup_system(setup.system())
        .add_system(ball_motion_system.system())
        .add_system(keyboard_input_system.system())
        .add_system(body_system.system())
        .run();
}

fn keyboard_input_system(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::A) {
        println!("'A' currently pressed");
    }

    if keyboard_input.just_pressed(KeyCode::A) {
        println!("'A' just pressed");
    }

    if keyboard_input.just_released(KeyCode::A) {
        println!("'A' just released");
    }
}

fn body_system(mut bodies_query: ResMut<RigidBodySet>) {
    for (handle, body) in bodies_query.iter_mut() {
        // debug!("{}", body.body_status == BodySt);
    }
}

fn ball_motion_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut bodies: ResMut<RigidBodySet>,
    colliders: Res<ColliderSet>
) {
    let x_dir = match (
        keyboard_input.pressed(KeyCode::Left),
        keyboard_input.pressed(KeyCode::Right),
    ) {
        (true, true) => None,
        (true, _) => Some(-1.),
        (_, true) => Some(1.),
        _ => None,
    };
    for (_, character_body) in bodies.iter_mut() {
        if character_body.user_data == 1 {
            x_dir.and_then(|x| {
                character_body.apply_force(Vector2::new(x * 1000000., 0.), true);
                Some(x)
            });
            let vel = character_body.linvel();
            let x_vel = vel[0];
            let abs_x_vel = f32::abs(x_vel);
            let mut next = Vector2::new(vel[0], vel[1]);
            if abs_x_vel > 300. {
              next[0] = if x_vel < 0. { -300. } else { 300. }
            }
            // jump
            if keyboard_input.just_pressed(KeyCode::Space) {
              // let can_jump = character_body.colliders().iter().find(|c| {
              //   if let Some(collider) = colliders.get(c) {
              //     collider.position()
              //   }
              // })
              // if can_jump {
              next[1] = next[1] + 200.;
            }
            character_body.set_linvel(next, false)

        }
    }
}
