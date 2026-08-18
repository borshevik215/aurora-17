use super::{rng::Rng,state::{GameState,RepairStep,SystemType}};

#[derive(Clone,Copy,PartialEq,Debug)]
pub enum EventKind { CoolantOverheat, LifeSupportPressureDrop, PowerBusFailure, SensorArrayGlitch, FuelValveStuck, MicrometeorStrike, NavigationDrift }
impl EventKind {
 pub fn id(self)->u8{match self{Self::CoolantOverheat=>0,Self::LifeSupportPressureDrop=>1,Self::PowerBusFailure=>2,Self::SensorArrayGlitch=>3,Self::FuelValveStuck=>4,Self::MicrometeorStrike=>5,Self::NavigationDrift=>6}}
 pub fn required_system(self)->SystemType{match self{Self::CoolantOverheat|Self::FuelValveStuck=>SystemType::Engine,Self::LifeSupportPressureDrop=>SystemType::LifeSupport,Self::PowerBusFailure=>SystemType::Power,Self::SensorArrayGlitch|Self::NavigationDrift=>SystemType::Navigation,Self::MicrometeorStrike=>SystemType::Power}}
 pub fn generate_repair_sequence(self,rng:&mut Rng)->Vec<RepairStep>{
  let v=rng.range_u32(1,4);
  match self {
   Self::CoolantOverheat=>vec![s("purge","coolant","Flush reactor coolant loop"),s("valve",&format!("{} close",v),"Isolate overheated branch"),s("power","aux reset","Restore auxiliary pump feed")],
   Self::LifeSupportPressureDrop=>vec![s("valve",&format!("{} open",v),"Open pressure equalization valve"),s("purge","vent","Purge contaminated pressure line"),s("power","aux boost","Stabilize life-support bus")],
   Self::PowerBusFailure=>vec![s("power","aux reset","Reset auxiliary bus"),s("power","main boost","Prime main relay"),s("diagnose","power","Verify bus stability")],
   Self::SensorArrayGlitch=>vec![s("tracker","","Read star tracker reference"),s("diagnose","sensors","Run sensor matrix check"),s("power","aux reset","Reset navigation electronics")],
   Self::FuelValveStuck=>vec![s("purge","fuel","Depressurize injector line"),s("valve",&format!("{} open",v),"Free seized injector valve"),s("valve",&format!("{} close",v),"Re-seat injector valve")],
   Self::MicrometeorStrike=>vec![s("power","aux reset","Reroute damaged electronics"),s("valve","2 close","Seal impact compartment"),s("purge","vent","Clear pressure debris")],
   Self::NavigationDrift=>vec![s("scan","","Rescan navigation reference"),s("tracker","","Cross-check star tracker"),s("power","aux reset","Reinitialize navigation reference")],
  }
 }
}
fn s(c:&str,a:&str,d:&str)->RepairStep{RepairStep{command:c.into(),args:a.into(),description:d.into()}}

pub struct Event{pub kind:EventKind,pub title:&'static str,pub lines:Vec<&'static str>,pub repair_sequence:Vec<RepairStep>,pub required_system:SystemType,pub damage_per_second:f32}

pub fn choose_event(state:&mut GameState)->Option<Event>{
 if !state.transit_active || state.play_time<state.next_event_time || state.active_repair.is_some() || state.game_over || state.mission_phase!=super::state::MissionPhase::Transit{return None;}
 let candidates=[EventKind::CoolantOverheat,EventKind::LifeSupportPressureDrop,EventKind::PowerBusFailure,EventKind::SensorArrayGlitch,EventKind::FuelValveStuck,EventKind::MicrometeorStrike,EventKind::NavigationDrift];
 let valid:Vec<_>=candidates.into_iter().filter(|k|!state.was_recent(k.id())).collect(); if valid.is_empty(){return None;}
 let kind=valid[state.rng.range_usize(0,valid.len()-1)]; state.remember_event(kind.id()); state.last_event_time=state.play_time; state.player.last_event_time=state.play_time; state.player.events_seen+=1;
 let base=if state.transit_active{18.0}else{35.0}; let spread=if state.transit_active{28.0}else{45.0}; state.next_event_time=state.play_time+state.rng.range_f32(base,base+spread);
 let (title,lines,dmg)=match kind{
  EventKind::CoolantOverheat=>("CRITICAL // REACTOR COOLING FAILURE",vec!["REACTOR THERMAL CONTROL: COOLANT PUMP #2 OFFLINE","COOLANT FLOW: 31% // BELOW SAFE LIMIT","CORE TEMPERATURE: RISING","FAILURE MODE: THERMAL RUNAWAY IF UNRESOLVED"],0.24),
  EventKind::LifeSupportPressureDrop=>("CRITICAL // HABITAT PRESSURE LOSS",vec!["HABITAT PRESSURE REGULATOR #3 STUCK OPEN","SECTOR 03 CONTAINMENT COMPROMISED","CABIN PRESSURE: FALLING","FAILURE MODE: OXYGEN RESERVE DEPLETION"],0.18),
  EventKind::PowerBusFailure=>("FAULT // POWER BUS DESYNCHRONIZATION",vec!["MAIN POWER BUS RELAY K-4 DESYNCHRONIZED","MAIN BUS VOLTAGE: OSCILLATING","AUXILIARY FEED: UNSTABLE","FAILURE MODE: CASCADE LOAD SHEDDING"],0.12),
  EventKind::SensorArrayGlitch=>("WARNING // STAR TRACKER ARRAY FAULT",vec!["STAR TRACKER ARRAY CHANNEL B OFFLINE","REFERENCE CONFIDENCE: 61%","NAVIGATION SOLUTION: DEGRADED","FAILURE MODE: AUTONAV MAY DROP COURSE"],0.0),
  EventKind::FuelValveStuck=>("ALARM // ENGINE INJECTOR VALVE SEIZURE",vec!["FUEL INJECTOR VALVE ACTUATOR: SEIZED","FUEL PRESSURE: IRREGULAR","THRUST MARGIN: FALLING","FAILURE MODE: ENGINE FEED INTERRUPTION"],0.08),
  EventKind::MicrometeorStrike=>("IMPACT // MICROMETEOROID HIT",vec!["HULL PANEL C-07: IMPACT DETECTED","ELECTRONICS COMPARTMENT: ARCING","ISOLATION VALVE: REQUIRED","FAILURE MODE: HULL INTEGRITY LOSS"],0.32),
  EventKind::NavigationDrift=>("ANOMALY // NAVIGATION REFERENCE DRIFT",vec!["INERTIAL REFERENCE UNIT IRU-2: BIAS DETECTED","REFERENCE VECTOR: MOVING","AUTONAV CONFIDENCE: FALLING","FAILURE MODE: COURSE LOCK WILL BE DROPPED"],0.0),
 };
 Some(Event{kind,title,lines,required_system:kind.required_system(),repair_sequence:kind.generate_repair_sequence(&mut state.rng),damage_per_second:dmg})
}
