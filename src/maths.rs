use glam::{IVec2, Mat4, Vec2, Vec3};

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
pub fn yolo_min<N: std::cmp::PartialOrd>(a: N, b: N) -> N {
    std::cmp::min_by(a, b, yolo_compare)
}

#[inline]
pub fn yolo_max<N: std::cmp::PartialOrd>(a: N, b: N) -> N {
    std::cmp::max_by(a, b, yolo_compare)
}

pub fn look_at_transform(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    let z = (eye - center).normalize();
    let x = up.cross(z).normalize();
    let y = z.cross(x).normalize();
    let mut minv = Mat4::IDENTITY;
    let mut tr = Mat4::IDENTITY;
    for i in 0..3 {
        minv.col_mut(i)[0] = x[i];
        minv.col_mut(i)[1] = y[i];
        minv.col_mut(i)[2] = z[i];
        tr.col_mut(3)[i] = -center[i];
    }
    minv * tr
}

pub const DEPTH_MAX: f32 = 255.0;

// viewport matrix resizes/repositions the result to fit on screen
pub fn viewport_transform(x: f32, y: f32, w: f32, h: f32) -> Mat4 {
    Mat4::from_cols(
        [w / 2.0, 0.0, 0.0, 0.0].into(),
        [0.0, h / 2.0, 0.0, 0.0].into(),
        [0.0, 0.0, DEPTH_MAX / 2.0, 0.0].into(),
        [x + w / 2.0, y + h / 2.0, DEPTH_MAX / 2.0, 1.0].into(),
    )
}
