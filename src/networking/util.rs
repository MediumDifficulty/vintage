use super::Byte;

pub fn to_angle_byte(n: f32) -> Byte {
    (n / std::f32::consts::TAU * 255.) as Byte
}

pub fn angle_to_f32(n: Byte) -> f32 {
    n as f32 / 255. * std::f32::consts::TAU
}
