use bevy::prelude::*;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use std::fs::File;
use std::io::{BufRead, BufReader};

use mdi::{Mdi, Role, Method, Communicator, DataType, MdiData, Error as MdiError};
use std::ffi::{CStr, CString};

/// Atom data parsed from XYZ file
#[derive(Debug, Clone)]
struct Atom {
    element: String,
    position: Vec3,
}

/// Resource holding molecular data
#[derive(Resource)]
struct Molecule {
    atoms: Vec<Atom>,
}

/// Marker component for the molecule parent entity
#[derive(Component)]
struct MoleculeRoot;

/// Camera orbit controller (VMD-style)
#[derive(Resource)]
struct CameraController {
    distance: f32,
    rotation: Quat,
    target: Vec3,
    rotate_sensitivity: f32,
    pan_speed: f32,
    zoom_speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            distance: 15.0,
            rotation: Quat::from_rotation_x(-0.3),
            target: Vec3::ZERO,
            rotate_sensitivity: 0.005,
            pan_speed: 5.0,
            zoom_speed: 1.0,
        }
    }
}

fn main() {
    let molecule = parse_xyz("water_dimer.xyz").expect("Failed to parse XYZ file");


    // Parse command line arguments to find -mdi option
    let args: Vec<String> = std::env::args().collect();
    let mut mdi_options: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if args[i] == "--mdi" && i + 1 < args.len() {
            mdi_options = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
    }

    let options = mdi_options.expect("Must provide -mdi option");
    //let c_options = CString::new(options).expect("Invalid options string");

    /*
    let ret = unsafe { Mdi::init_with_options(c_options.as_ptr()) };
    if ret != 0 {
        panic!("MDI_Init_with_options failed");
    }
    */
    Mdi::init_with_options(&options);


    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(molecule)
        .insert_resource(CameraController::default())
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_rotation, camera_pan, camera_zoom, update_camera))
        .run();
}

fn parse_xyz(path: &str) -> Result<Molecule, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // First line: number of atoms
    let num_atoms: usize = lines
        .next()
        .ok_or("Missing atom count")??
        .trim()
        .parse()?;

    // Second line: comment (skip)
    lines.next();

    // Parse atom lines
    let mut atoms = Vec::with_capacity(num_atoms);
    for line in lines.take(num_atoms) {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("invalid atom line".into());
        }
        let element = parts[0].to_string();
        let x: f32 = parts[1].parse()?;
        let y: f32 = parts[2].parse()?;
        let z: f32 = parts[3].parse()?;
        atoms.push(Atom {
            element,
            position: Vec3::new(x, y, z),
        });
    }

    Ok(Molecule { atoms })
}

/// CPK coloring scheme for atoms
fn get_atom_color(element: &str) -> Color {
    match element.to_uppercase().as_str() {
        "H" => Color::srgb(1.0, 1.0, 1.0),        // White
        "C" => Color::srgb(0.3, 0.3, 0.3),        // Dark gray
        "N" => Color::srgb(0.2, 0.2, 1.0),        // Blue
        "O" => Color::srgb(1.0, 0.2, 0.2),        // Red
        "S" => Color::srgb(1.0, 1.0, 0.2),        // Yellow
        "P" => Color::srgb(1.0, 0.5, 0.0),        // Orange
        "F" | "CL" => Color::srgb(0.2, 1.0, 0.2), // Green
        "BR" => Color::srgb(0.6, 0.1, 0.1),       // Dark red
        "I" => Color::srgb(0.4, 0.0, 0.7),        // Purple
        "FE" => Color::srgb(0.9, 0.5, 0.0),       // Orange
        "CA" => Color::srgb(0.2, 0.8, 0.2),       // Green
        "MG" => Color::srgb(0.0, 0.5, 0.0),       // Dark green
        "ZN" => Color::srgb(0.5, 0.5, 0.6),       // Slate gray
        _ => Color::srgb(1.0, 0.5, 1.0),          // Pink for unknown
    }
}

/// Van der Waals radii (scaled for visualization)
fn get_atom_radius(element: &str) -> f32 {
    let scale = 0.4;
    let radius = match element.to_uppercase().as_str() {
        "H" => 1.20,
        "C" => 1.70,
        "N" => 1.55,
        "O" => 1.52,
        "S" => 1.80,
        "P" => 1.80,
        "F" => 1.47,
        "CL" => 1.75,
        "BR" => 1.85,
        "I" => 1.98,
        "FE" => 2.00,
        "CA" => 2.31,
        "MG" => 1.73,
        "ZN" => 1.39,
        _ => 1.50,
    };
    radius * scale
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    molecule: Res<Molecule>,
    mut controller: ResMut<CameraController>,
) {
    // Calculate molecule center for initial camera target
    let center = if !molecule.atoms.is_empty() {
        molecule
            .atoms
            .iter()
            .map(|a| a.position)
            .reduce(|a, b| a + b)
            .unwrap()
            / molecule.atoms.len() as f32
    } else {
        Vec3::ZERO
    };

    controller.target = center;

    // Create molecule parent entity
    let molecule_root = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            MoleculeRoot,
        ))
        .id();

    // Create atoms as spheres
    for atom in &molecule.atoms {
        let color = get_atom_color(&atom.element);
        let radius = get_atom_radius(&atom.element);

        let atom_entity = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(radius))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    perceptual_roughness: 0.5,
                    metallic: 0.1,
                    ..default()
                })),
                Transform::from_translation(atom.position),
            ))
            .id();

        commands.entity(molecule_root).add_child(atom_entity);
    }

    // Point light
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 10.0, 10.0),
    ));

    // Ambient light
    commands.spawn(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
        ..default()
    });

    // Camera
    let camera_pos = calculate_camera_position(&controller, controller.target);
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_pos).with_rotation(controller.rotation),
    ));

    println!("Molecular Viewer Controls:");
    println!("  Left mouse drag: Rotate view");
    println!("  Scroll wheel: Zoom in/out");
    println!("  Arrow keys: Pan view");
    println!("\nLoaded {} atoms", molecule.atoms.len());
}

fn calculate_camera_position(controller: &CameraController, target: Vec3) -> Vec3 {
    let direction = controller.rotation * Vec3::Z;
    target + direction * controller.distance
}

fn camera_rotation(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut controller: ResMut<CameraController>,
) {
    // VMD-style: left mouse button for rotation
    if mouse_button.pressed(MouseButton::Left) {
        let delta = mouse_motion.delta;

        // Rotate around camera's local Y axis for horizontal movement
        let up = controller.rotation * Vec3::Y;
        let yaw = Quat::from_axis_angle(up, -delta.x * controller.rotate_sensitivity);

        // Rotate around camera's local X axis for vertical movement
        let right = controller.rotation * Vec3::X;
        let pitch = Quat::from_axis_angle(right, -delta.y * controller.rotate_sensitivity);

        controller.rotation = (pitch * yaw * controller.rotation).normalize();
    }
}

fn camera_pan(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut controller: ResMut<CameraController>,
) {
    let mut pan = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        pan.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        pan.x += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        pan.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        pan.y -= 1.0;
    }

    if pan != Vec3::ZERO {
        // Transform pan direction based on camera orientation
        let right = controller.rotation * Vec3::X;
        let up = controller.rotation * Vec3::Y;
        let pan_delta = (right * pan.x + up * pan.y) * controller.pan_speed * time.delta_secs();
        controller.target += pan_delta;
    }
}

fn camera_zoom(
    scroll: Res<AccumulatedMouseScroll>,
    mut controller: ResMut<CameraController>,
) {
    controller.distance -= scroll.delta.y * controller.zoom_speed;
    controller.distance = controller.distance.clamp(2.0, 100.0);
}

fn update_camera(
    controller: Res<CameraController>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    for mut transform in camera_query.iter_mut() {
        let pos = calculate_camera_position(&controller, controller.target);
        transform.translation = pos;
        transform.rotation = controller.rotation;
    }
}
