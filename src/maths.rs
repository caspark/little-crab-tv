use glam::{IVec2, Vec3};

pub(crate) fn barycentric_coords(pts: &[IVec2], p: IVec2) -> Vec3 {
    let u: Vec3 = Vec3::new(
        (pts[2][0] - pts[0][0]) as f32,
        (pts[1][0] - pts[0][0]) as f32,
        (pts[0][0] - p[0]) as f32,
    )
    .cross(Vec3::new(
        (pts[2][1] - pts[0][1]) as f32,
        (pts[1][1] - pts[0][1]) as f32,
        (pts[0][1] - p[1]) as f32,
    ));

    if u[2].abs() < 1.0 {
        return Vec3::new(-1.0, 1.0, 1.0);
    }
    return Vec3::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z);
}
