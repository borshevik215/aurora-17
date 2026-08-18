pub mod navigation;

#[derive(Clone)]
pub struct Ship {
    pub power:u8, pub fuel:u8, pub hull:u8, pub navigation_calibrated:bool, pub course_locked:bool, pub mission_complete:bool,
    pub engine_temperature:f32, pub engine_pressure:f32, pub power_load:f32, pub life_pressure:f32, pub sensor_noise:f32,
    pub reactor_cooling:f32, pub life_support_integrity:f32, pub power_bus_integrity:f32, pub engine_integrity:f32,
}
impl Ship { pub fn new()->Self{Self{power:100,fuel:98,hull:100,navigation_calibrated:false,course_locked:false,mission_complete:false,engine_temperature:42.0,engine_pressure:100.0,power_load:54.0,life_pressure:1.0,sensor_noise:0.0,reactor_cooling:100.0,life_support_integrity:100.0,power_bus_integrity:100.0,engine_integrity:100.0}} }
