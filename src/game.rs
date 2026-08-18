use macroquad::prelude::*;
use macroquad::camera::Camera2D;
use macroquad::material::{
    load_material,
    Material,
    MaterialParams,
};

use crate::{
    audio::AudioDirector,
    commands::execute,
    engine::{
        events::{
            Event,
            EventKind,
        },
        scheduler::EventScheduler,
        state::GameState,
    },
    ship::Ship,
    terminal::Terminal,
};

// Фиксированный размер offscreen target — логическая единица игры
const TARGET_W: f32 = 1280.0;
const TARGET_H: f32 = 720.0;

pub struct Game {
    // One-shot cinematic intro + staged terminal boot.
    intro_active: bool,
    intro_elapsed: f32,
    intro_stage: usize,
    boot_sequence_active: bool,
    boot_stage: usize,
    boot_stage_timer: f32,

    terminal: Terminal,
    ship: Ship,
    state: GameState,
    scheduler: EventScheduler,
    audio: AudioDirector,

    target: RenderTarget,
    crt: Material,

    elapsed: f32,
    tracking_timer: f32,
    booted: bool,
    boot_wait_timer: f32,
    last_click_time: f32,
    critical_alert: bool,
    critical_reboot_pending: bool,
}

impl Game {
    pub async fn new(seed: u64) -> Self {
        let terminal = Terminal::new();

        let mut audio = AudioDirector::new().await;

        let target = render_target(TARGET_W as u32, TARGET_H as u32);

        let crt = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                ..Default::default()
            },
        )
        .expect("Failed to create CRT material");

        Self {
            intro_active: true,
            intro_elapsed: 0.0,
            intro_stage: 0,
            boot_sequence_active: false,
            boot_stage: 0,
            boot_stage_timer: 0.0,
            // core components
            terminal,
            ship: Ship::new(),
            state: GameState::new(seed),
            scheduler: EventScheduler::new(),
            audio,
            target,
            crt,
            elapsed: 0.0,
            tracking_timer: 0.0,
            booted: false,
            boot_wait_timer: 0.0,
            last_click_time: 0.0,
            critical_alert: false,
            critical_reboot_pending: false,
        }
    }

    pub fn update(&mut self) {
        let dt = get_frame_time();

        self.elapsed += dt;

        // The cinematic is a one-shot state. Gameplay time does not advance until
        // the operator terminal has finished booting, so the intro cannot consume
        // access-code windows or trigger incidents.
        if self.intro_active {
            self.update_intro(dt);
            return;
        }

        let was_in_transit = self.state.transit_active;
        self.state.tick(dt);

        // Centralized consequence model: unresolved incidents continuously damage the vessel.
        let mut catastrophic = false;
        if let Some(repair) = &self.state.active_repair {
            let damage = repair.damage_per_second * dt;
            match repair.event_kind {
                EventKind::CoolantOverheat => {
                    self.ship.reactor_cooling = (self.ship.reactor_cooling - damage * 70.0).max(0.0);
                    self.ship.engine_temperature = (self.ship.engine_temperature + damage * 32.0).min(150.0);
                    self.ship.engine_integrity = (self.ship.engine_integrity - damage * 5.0).max(0.0);
                }
                EventKind::LifeSupportPressureDrop => {
                    self.ship.life_support_integrity = (self.ship.life_support_integrity - damage * 55.0).max(0.0);
                    self.ship.life_pressure = (self.ship.life_pressure - damage * 0.18).max(0.0);
                }
                EventKind::PowerBusFailure => {
                    self.ship.power_bus_integrity = (self.ship.power_bus_integrity - damage * 42.0).max(0.0);
                    self.ship.power = self.ship.power.saturating_sub((damage * 12.0) as u8);
                }
                EventKind::SensorArrayGlitch | EventKind::NavigationDrift => {
                    self.ship.sensor_noise = (self.ship.sensor_noise + damage * 28.0 + 0.18 * dt).min(100.0);
                    if self.ship.sensor_noise > 55.0 { self.ship.course_locked = false; }
                }
                EventKind::FuelValveStuck => {
                    self.ship.fuel = self.ship.fuel.saturating_sub((damage * 5.0 + 0.25 * dt) as u8);
                    self.ship.engine_integrity = (self.ship.engine_integrity - damage * 18.0).max(0.0);
                }
                EventKind::MicrometeorStrike => {
                    self.ship.hull = self.ship.hull.saturating_sub((damage * 14.0 + 0.2 * dt) as u8);
                    self.ship.power_bus_integrity = (self.ship.power_bus_integrity - damage * 12.0).max(0.0);
                }
            }
            catastrophic = self.ship.engine_temperature >= 130.0
                || self.ship.life_support_integrity <= 0.0
                || self.ship.engine_integrity <= 0.0
                || self.ship.hull <= 0
                || self.ship.power == 0;
        }

        let critical_now = self.ship.engine_temperature >= 100.0
            || self.ship.reactor_cooling <= 25.0
            || self.ship.life_support_integrity <= 30.0
            || self.ship.power_bus_integrity <= 25.0
            || self.ship.engine_integrity <= 30.0
            || self.ship.hull <= 30;

        if critical_now && !self.critical_alert && !self.state.game_over {
            self.critical_alert = true;
            self.critical_reboot_pending = true;
            self.boot_wait_timer = 1.35;
            self.audio.start_siren();
            self.terminal.clear();
            self.terminal.queue(&[
                "!!! EMERGENCY POWER INTERRUPTION !!!".into(),
                "CRITICAL VESSEL CONDITION DETECTED.".into(),
                "TERMINAL SAFETY WATCHDOG TRIPPED.".into(),
                "FORCED CONSOLE REBOOT IN PROGRESS...".into(),
            ]);
        } else if !critical_now && self.critical_alert {
            self.critical_alert = false;
            self.audio.stop_siren();
        }

        if catastrophic {
            self.state.fail("CATASTROPHIC SYSTEM FAILURE // VESSEL LOST");
            self.audio.stop_siren();
            self.critical_alert = true;
            self.terminal.queue(&["".into(),"!!! CRITICAL FAILURE !!!".into(),"AURORA-17 HAS LOST SAFE OPERATING MARGIN.".into(),"MISSION FAILED.".into()]);
        }

        // Transit hazards are resolved by the same incident director as ordinary failures.
        if was_in_transit && !self.state.transit_active && self.state.mission_phase == crate::engine::state::MissionPhase::Approach {
            self.ship.fuel = self.ship.fuel.saturating_sub(12);
            self.terminal.queue(&[
                "".into(),
                "WAYPOINT-01 // ORBITAL APPROACH".into(),
                "TRANSIT COMPLETE".into(),
                "NAVIGATION SOLUTION................. VERIFIED".into(),
                "AUTOPILOT........................... DISENGAGED".into(),
                "ORBITAL CONTROL CHANNEL............. ACQUIRED".into(),
                "".into(),
                "DOCKING CLEARANCE REQUIRED.".into(),
                "USE: DOCK REQUEST".into(),
            ]);
            self.audio.beep();
        }

        if self.booted && self.state.active_repair.is_none() && !self.state.game_over && self.state.mission_phase != crate::engine::state::MissionPhase::Approach && self.state.mission_phase != crate::engine::state::MissionPhase::Complete {
            if let Some(event) = self.scheduler.update(&mut self.state) {
                self.apply_event(event);
            }
        }

        self.tracking_timer =
            (self.tracking_timer - dt).max(0.0);

        let mut input = self.terminal.update_input();

        if self.boot_sequence_active {
            self.update_boot_sequence(dt);
            return;
        }

        if self.boot_wait_timer > 0.0 {
            self.boot_wait_timer -= dt;
            if self.boot_wait_timer <= 0.0 && self.critical_reboot_pending {
                self.critical_reboot_pending = false;
                self.terminal.queue_slow(&[
                    "AURORA-17 // SAFETY WATCHDOG REBOOT".to_string(),
                    "".to_string(),
                    "EMERGENCY CONSOLE RECOVERY COMPLETE.".to_string(),
                    "CRITICAL CONDITION PERSISTS.".to_string(),
                    "".to_string(),
                    "IMMEDIATE ACTION REQUIRED.".to_string(),
                    "RUN: DIAGNOSE".to_string(),
                ]);
            }
            return;
        }

        if input.key_pressed {
            if self.elapsed - self.last_click_time > 0.15 {
                self.audio.key_click();
                self.last_click_time = self.elapsed;
            }
        }

        if let Some(command) = input.command {
            self.audio.command();

            let result = execute(
                &command,
                &mut self.ship,
                &mut self.state,
            );

            if result.clear {
                self.terminal.clear();
            }

            if result.resolved_diagnostic {
                self.state.diagnostic_attention = false;
            }

            if result.lines.iter().any(|l| l.contains("DOCKING CLEARANCE GRANTED")) { self.audio.dock_clamp(); }
            self.terminal.queue(&result.lines);
        }
    }

    fn update_intro(&mut self, dt: f32) {
        self.intro_elapsed += dt;

        // Each stage is emitted exactly once. No input is read during the sequence,
        // preventing the old boot/intro double-playback path.
        match self.intro_stage {
            0 if self.intro_elapsed >= 0.2 => {
                self.audio.intro_wake();
                self.terminal.clear();
                self.terminal.queue_slow(&[
                    "AURORA-17".into(),
                    "DEEP SPACE TRANSIT VESSEL".into(),
                    "".into(),
                    "MISSION 01 // WAYPOINT-01".into(),
                ]);
                self.intro_stage = 1;
            }
            1 if self.intro_elapsed >= 4.6 => {
                self.audio.intro_radio();
                self.terminal.queue_slow(&[
                    "REMOTE OPERATIONS LINK // ENCRYPTED".into(),
                    "".into(),
                    "UPLINK LATENCY ........ 8.4 SEC".into(),
                    "AUTONOMOUS CONTROL ..... STANDBY".into(),
                ]);
                self.intro_stage = 2;
            }
            2 if self.intro_elapsed >= 8.8 => {
                self.audio.intro_hum();
                self.terminal.queue_slow(&[
                    "VESSEL STATUS".into(),
                    "".into(),
                    "REACTOR ................ NOMINAL".into(),
                    "LIFE SUPPORT ........... NOMINAL".into(),
                    "NAVIGATION ............. CALIBRATION REQUIRED".into(),
                ]);
                self.intro_stage = 3;
            }
            3 if self.intro_elapsed >= 13.6 => {
                self.audio.intro_relay();
                self.terminal.queue_slow(&[
                    "".into(),
                    "WAKE AUTHORITY ACCEPTED.".into(),
                    "OPERATOR CONSOLE POWERING ON...".into(),
                ]);
                self.intro_stage = 4;
            }
            4 if self.intro_elapsed >= 17.2 => {
                self.intro_active = false;
                self.terminal.clear();
                self.boot_sequence_active = true;
                self.boot_stage = 0;
                self.boot_stage_timer = 0.0;
                self.audio.boot_power();
            }
            _ => {}
        }
    }

    fn update_boot_sequence(&mut self, dt: f32) {
        self.boot_stage_timer += dt;
        match self.boot_stage {
            0 if self.boot_stage_timer >= 1.2 => {
                self.audio.boot_scan();
                self.terminal.queue(&[
                    "AURORA-17 // VESSEL TERMINAL FW 5.0 // GAME 0.6.2".into(),
                    "".into(),
                    "POWERING SYSTEM BACKPLANE...".into(),
                    "POWER BUS ................................ ONLINE".into(),
                    "AUXILIARY / EMERGENCY DYN-FEED ............ STABLE".into(),
                ]);
                self.boot_stage = 1;
                self.boot_stage_timer = 0.0;
            }
            1 if self.boot_stage_timer >= 2.4 => {
                self.terminal.queue(&[
                    "LIFE SUPPORT RECIRCULATION ................ OK".into(),
                    "THERMAL CONTROL ............................ NOMINAL".into(),
                    "FLIGHT COMPUTER ............................ ONLINE".into(),
                    "NAVIGATION REFERENCE ....................... STANDBY".into(),
                ]);
                self.audio.relay_engage();
                self.boot_stage = 2;
                self.boot_stage_timer = 0.0;
            }
            2 if self.boot_stage_timer >= 2.8 => {
                self.terminal.queue(&[
                    "".into(),
                    "REMOTE OPERATOR LINK ESTABLISHED.".into(),
                    "SECURITY TOKEN AUTHORITY .................. ONLINE".into(),
                    "INCIDENT DIRECTOR .......................... STANDBY".into(),
                ]);
                self.boot_stage = 3;
                self.boot_stage_timer = 0.0;
            }
            3 if self.boot_stage_timer >= 2.6 => {
                self.terminal.queue_slow(&[
                    "".into(),
                    "BOOT SEQUENCE COMPLETE".into(),
                    "".into(),
                    "NAVIGATION SUBSYSTEM REQUIRES OPERATOR INPUT.".into(),
                    "TYPE 'HELP' FOR AVAILABLE COMMANDS.".into(),
                ]);
                self.audio.terminal_ready();
                self.audio.start_ambient(0.2);
                self.booted = true;
                self.boot_sequence_active = false;
                self.boot_stage = 4;
            }
            _ => {}
        }
    }

    fn apply_event(&mut self, event: Event) {
        // Freeze event selection until this incident is explicitly resolved.
        self.state.active_repair = Some(crate::engine::state::ActiveRepair {
            event_kind: event.kind,
            sequence: event.repair_sequence.clone(),
            current_step: 0,
            required_system: event.required_system,
            started_at: self.state.play_time,
            authorization: None,
            attempts: 0,
            damage_per_second: event.damage_per_second,
        });
        self.state.diagnostic_attention = true;

        self.audio.anomaly();
        if matches!(event.kind, EventKind::CoolantOverheat | EventKind::LifeSupportPressureDrop) {
            self.audio.beep();
        }

        let mut lines = vec![String::new(), event.title.to_string(), String::new()];

        for line in event.lines {
            lines.push(line.to_string());
        }

        self.terminal.queue(&lines);
    }

    pub fn draw(&mut self) {
        // --- 1. Рендерим терминал в offscreen target фиксированного размера ---
        set_camera(&Camera2D {
            render_target: Some(self.target.clone()),
            target: vec2(TARGET_W / 2.0, TARGET_H / 2.0),
            zoom: vec2(1.0 / (TARGET_W / 2.0), 1.0 / (TARGET_H / 2.0)),
            ..Default::default()
        });

        self.terminal.draw_terminal(
            self.elapsed,
            self.state.diagnostic_attention,
            self.critical_alert,
        );

        set_default_camera();

        // --- 2. Рендерим результат на экран с CRT шейдером ---
        clear_background(
            Color::new(
                0.002,
                0.003,
                0.002,
                1.0,
            )
        );

        // Считаем масштаб, чтобы вписать 1280x720 в текущее окно с сохранением пропорций
        let sw = screen_width();
        let sh = screen_height();

        let scale_x = sw / TARGET_W;
        let scale_y = sh / TARGET_H;
        let scale = scale_x.min(scale_y) * 0.94;
        
        let draw_w = TARGET_W * scale;
        let draw_h = TARGET_H * scale;
        let draw_x = (sw - draw_w) * 0.5;
        let draw_y = (sh - draw_h) * 0.5;

        gl_use_material(&self.crt);

        draw_texture_ex(
            &self.target.texture,
            draw_x,
            draw_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                ..Default::default()
            },
        );

        gl_use_default_material();

        // Minimal outer frame: makes the CRT feel like a physical console instead of a
        // floating texture, while preserving the terminal's monochrome identity.
        draw_rectangle_lines(
            draw_x - 2.0,
            draw_y - 2.0,
            draw_w + 4.0,
            draw_h + 4.0,
            2.0,
            Color::new(0.10, 0.35, 0.16, 0.55),
        );

        // --- 3. VHS tracking disturbance (рисуем уже в экранных координатах) ---
        if self.tracking_timer > 0.0 {
            let h = draw_h;

            let y =
                (self.elapsed * 510.0) % (h + 80.0)
                    - 40.0;

            draw_rectangle(
                draw_x,
                draw_y + y,
                draw_w,
                3.0,
                Color::new(
                    0.35,
                    1.0,
                    0.42,
                    0.075,
                ),
            );
        }
    }
}

const VERTEX_SHADER: &str = r#"
#version 100

attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position =
        Projection
        * Model
        * vec4(
            position,
            1.0
        );

    uv = texcoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 100

precision lowp float;

varying lowp vec2 uv;

uniform vec4 _Time;
uniform sampler2D Texture;

void main() {
    vec2 p = uv * 2.0 - 1.0;

    float r2 = dot(p, p);

    // Уменьшено искажение
    p *= 1.0 + 0.02 * r2;

    vec2 warped = p * 0.5 + 0.5;

    vec3 color = texture2D(Texture, warped).rgb;

    float scan =
        0.975
        + 0.025
        * sin(warped.y * 900.0);

    color *= scan;

    float shimmer =
        1.0
        + 0.009
        * sin(_Time.x * 5.7);

    color *= shimmer;

    color.r *= 0.88;
    color.g *= 1.02;
    color.b *= 0.88;

    // Убрана жесткая маска inside, чтобы углы не обрезались

    // Мягкая виньетка по краям
    float vignette =
        1.0
        - smoothstep(
            0.80,
            1.50,
            r2
        );

    color *= 0.90 + 0.10 * vignette;

    gl_FragColor = vec4(color, 1.0);
}
"#;
