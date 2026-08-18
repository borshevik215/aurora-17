use macroquad::audio::{load_sound, play_sound, stop_sound, PlaySoundParams, Sound};
use std::collections::HashMap;

pub struct AudioDirector {
    sounds: HashMap<String, Sound>,
    ambient_started: bool,
    siren_started: bool,
}

impl AudioDirector {
    pub async fn new() -> Self {
        let mut sounds = HashMap::new();
        for name in [
            "hum", "vent", "click", "relay", "beep", "glitch", "siren",
            "boot_power", "boot_scan", "relay_engage", "terminal_ready", "dock_clamp",
            "intro_wake", "intro_radio", "intro_hum", "intro_relay",
        ] {
            if let Ok(sound) = load_sound(&format!("assets/audio/{name}.wav")).await {
                sounds.insert(name.into(), sound);
            }
        }
        Self { sounds, ambient_started: false, siren_started: false }
    }

    fn play(&self, name: &str, volume: f32) {
        if let Some(sound) = self.sounds.get(name) {
            play_sound(sound, PlaySoundParams { looped: false, volume });
        }
    }

    pub fn start_ambient(&mut self, volume: f32) {
        if self.ambient_started { return; }
        if let Some(sound) = self.sounds.get("hum") {
            play_sound(sound, PlaySoundParams { looped: true, volume: 0.10 * volume });
        }
        if let Some(sound) = self.sounds.get("vent") {
            play_sound(sound, PlaySoundParams { looped: true, volume: 0.035 * volume });
        }
        self.ambient_started = true;
    }

    pub fn key_click(&self) { self.play("click", 0.045); }
    pub fn command(&self) { self.play("relay", 0.075); }
    pub fn beep(&self) { self.play("beep", 0.06); }
    pub fn boot_power(&self) { self.play("boot_power", 0.18); }
    pub fn boot_scan(&self) { self.play("boot_scan", 0.12); }
    pub fn relay_engage(&self) { self.play("relay_engage", 0.15); }
    pub fn terminal_ready(&self) { self.play("terminal_ready", 0.11); }
    pub fn dock_clamp(&self) { self.play("dock_clamp", 0.18); }

    // Cinematic intro palette: restrained low-frequency machine sounds, not alarms.
    pub fn intro_wake(&self) { self.play("intro_wake", 0.22); }
    pub fn intro_radio(&self) { self.play("intro_radio", 0.14); }
    pub fn intro_hum(&self) { self.play("intro_hum", 0.12); }
    pub fn intro_relay(&self) { self.play("intro_relay", 0.18); }

    pub fn start_siren(&mut self) {
        if self.siren_started { return; }
        if let Some(sound) = self.sounds.get("siren") {
            play_sound(sound, PlaySoundParams { looped: true, volume: 0.035 });
            self.siren_started = true;
        }
    }

    pub fn stop_siren(&mut self) {
        if !self.siren_started { return; }
        if let Some(sound) = self.sounds.get("siren") { stop_sound(sound); }
        self.siren_started = false;
    }

    pub fn anomaly(&self) { self.play("glitch", 0.11); }
}
