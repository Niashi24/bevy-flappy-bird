use bevy::prelude::*;

pub const SCREEN_SCALE: f32 = 4.0;
pub const BASE_RESOLUTION: Vec2 = Vec2 { x: 144.0, y: 200.0 };

pub const PLAYER_SIZE: Vec2 = Vec2::new(17.0, 12.0);

pub const GRAVITY: f32 = -650.0;
pub const JUMP_VELOCITY: f32 = 150.0;

// pub const DEFAULT_AUDIO_SETTINGS: PlaybackSettings = PlaybackSettings {
//     volume: bevy::audio::Volume::Relative(VolumeLevel::new(0.1)),
//     ..PlaybackSettings::ONCE
// };

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Flappy Bird!".to_owned(),
                        resolution: (BASE_RESOLUTION * SCREEN_SCALE).into(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        // Tells wasm to resize the window according to the available canvas
                        fit_canvas_to_parent: false,
                        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(GlobalVolume::new(0.2))
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, (spawn_camera, spawn_background))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Game,
    GameOver,
}

pub fn spawn_camera(mut commands: Commands) {
    let xy = lerp_window((0.5, 0.5).into());
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.0 / SCREEN_SCALE,
            ..default()
        },
        transform: Transform::from_xyz(xy.x, xy.y, 0.0),
        ..default()
    });
}

fn spawn_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    let xy = lerp_window((0.5, 0.5).into());

    commands.spawn(SpriteBundle {
        transform: Transform::from_xyz(xy.x, xy.y, -1.0),
        texture: asset_server.load("sprites/city-background.png"),
        ..default()
    });
}

#[derive(Component)]
pub struct Player {
    y_vel: f32,
}

#[derive(Component)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FlapEvent>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, move_system)
            .add_systems(Update, flap_input_system)
            .add_systems(Update, player_flap_system.after(flap_input_system))
            .add_systems(Update, gravity_system.before(constrain_player_system))
            .add_systems(Update, constrain_player_system.before(move_system))
            .add_systems(Update, debug_on_press);
    }
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let xy = lerp_window((1.0 / 3.0, 0.5).into());
    println!("Spawned Player");

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(xy.x, xy.y, 0.0),
            texture: asset_server.load("sprites/bird-0.png"),
            ..default()
        },
        Player { y_vel: 0.0 },
    ));
}

pub fn gravity_system(mut query: Query<&mut Player>, time: Res<Time>) {
    if let Ok(mut player) = query.get_single_mut() {
        player.y_vel += GRAVITY * time.delta_seconds();
    }
}

fn move_system(mut query: Query<(&mut Transform, &Player)>, time: Res<Time>) {
    if let Ok((mut player_transform, player)) = query.get_single_mut() {
        player_transform.translation += Vec3::Y * player.y_vel * time.delta_seconds();
    }
}

#[derive(Event, Default)]
pub struct FlapEvent;

pub fn flap_input_system(
    query: Query<&Player>,
    mouse_input: Res<Input<MouseButton>>,
    mut event_writer: EventWriter<FlapEvent>,
) {
    // Player not in scene
    if query.get_single().is_err() {
        return;
    }

    if mouse_input.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        event_writer.send_default();
    }
}

pub fn player_flap_system(
    mut query: Query<&mut Player>,
    mut flap_event: EventReader<FlapEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if let Ok(mut player) = query.get_single_mut() {
        if flap_event.iter().any(|_| true) {
            player.y_vel = JUMP_VELOCITY;
            commands.spawn(AudioBundle {
                source: asset_server.load("audio/sfx_wing.ogg"),
                settings: PlaybackSettings::DESPAWN,
            });
        }
    }
}

pub fn constrain_player_system(mut query: Query<(&mut Player, &mut Transform)>) {
    if let Ok((mut player, mut transform)) = query.get_single_mut() {
        if transform.translation.y < -PLAYER_SIZE.y && player.y_vel < 0.0 {
            transform.translation.y = -PLAYER_SIZE.y;
            player.y_vel = 0.0;
        } else if transform.translation.y > BASE_RESOLUTION.y + PLAYER_SIZE.y && player.y_vel > 0.0
        {
            transform.translation.y = BASE_RESOLUTION.y + PLAYER_SIZE.y;
            player.y_vel = 0.0;
        }
    }
}

pub fn debug_on_press(query: Query<(&Transform, &Player)>, keyboard_input: Res<Input<KeyCode>>) {
    if let Ok((transform, player)) = query.get_single() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            println!("XYZ: {}, Y-Vel: {}", transform.translation, player.y_vel);
        }
    }
}

fn lerp_window(uv: Vec2) -> Vec2 {
    lerp_2d(BASE_RESOLUTION, uv)
}

fn lerp_2d(vec: Vec2, uv: Vec2) -> Vec2 {
    Vec2 {
        x: lerp(uv.x, 0.0, vec.x),
        y: lerp(uv.y, 0.0, vec.y),
    }
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a * (1.0 - t) + b * t
}
