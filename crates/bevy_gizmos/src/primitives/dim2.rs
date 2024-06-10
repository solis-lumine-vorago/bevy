//! A module for rendering each of the 2D [`bevy_math::primitives`] with [`Gizmos`].

use std::f32::consts::{FRAC_PI_2, PI, SQRT_2, TAU};

use super::helpers::*;

use bevy_color::Color;
use bevy_math::primitives::{
    Annulus, Arc2d, BoxedPolygon, BoxedPolyline2d, Capsule2d, Circle, CircularSector,
    CircularSegment, Ellipse, Line2d, Plane2d, Polygon, Polyline2d, Primitive2d, Rectangle,
    RegularPolygon, Rhombus, Segment2d, Triangle2d,
};
use bevy_math::{Dir2, Vec2};

use crate::arcs::arc_2d_inner;
use crate::circles::DEFAULT_CIRCLE_RESOLUTION;
use crate::prelude::{GizmoConfigGroup, Gizmos};

// some magic number since using directions as offsets will result in lines of length 1 pixel
const MIN_LINE_LEN: f32 = 50.0;
// length used to simulate infinite lines
const INFINITE_LEN: f32 = 100_000.0;

/// A trait for `Primitive2d`s that can be drawn using gizmos.
pub trait GizmoPrimitive2d<'a>: Primitive2d + 'a {
    /// The output of [`Self::gizmos`]. This will be a [`GizmoBuilder2d`] used for drawing the gizmos.
    type Output: GizmoBuilder2d
    where
        Self: 'a;

    /// Creates a [`GizmoBuilder2d`] that can be used to draw gizmos representing this primitive.
    fn gizmos(&'a self) -> Self::Output;
}

/// A trait used to draw gizmos from a configuration.
pub trait GizmoBuilder2d {
    /// Get the linestrips representing the gizmos of this shape.
    ///
    /// You can assume that the shape is not rotated and positioned at `Vec2::ZERO`.
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>>;
}

/// A builder for drawing [`Primitive2d`]s in 2D returned by [`Gizmos::primitive_2d`].
pub struct GizmoPrimitive2dBuilder<'a, 'w, 's, P, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
    P: GizmoPrimitive2d<'a>,
{
    /// The underlying builder for the primitive.
    pub builder: P::Output,
    gizmos: &'a mut Gizmos<'w, 's, Config, Clear>,
    position: Vec2,
    angle: f32,
    color: Color,
}

impl<'w, 's, Config, Clear> Gizmos<'w, 's, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
{
    /// Renders a 2D primitive with its associated details.
    pub fn primitive_2d<'a, P: GizmoPrimitive2d<'a>>(
        &'a mut self,
        primitive: &'a P,
        position: Vec2,
        angle: f32,
        color: impl Into<Color>,
    ) -> GizmoPrimitive2dBuilder<'a, 'w, 's, P, Config, Clear> {
        GizmoPrimitive2dBuilder {
            builder: primitive.gizmos(),
            gizmos: self,
            position,
            angle,
            color: color.into(),
        }
    }
}

impl<'a, 'w, 's, Config, Clear, P> Drop for GizmoPrimitive2dBuilder<'a, 'w, 's, P, Config, Clear>
where
    Config: GizmoConfigGroup,
    Clear: 'static + Send + Sync,
    P: GizmoPrimitive2d<'a>,
{
    fn drop(&mut self) {
        if !self.gizmos.enabled {
            return;
        }

        let transform = rotate_then_translate_2d(self.angle, self.position);
        for linestrip in self.builder.linestrips() {
            self.gizmos
                .linestrip_2d(linestrip.into_iter().map(&transform), self.color);
        }
    }
}

// direction 2d

/// Builder for configuring the drawing options of [`Dir2`].
pub struct Dir2Builder {
    direction: Dir2,
}

impl GizmoBuilder2d for Dir2Builder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        arrow_2d(Vec2::ZERO, self.direction * MIN_LINE_LEN)
    }
}

impl<'a> GizmoPrimitive2d<'a> for Dir2 {
    type Output = Dir2Builder;

    fn gizmos(&self) -> Self::Output {
        Dir2Builder { direction: *self }
    }
}

// arc 2d

enum ArcKind {
    Arc,
    Sector,
    Segment,
}

/// Builder for configuring the drawing options of [`Arc2d`], [`CircularSector`] and [`CircularSegment`].
pub struct Arc2dBuilder {
    arc: Arc2d,
    arc_kind: ArcKind,
    resolution: Option<usize>,
}

impl Arc2dBuilder {
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution.replace(resolution);
        self
    }
}

impl GizmoBuilder2d for Arc2dBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        let resolution = self.resolution.unwrap_or_else(|| {
            ((self.arc.half_angle.abs() / PI) * DEFAULT_CIRCLE_RESOLUTION as f32).ceil() as usize
        });

        let mut arc_positions = {
            let mut positions = Vec::with_capacity(resolution + 2);
            let delta = 2.0 * self.arc.half_angle / (resolution as f32);
            let start = FRAC_PI_2 - self.arc.half_angle;
            positions.extend((0..resolution + 1).map(|i| {
                let angle = start + (i as f32 * delta);
                let (sin, cos) = angle.sin_cos();
                Vec2::new(cos, sin) * self.arc.radius
            }));

            positions
        };

        match self.arc_kind {
            ArcKind::Arc => {}
            ArcKind::Sector => {
                arc_positions.push(Vec2::ZERO);
                arc_positions.push(arc_positions[0]);
            }
            ArcKind::Segment => {
                arc_positions.push(arc_positions[0]);
            }
        };

        [arc_positions]
    }
}

impl<'a> GizmoPrimitive2d<'a> for Arc2d {
    type Output = Arc2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Arc2dBuilder {
            arc: *self,
            arc_kind: ArcKind::Arc,
            resolution: None,
        }
    }
}

// circular sector 2d

impl<'a> GizmoPrimitive2d<'a> for CircularSector {
    type Output = Arc2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Arc2dBuilder {
            arc: self.arc,
            arc_kind: ArcKind::Sector,
            resolution: None,
        }
    }
}

// circular segment 2d

impl<'a> GizmoPrimitive2d<'a> for CircularSegment {
    type Output = Arc2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Arc2dBuilder {
            arc: self.arc,
            arc_kind: ArcKind::Segment,
            resolution: None,
        }
    }
}

// circle 2d

impl<'a> GizmoPrimitive2d<'a> for Circle {
    type Output = EllipseBuilder;

    fn gizmos(&self) -> Self::Output {
        EllipseBuilder {
            half_size: Vec2::splat(self.radius),
            resolution: DEFAULT_CIRCLE_RESOLUTION,
        }
    }
}

// ellipse 2d

/// Builder for configuring the drawing options of [`Ellipse`].
pub struct EllipseBuilder {
    half_size: Vec2,
    resolution: usize,
}

impl EllipseBuilder {
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = resolution;
        self
    }
}

impl GizmoBuilder2d for EllipseBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        [ellipse_inner(self.half_size, self.resolution).collect()]
    }
}

impl<'a> GizmoPrimitive2d<'a> for Ellipse {
    type Output = EllipseBuilder;

    fn gizmos(&self) -> Self::Output {
        EllipseBuilder {
            half_size: self.half_size,
            resolution: DEFAULT_CIRCLE_RESOLUTION,
        }
    }
}

// annulus 2d

/// Builder for configuring the drawing options of [`Annulus`].
pub struct AnnulusBuilder {
    inner_radius: f32,
    outer_radius: f32,
    inner_resolution: usize,
    outer_resolution: usize,
}

impl AnnulusBuilder {
    /// Set the number of line-segments for each circle of the annulus.
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.outer_resolution = resolution;
        self.inner_resolution = resolution;
        self
    }

    /// Set the number of line-segments for the outer circle of the annulus.
    pub fn outer_resolution(mut self, resolution: usize) -> Self {
        self.outer_resolution = resolution;
        self
    }

    /// Set the number of line-segments for the inner circle of the annulus.
    pub fn inner_resolution(mut self, resolution: usize) -> Self {
        self.inner_resolution = resolution;
        self
    }
}

impl GizmoBuilder2d for AnnulusBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        let inner_positions =
            ellipse_inner(Vec2::splat(self.inner_radius), self.inner_resolution).collect();
        let outer_positions =
            ellipse_inner(Vec2::splat(self.outer_radius), self.outer_resolution).collect();

        [inner_positions, outer_positions]
    }
}

impl<'a> GizmoPrimitive2d<'a> for Annulus {
    type Output = AnnulusBuilder;

    fn gizmos(&self) -> Self::Output {
        AnnulusBuilder {
            inner_radius: self.inner_circle.radius,
            outer_radius: self.outer_circle.radius,
            inner_resolution: DEFAULT_CIRCLE_RESOLUTION,
            outer_resolution: DEFAULT_CIRCLE_RESOLUTION,
        }
    }
}

// rhombus 2d

/// Builder for configuring the drawing options of [`Rhombus`].
pub struct RhombusBuilder {
    half_diagonals: Vec2,
}

impl GizmoBuilder2d for RhombusBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        [vec![
            Vec2::new(self.half_diagonals.x, 0.0),
            Vec2::new(0.0, self.half_diagonals.y),
            Vec2::new(-self.half_diagonals.x, 0.0),
            Vec2::new(0.0, -self.half_diagonals.y),
            Vec2::new(self.half_diagonals.x, 0.0),
        ]]
    }
}

impl<'a> GizmoPrimitive2d<'a> for Rhombus {
    type Output = RhombusBuilder;

    fn gizmos(&self) -> Self::Output {
        RhombusBuilder {
            half_diagonals: self.half_diagonals,
        }
    }
}

// capsule 2d

/// Builder for configuring the drawing options of [`Capsule2d`].
pub struct Capsule2dBuilder {
    radius: f32,
    half_length: f32,
    resolution: usize,
}

impl GizmoBuilder2d for Capsule2dBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        let arc_points: Vec<Vec2> =
            arc_2d_inner(FRAC_PI_2, FRAC_PI_2, self.radius, self.resolution).collect();
        let mut positions = Vec::with_capacity(2 * self.resolution + 3);
        positions.extend(
            arc_points
                .iter()
                .map(|p| Vec2::new(0.0, self.half_length) + *p),
        );
        positions.extend(
            arc_points
                .iter()
                .map(|p| Vec2::new(0.0, -self.half_length) - *p),
        );
        positions.push(positions[0]);

        [positions]
    }
}

impl<'a> GizmoPrimitive2d<'a> for Capsule2d {
    type Output = Capsule2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Capsule2dBuilder {
            radius: self.radius,
            half_length: self.half_length,
            resolution: DEFAULT_CIRCLE_RESOLUTION,
        }
    }
}

// line 2d

/// Builder for configuring the drawing options of [`Line2d`].
pub struct Line2dBuilder {
    direction: Dir2, // Direction of the line

    draw_arrow: bool, // decides whether to indicate the direction of the line with an arrow
}

impl Line2dBuilder {
    /// Set the drawing mode of the line (arrow vs. plain line)
    pub fn draw_arrow(mut self, is_enabled: bool) -> Self {
        self.draw_arrow = is_enabled;
        self
    }
}

impl GizmoBuilder2d for Line2dBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        let start = self.direction * INFINITE_LEN;
        let end = -start;

        if self.draw_arrow {
            vec![
                vec![start, end],
                arrow_head(Vec2::ZERO, self.direction, MIN_LINE_LEN),
            ]
        } else {
            vec![vec![start, end]]
        }
    }
}

impl<'a> GizmoPrimitive2d<'a> for Line2d {
    type Output = Line2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Self::Output {
            direction: self.direction,
            draw_arrow: false,
        }
    }
}
// plane 2d

/// Builder for configuring the drawing options of [`Plane2d`].
pub struct Plane2dBuilder {
    normal: Dir2,
}

impl GizmoBuilder2d for Plane2dBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        let line_dir = Dir2::new_unchecked(-self.normal.perp());

        // The normal of the plane (orthogonal to the plane itself)
        let mut linestrips = arrow_2d(Vec2::ZERO, self.normal * MIN_LINE_LEN);
        // The plane line
        linestrips.push(vec![line_dir * INFINITE_LEN, -line_dir * INFINITE_LEN]);

        // An arrow such that the normal is always left side of the plane with respect to the
        // planes direction. This is to follow the "counter-clockwise" convention
        linestrips.push(arrow_head(
            line_dir * MIN_LINE_LEN,
            line_dir,
            MIN_LINE_LEN / 10.,
        ));

        linestrips
    }
}

impl<'a> GizmoPrimitive2d<'a> for Plane2d {
    type Output = Plane2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Self::Output {
            normal: self.normal,
        }
    }
}

// segment 2d

/// Builder for configuring the drawing options of [`Segment2d`].
pub struct Segment2dBuilder {
    direction: Dir2,  // Direction of the line segment
    half_length: f32, // Half-length of the line segment

    draw_arrow: bool, // decides whether to draw just a line or an arrow
}

impl Segment2dBuilder {
    /// Set the drawing mode of the line (arrow vs. plain line)
    pub fn draw_arrow(mut self, is_enabled: bool) -> Self {
        self.draw_arrow = is_enabled;
        self
    }
}

impl GizmoBuilder2d for Segment2dBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        let end = self.direction.as_vec2() * self.half_length;
        let start = -end;
        if self.draw_arrow {
            arrow_2d(start, end)
        } else {
            let segment = vec![start, end];
            vec![segment]
        }
    }
}

impl<'a> GizmoPrimitive2d<'a> for Segment2d {
    type Output = Segment2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Self::Output {
            direction: self.direction,
            half_length: self.half_length,
            draw_arrow: false,
        }
    }
}

// polyline 2d

/// Builder for configuring the drawing options of [`Polyline2d<N>`] and [`BoxedPolyline2d`].
pub struct Polyline2dBuilder<'a> {
    vertices: &'a [Vec2],
}

impl<'a> GizmoBuilder2d for Polyline2dBuilder<'a> {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        match self.vertices.len() {
            0 | 1 => [vec![]],
            _ => [self.vertices.into()],
        }
    }
}

impl<'a, const N: usize> GizmoPrimitive2d<'a> for Polyline2d<N> {
    type Output = Polyline2dBuilder<'a>;

    fn gizmos(&'a self) -> Self::Output {
        Self::Output {
            vertices: &self.vertices,
        }
    }
}

// boxed polyline 2d

impl<'a> GizmoPrimitive2d<'a> for BoxedPolyline2d {
    type Output = Polyline2dBuilder<'a>;

    fn gizmos(&'a self) -> Self::Output {
        Self::Output {
            vertices: &self.vertices,
        }
    }
}

// triangle 2d

/// Builder for configuring the drawing options of [`Triangle2d`].
pub struct Triangle2dBuilder {
    vertices: [Vec2; 3],
}

impl GizmoBuilder2d for Triangle2dBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        [vec![
            self.vertices[0],
            self.vertices[1],
            self.vertices[2],
            self.vertices[0],
        ]]
    }
}

impl<'a> GizmoPrimitive2d<'a> for Triangle2d {
    type Output = Triangle2dBuilder;

    fn gizmos(&self) -> Self::Output {
        Triangle2dBuilder {
            vertices: self.vertices,
        }
    }
}

// rectangle 2d

/// Builder for configuring the drawing options of [`Rectangle`].
pub struct RectangleBuilder {
    half_size: Vec2,
}

impl GizmoBuilder2d for RectangleBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        [vec![
            self.half_size,
            Vec2::new(self.half_size.x, -self.half_size.y),
            Vec2::new(-self.half_size.x, -self.half_size.y),
            Vec2::new(-self.half_size.x, self.half_size.y),
            self.half_size,
        ]]
    }
}

impl<'a> GizmoPrimitive2d<'a> for Rectangle {
    type Output = RectangleBuilder;

    fn gizmos(&self) -> Self::Output {
        RectangleBuilder {
            half_size: self.half_size,
        }
    }
}

// polygon 2d

/// Builder for configuring the drawing options of [`Polygon<N>`] and [`BoxedPolygon`].
pub struct PolygonBuilder<'a> {
    vertices: &'a [Vec2],
}

impl<'a> GizmoBuilder2d for PolygonBuilder<'a> {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        match self.vertices.len() {
            0 | 1 => [vec![]],
            2 => [self.vertices.into()],
            _ => [self
                .vertices
                .iter()
                .copied()
                .chain([self.vertices[0]])
                .collect()],
        }
    }
}

impl<'a, const N: usize> GizmoPrimitive2d<'a> for Polygon<N> {
    type Output = PolygonBuilder<'a>;

    fn gizmos(&'a self) -> Self::Output {
        PolygonBuilder {
            vertices: &self.vertices,
        }
    }
}

// boxed polygon 2d

impl<'a> GizmoPrimitive2d<'a> for BoxedPolygon {
    type Output = PolygonBuilder<'a>;

    fn gizmos(&'a self) -> Self::Output {
        PolygonBuilder {
            vertices: &self.vertices,
        }
    }
}

// regular polygon 2d

/// Builder for configuring the drawing options of [`RegularPolygon`].
pub struct RegularPolygonBuilder {
    circumradius: f32,
    sides: usize,
}

impl GizmoBuilder2d for RegularPolygonBuilder {
    fn linestrips(&self) -> impl IntoIterator<Item = Vec<Vec2>> {
        let points = (0..=self.sides)
            .map(|p| single_circle_coordinate(self.circumradius, self.sides, p))
            .collect();

        [points]
    }
}

impl<'a> GizmoPrimitive2d<'a> for RegularPolygon {
    type Output = RegularPolygonBuilder;

    fn gizmos(&self) -> Self::Output {
        RegularPolygonBuilder {
            circumradius: self.circumradius(),
            sides: self.sides,
        }
    }
}

fn arrow_2d(start: Vec2, end: Vec2) -> Vec<Vec<Vec2>> {
    let segment = vec![start, end];

    let tip_length = (end - start).length() / 10.;
    let direction = Dir2::new_unchecked((end - start).normalize());

    vec![segment, arrow_head(end, direction, tip_length)]
}

fn arrow_head(position: Vec2, direction: Dir2, tip_length: f32) -> Vec<Vec2> {
    let left_offset = direction.rotate(Vec2::new(-SQRT_2, SQRT_2)) * tip_length;
    let right_offset = direction.rotate(Vec2::new(-SQRT_2, -SQRT_2)) * tip_length;

    vec![position + left_offset, position, position + right_offset]
}

fn ellipse_inner(half_size: Vec2, resolution: usize) -> impl Iterator<Item = Vec2> {
    (0..resolution + 1).map(move |i| {
        let angle = i as f32 * TAU / resolution as f32;
        let (x, y) = angle.sin_cos();
        Vec2::new(x, y) * half_size
    })
}
