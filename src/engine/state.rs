use super::events::EventKind;
use super::rng::Rng;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SystemType { Engine, Navigation, Power, LifeSupport }

impl SystemType {
    pub fn as_str(self) -> &'static str {
        match self { Self::Engine=>"ENGINE", Self::Navigation=>"NAVIGATION", Self::Power=>"POWER", Self::LifeSupport=>"LIFE SUPPORT" }
    }
    pub fn index(self) -> usize { match self { Self::Engine=>0, Self::Navigation=>1, Self::Power=>2, Self::LifeSupport=>3 } }
}

#[derive(Clone, Debug)]
pub struct RepairStep { pub command:String, pub args:String, pub description:String }

#[derive(Clone, Debug)]
pub struct ActiveRepair {
    pub event_kind: EventKind,
    pub sequence: Vec<RepairStep>,
    pub current_step: usize,
    pub required_system: SystemType,
    pub started_at: f32,
    pub authorization: Option<Authorization>,
    pub attempts: u32,
    pub damage_per_second: f32,
}

#[derive(Clone, Debug)]
pub struct Authorization { pub system:SystemType, pub code:u32, pub granted_at:f32 }

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MissionPhase { Preflight, Transit, Approach, Docking, Complete, Failed }

pub struct PlayerProfile {
    pub commands_run:u64, pub diagnostics_run:u64, pub logs_run:u64, pub scans_run:u64, pub comms_run:u64,
    pub events_seen:u64, pub repair_errors:u64, pub help_calls:u64, pub repair_successes:u64,
    pub total_reaction_time:f32, pub last_event_time:f32,
}
impl PlayerProfile { pub fn new()->Self{Self{commands_run:0,diagnostics_run:0,logs_run:0,scans_run:0,comms_run:0,events_seen:0,repair_errors:0,help_calls:0,repair_successes:0,total_reaction_time:0.0,last_event_time:0.0}} }

pub struct GameState {
    pub rng:Rng, pub seed:u64, pub play_time:f32, pub player:PlayerProfile,
    pub diagnostic_attention:bool, pub power_instability:f32, pub last_event_time:f32, pub next_event_time:f32,
    pub event_history:Vec<u8>, pub transit_active:bool, pub transit_time_left:f32,
    pub mission_phase:MissionPhase, pub access_codes:[u32;4], pub last_code_rotation:f32,
    pub active_repair:Option<ActiveRepair>, pub game_over:bool, pub failure_reason:String,
    pub docking_requested:bool, pub docking_denied:bool, pub docking_granted:bool,
    pub transit_incident_timer:f32,
}

impl GameState {
    pub fn new(seed:u64)->Self {
        let mut rng=Rng::new(seed);
        let codes=[rng.range_u32(10000,99999),rng.range_u32(10000,99999),rng.range_u32(10000,99999),rng.range_u32(10000,99999)];
        Self { rng:rng.clone(), seed, play_time:0.0, player:PlayerProfile::new(), diagnostic_attention:false,
            power_instability:0.0,last_event_time:-999.0,next_event_time:rng.range_f32(55.0,90.0),event_history:Vec::new(),
            transit_active:false,transit_time_left:300.0,mission_phase:MissionPhase::Preflight,access_codes:codes,last_code_rotation:0.0,
            active_repair:None,game_over:false,failure_reason:String::new(),docking_requested:false,docking_denied:false,docking_granted:false,
            transit_incident_timer:rng.range_f32(24.0,42.0) }
    }
    pub fn tick(&mut self,dt:f32) {
        if self.game_over || self.mission_phase==MissionPhase::Complete { return; }
        self.play_time+=dt;
        self.power_instability=(self.power_instability-dt*0.004).max(0.0);
        let minute=(self.play_time/60.0).floor() as u32;
        let old=(self.last_code_rotation/60.0).floor() as u32;
        if minute>old { for c in &mut self.access_codes {*c=self.rng.range_u32(10000,99999);} self.last_code_rotation=minute as f32*60.0; }
        if self.transit_active { self.transit_time_left=(self.transit_time_left-dt).max(0.0); if self.transit_time_left<=0.0 { self.transit_active=false; self.mission_phase=MissionPhase::Approach; } }
        if self.transit_active { self.transit_incident_timer-=dt; if self.transit_incident_timer<=0.0 { self.transit_incident_timer=self.rng.range_f32(28.0,52.0); self.next_event_time=self.play_time; } }
        if let Some(r)=&self.active_repair { if self.play_time-r.started_at>95.0 { self.player.repair_errors+=1; self.active_repair=None; self.diagnostic_attention=false; self.next_event_time=self.play_time+10.0; } }
    }
    pub fn code_for(&self,system:SystemType)->u32 { self.access_codes[system.index()] }
    pub fn code_valid(&self,system:SystemType,code:u32)->bool { self.code_for(system)==code && self.play_time-self.last_code_rotation<60.0 }
    pub fn code_remaining(&self)->f32 { (60.0-(self.play_time-self.last_code_rotation)).clamp(0.0,60.0) }
    pub fn remember_event(&mut self,id:u8){self.event_history.push(id);if self.event_history.len()>10{self.event_history.remove(0);}}
    pub fn was_recent(&self,id:u8)->bool{self.event_history.iter().rev().take(4).any(|v|*v==id)}
    pub fn fail(&mut self,reason:&str){self.game_over=true;self.mission_phase=MissionPhase::Failed;self.failure_reason=reason.to_string();self.transit_active=false;self.active_repair=None;}
}
