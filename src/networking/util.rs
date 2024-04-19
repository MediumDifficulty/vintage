use super::Byte;

pub fn to_angle_byte(n: f32) -> Byte {
    (n / (2. * std::f32::consts::TAU) * 255.) as Byte
}
