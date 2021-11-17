use glam::{IVec2, Vec2, Vec3};

pub(crate) fn barycentric_coords_2d(pts: &[IVec2], p: IVec2) -> Vec3 {
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

pub(crate) fn barycentric_coords_3d(pts: &[Vec3], p: Vec2) -> Vec3 {
    let u: Vec3 = Vec3::new(
        pts[2][0] - pts[0][0],
        pts[1][0] - pts[0][0],
        pts[0][0] - p[0],
    )
    .cross(Vec3::new(
        pts[2][1] - pts[0][1],
        pts[1][1] - pts[0][1],
        pts[0][1] - p[1],
    ));

    if u[2].abs() < 1.0 {
        return Vec3::new(-1.0, 1.0, 1.0);
    }
    return Vec3::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z);
}

#[inline]
pub(crate) fn yolo_compare<N: std::cmp::PartialOrd>(a: &N, b: &N) -> std::cmp::Ordering {
    a.partial_cmp(&b).expect("hopefully a and b are comparable")
}

#[inline]
pub(crate) fn yolo_min<N: std::cmp::PartialOrd>(a: N, b: N) -> N {
    std::cmp::min_by(a, b, yolo_compare)
}

#[inline]
pub(crate) fn yolo_max<N: std::cmp::PartialOrd>(a: N, b: N) -> N {
    std::cmp::max_by(a, b, yolo_compare)
}
