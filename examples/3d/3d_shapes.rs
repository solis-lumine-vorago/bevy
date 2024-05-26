//! This example demonstrates the built-in 3d shapes in Bevy.
//! The scene includes a patterned texture and a rotation for visualizing the normals and UVs.

use std::f32::consts::{PI, TAU};

use bevy::{
    color::palettes::css::{BLUE, GREEN, RED, WHITE},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<Axes>()
        .add_systems(Startup, setup)
        .add_systems(Update, (input, draw_gizmos))
        .run();
}

impl Projectable for ShapeProjection<Torus> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let minor_r = self.primitive.minor_radius;
        let major_r = self.primitive.major_radius;
        let local_y = (self.rotation * Vec3::Y).xy().normalize_or(Vec2::Y);
        let local_x = local_y.rotate(Vec2::NEG_Y);
        let dir = self.rotation.conjugate() * Vec3::NEG_Z;
        
        let semi_minor = dir.y.abs() * major_r;
        let mut segments = vec![
            PerimeterSegment {
                max_samples: None,
                sampler: Box::new(move |t: f32| {
                    let (sin, cos) = (TAU * t).sin_cos();
                    let normal = Vec2::new(semi_minor * cos, major_r * sin).normalize() * minor_r;
                    (cos * major_r + normal.x) * local_x - (sin * semi_minor + normal.y) * local_y
                }),
            },
        ];

        if semi_minor <= minor_r {
            return segments;
        }

        segments.push(
            PerimeterSegment {
                max_samples: None,
                sampler: Box::new(move |t: f32| {
                    let (sin, cos) = (TAU * t).sin_cos();
                    let normal = Vec2::new(semi_minor * cos, major_r * sin).normalize() * minor_r;
                    (cos * major_r - normal.x) * local_x - (sin * semi_minor - normal.y) * local_y
                }),
            }
        );
        segments 
    }
}
impl Projectable for ShapeProjection<Cuboid> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let mut points = [
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
        ].map(|p| (self.rotation * (p * self.primitive.half_size)).xy());
        points.sort_by(|a, b| (&a.to_angle()).total_cmp(&b.to_angle()));
        let mut final_positions = vec![];
        for i in 0..points.len() {
            let a = points[(i as i32 - 1).rem_euclid(points.len() as i32) as usize];
            let b = points[(i).rem_euclid(points.len())];
            let c = points[(i + 1).rem_euclid(points.len())];

            let ac = c - a;
            let n = Vec2::new(-ac.y, ac.x).normalize();
            if (n * b).element_sum() - (n*a).element_sum() > 0. {
                continue;
            }
            final_positions.push(b);
        };
        final_positions.push(final_positions[0]);

        vec![
            PerimeterSegment {
                max_samples: Some(final_positions.len()),
                sampler: Box::new(move |t: f32| {
                    let i = (t * final_positions.len() as f32 - 0.001) as usize;
                    final_positions[i]
                }),
            }
        ]
    }
}
impl Projectable for ShapeProjection<Tetrahedron> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let mut points = self.primitive.vertices.map(|p| (self.rotation * p).xy());
        points.sort_by(|a, b| (&a.to_angle()).total_cmp(&b.to_angle()));
        let mut final_positions = vec![];
        for i in 0..points.len() {
            let a = points[(i as i32 - 1).rem_euclid(points.len() as i32) as usize];
            let b = points[(i).rem_euclid(points.len())];
            let c = points[(i + 1).rem_euclid(points.len())];

            let ac = c - a;
            let n = Vec2::new(-ac.y, ac.x).normalize();
            if (n * b).element_sum() - (n*a).element_sum() > 0. {
                continue;
            }
            final_positions.push(b);
        };
        final_positions.push(final_positions[0]);

        vec![
            PerimeterSegment {
                max_samples: Some(final_positions.len()),
                sampler: Box::new(move |t: f32| {
                    let i = (t * final_positions.len() as f32 - 0.001) as usize;
                    final_positions[i]
                }),
            }
        ]
    }
}
impl Projectable for ShapeProjection<Cone> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let r = self.primitive.radius;
        let half_height = self.primitive.height / 2.;
        let local_y = (self.rotation * Vec3::Y).xy().normalize_or(Vec2::Y);
        let local_x = local_y.rotate(Vec2::NEG_Y);
        let dir = self.rotation.conjugate() * Vec3::NEG_Z;
        
        let semi_minor = dir.y.abs() * r;
        let y_offset = half_height * dir.xz().length();

        if semi_minor > 2. * y_offset {
            return vec![
                PerimeterSegment {
                    max_samples: None,
                    sampler: Box::new(move |t: f32| {
                        let (sin, cos) = (TAU * t).sin_cos();
                        cos * r * local_x - (sin * semi_minor + y_offset) * local_y
                    }),
                }
            ]
        }

        let intersect_x = r * (1. - (semi_minor / 2. / y_offset).powi(2)).sqrt();
        let intersect_y = semi_minor * (1. - (intersect_x / r).powi(2)).sqrt() - y_offset;
        let angle_offset = Vec2::new(semi_minor * intersect_x / r, intersect_y + y_offset).to_angle();
        let full_angle = PI + 2. * angle_offset;
        vec![
            PerimeterSegment {
                max_samples: None,
                sampler: Box::new(move |t: f32| {
                    let (sin, cos) = (full_angle * t - angle_offset).sin_cos();
                    cos * r * local_x - (sin * semi_minor + y_offset) * local_y
                }),
            },
            PerimeterSegment {
                max_samples: Some(3),
                sampler: Box::new(move |t: f32| {
                    if t < 0.33 {
                        intersect_x * local_x + intersect_y * local_y
                    } else if t < 0.666 {
                        y_offset * local_y
                    } else {
                        -intersect_x * local_x + intersect_y * local_y
                    }
                }),
            },
        ]
    }
}
impl Projectable for ShapeProjection<Capsule3d> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let r = self.primitive.radius;
        let half_height = self.primitive.half_length;
        let local_y = (self.rotation * Vec3::Y).xy().normalize_or(Vec2::Y);
        let local_x = local_y.rotate(Vec2::NEG_Y);
        let dir = self.rotation.conjugate() * Vec3::NEG_Z;

        let y_offset = half_height * dir.xz().length();
        vec![
            PerimeterSegment {
                max_samples: None,
                sampler: Box::new(move |t: f32| {
                    let (sin, cos) = (PI * t).sin_cos();
                    cos * r * local_x + (sin * r + y_offset) * local_y
                }),
            },
            PerimeterSegment {
                max_samples: None,
                sampler: Box::new(move |t: f32| {
                    let (sin, cos) = (PI * t).sin_cos();
                    cos * r * local_x - (sin * r + y_offset) * local_y
                }),
            },
            PerimeterSegment {
                max_samples: Some(2),
                sampler: Box::new(move |t: f32| {
                    if t < 0.5 {
                        r * local_x + y_offset * local_y
                    } else {
                        r * local_x - y_offset * local_y
                    }
                }),
            },
            PerimeterSegment {
                max_samples: Some(2),
                sampler: Box::new(move |t: f32| {
                    if t < 0.5 {
                        -r * local_x + y_offset * local_y
                    } else {
                        -r * local_x - y_offset * local_y
                    }
                }),
            },
        ]
    }
}
impl Projectable for ShapeProjection<Cylinder> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let r = self.primitive.radius;
        let half_height = self.primitive.half_height;
        let local_y = (self.rotation * Vec3::Y).xy().normalize_or(Vec2::Y);
        let local_x = local_y.rotate(Vec2::NEG_Y);
        let dir = self.rotation.conjugate() * Vec3::NEG_Z;

        let semi_minor = dir.y.abs() * r;
        let y_offset = half_height * dir.xz().length();
        vec![
            PerimeterSegment {
                max_samples: None,
                sampler: Box::new(move |t: f32| {
                    let (sin, cos) = (PI * t).sin_cos();
                    cos * r * local_x + (sin * semi_minor + y_offset) * local_y
                }),
            },
            PerimeterSegment {
                max_samples: None,
                sampler: Box::new(move |t: f32| {
                    let (sin, cos) = (PI * t).sin_cos();
                    cos * r * local_x - (sin * semi_minor + y_offset) * local_y
                }),
            },
            PerimeterSegment {
                max_samples: Some(2),
                sampler: Box::new(move |t: f32| {
                    if t < 0.5 {
                        r * local_x + y_offset * local_y
                    } else {
                        r * local_x - y_offset * local_y
                    }
                }),
            },
            PerimeterSegment {
                max_samples: Some(2),
                sampler: Box::new(move |t: f32| {
                    if t < 0.5 {
                        -r * local_x + y_offset * local_y
                    } else {
                        -r * local_x - y_offset * local_y
                    }
                }),
            },
        ]
    }
}
impl Projectable for ShapeProjection<Sphere> {
    fn perimeter(&self) -> Vec<PerimeterSegment> {
        let r = self.primitive.radius;
        vec![PerimeterSegment {
            max_samples: None,
            sampler: Box::new(move |t: f32| {
                let (sin, cos) = (TAU * t).sin_cos();
                Vec2::new(cos, sin) * r
            }),
        }]
    }
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape(usize);
/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Resource, Default)]
struct Axes(bool);

const X_EXTENT: f32 = 12.0;

fn draw_gizmos(shapes: Query<(&Transform, &Shape)>, mut gizmos: Gizmos, axes: Res<Axes>) {
    let mut first = true;
    for (t, Shape(i)) in shapes.iter() {
        if first {
            let dir = t.rotation.conjugate() * Vec3::NEG_Z;
            gizmos.line_2d(
                Vec2::Y * (3. + 0.0),
                Vec2::X * X_EXTENT / 2. * dir.x + Vec2::Y * (3. + 0.0),
                RED,
            );
            gizmos.line_2d(
                Vec2::Y * (3. + 0.1),
                Vec2::X * X_EXTENT / 2. * dir.y + Vec2::Y * (3. + 0.1),
                GREEN,
            );
            gizmos.line_2d(
                Vec2::Y * (3. + 0.2),
                Vec2::X * X_EXTENT / 2. * dir.z + Vec2::Y * (3. + 0.2),
                BLUE,
            );
            first = false;
        }
        
        if axes.0 {
            gizmos.axes(t.clone(), 1.);
        }
        
        let num_shapes = 7;
        let color = Color::hsl(360. * *i as f32 / num_shapes as f32, 0.95, 0.7);
        match *i {
            0 => gizmos.projection(
                ShapeProjection::new(Cylinder::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            1 => gizmos.projection(
                ShapeProjection::new(Capsule3d::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            2 => gizmos.projection(
                ShapeProjection::new(Sphere::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            3 => gizmos.projection(
                ShapeProjection::new(Cone::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            4 => gizmos.projection(
                ShapeProjection::new(Tetrahedron::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            5 => gizmos.projection(
                ShapeProjection::new(Cuboid::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            6 => gizmos.projection(
                ShapeProjection::new(Torus::default(), t.rotation),
                t.translation.xy(),
                color,
            ),
            _ => todo!()
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmo_config: ResMut<GizmoConfigStore>,
) {
    let (config, _) = gizmo_config.config_mut::<DefaultGizmoConfigGroup>();
    config.depth_bias = -1.;

    let debug_material = materials.add(StandardMaterial {
        base_color: WHITE.into(),
        ..default()
    });

    let shapes = [
        meshes.add(Cylinder::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
        meshes.add(Cone::default().mesh()),
        meshes.add(Tetrahedron::default().mesh()),
        meshes.add(Cuboid::default().mesh()),
        meshes.add(Torus::default().mesh()),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        let x = if num_shapes > 1 {
            -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT
        } else {
            0.
        };
        commands.spawn((
            PbrBundle {
                mesh: shape,
                material: debug_material.clone(),
                transform: Transform::from_xyz(x, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
            Shape(i),
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        projection: Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::AutoMax {
                max_width: 15.,
                max_height: 200.,
            },
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0., 12.0),
        ..default()
    });
}

fn input(
    mut query: Query<(&mut Transform, &mut Visibility), With<Shape>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut axes: ResMut<Axes>, 
) {
    let around_x = {
        let mut delta = 0.;
        if keys.pressed(KeyCode::KeyS) {
            delta += 1.;
        }
        if keys.pressed(KeyCode::KeyW) {
            delta -= 1.;
        }
        delta * time.delta_seconds()
    };
    let around_y = {
        let mut delta = 0.;
        if keys.pressed(KeyCode::KeyD) {
            delta += 1.;
        }
        if keys.pressed(KeyCode::KeyA) {
            delta -= 1.;
        }
        delta * time.delta_seconds()
    };
    let around_z = {
        let mut delta = 0.;
        if keys.pressed(KeyCode::KeyQ) {
            delta += 1.;
        }
        if keys.pressed(KeyCode::KeyE) {
            delta -= 1.;
        }
        delta * time.delta_seconds()
    };

    let reset = keys.just_pressed(KeyCode::KeyR);
    let toggle_visibility = keys.just_pressed(KeyCode::KeyV);

    axes.0 ^= keys.just_pressed(KeyCode::KeyC);

    for (mut transform, mut visibility) in &mut query {
        if reset {
            transform.rotation = Quat::IDENTITY;
        } else {
            transform.rotate_x(around_x);
            transform.rotate_y(around_y);
            transform.rotate_z(around_z);
        }

        if toggle_visibility {
            if *visibility == Visibility::Hidden {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

struct PerimeterSegment {
    max_samples: Option<usize>,
    sampler: Box<dyn Fn(f32) -> Vec2>,
}

trait Projectable {
    fn perimeter(&self) -> Vec<PerimeterSegment>;
}

struct ShapeProjection<P: Primitive3d>
where
    ShapeProjection<P>: Projectable,
{
    primitive: P,
    rotation: Quat,
}
impl<P: Primitive3d> ShapeProjection<P>
where
    ShapeProjection<P>: Projectable,
{
    fn new(primitive: P, rotation: Quat) -> Self {
        Self {
            primitive,
            rotation,
        }
    }
}

trait GizmoProjection<P: Primitive3d>
where
    ShapeProjection<P>: Projectable,
{
    fn projection(&mut self, projection: ShapeProjection<P>, position: Vec2, color: Color);
}
impl<'w, 's, Config, Clear, P: Primitive3d> GizmoProjection<P> for Gizmos<'w, 's, Config, Clear>
where
    ShapeProjection<P>: Projectable,
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    fn projection(&mut self, projection: ShapeProjection<P>, position: Vec2, color: Color) {
        const DEFAULT_SAMPLES: usize = 96;

        for segment in projection.perimeter() {
            let samples = segment.max_samples.unwrap_or(DEFAULT_SAMPLES);
            let mut linestrip = vec![];
            for i in 0..samples {
                let t = i as f32 / (samples as f32 - 1.);
                linestrip.push((segment.sampler)(t) + position);
            }

            self.linestrip_2d(linestrip, color);
        }
    }
}
