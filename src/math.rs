use druid::Color;
use glam::{Vec3, Mat3};

/// Edge function used in rasterization
#[inline(always)]
pub fn edge_function(a: &[f32; 2], b: &[f32; 2], c: &[f32; 2]) -> f32 {
    (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0])
}

/// Checks if a point is inside a triangle using the edge function with early exit
#[inline(always)]
pub fn point_in_triangle(p: &[f32; 2], a: &[f32; 2], b: &[f32; 2], c: &[f32; 2]) -> bool {
    let w0 = edge_function(b, c, p);
    if w0 < 0.0 {
        let w1 = edge_function(c, a, p);
        if w1 < 0.0 {
            let w2 = edge_function(a, b, p);
            return w2 < 0.0;
        }
        return false;
    } else {
        let w1 = edge_function(c, a, p);
        if w1 < 0.0 {
            return false;
        }
        let w2 = edge_function(a, b, p);
        return w2 >= 0.0;
    }
}

/// Multiplies a 3x3 matrix by a 3-dimensional vector using SIMD
#[inline(always)]
pub fn multiply_matrix_vector(matrix: &[[f32; 3]; 3], vector: &[f32; 3]) -> [f32; 3] {
    let mat = Mat3::from_cols_array_2d(matrix);
    let vec = Vec3::from_array(*vector);
    (mat * vec).to_array()
}

/// Multiplies two 3x3 matrices using SIMD
#[inline(always)]
pub fn multiply_matrices(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mat_a = Mat3::from_cols_array_2d(a);
    let mat_b = Mat3::from_cols_array_2d(b);
    (mat_a * mat_b).to_cols_array_2d()
}

/// Calculates the normal vector of a triangle using SIMD
#[inline(always)]
pub fn calculate_normal(a: &[f32; 3], b: &[f32; 3], c: &[f32; 3]) -> [f32; 3] {
    let va = Vec3::from_array(*a);
    let vb = Vec3::from_array(*b);
    let vc = Vec3::from_array(*c);
    
    let u = vb - va;
    let v = vc - va;
    
    u.cross(v).normalize().to_array()
}

/// Calculates the light intensity based on the normal vector and light position using SIMD
#[inline(always)]
pub fn calculate_light_intensity(
    normal: &[f32; 3],
    position: &[f32; 3],
    light_pos: &[f32; 3],
) -> f32 {
    let n = Vec3::from_array(*normal);
    let p = Vec3::from_array(*position);
    let l = Vec3::from_array(*light_pos);
    
    let light_dir = (l - p).normalize();
    n.dot(light_dir).max(0.1) // Ensure a minimum ambient light
}

/// Applies lighting to a color (optimized with bit manipulation)
#[inline(always)]
pub fn apply_lighting(color: &Color, intensity: f32) -> Color {
    let rgba = color.as_rgba8();
    let r = ((rgba.0 as f32 * intensity).min(255.0) + 0.5) as u8;
    let g = ((rgba.1 as f32 * intensity).min(255.0) + 0.5) as u8;
    let b = ((rgba.2 as f32 * intensity).min(255.0) + 0.5) as u8;
    Color::rgb8(r, g, b)
}

/// Vector normalization using SIMD
#[inline(always)]
pub fn normalize(v: &[f32; 3]) -> [f32; 3] {
    Vec3::from_array(*v).normalize().to_array()
}
