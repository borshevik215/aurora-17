use crate::{engine::state::{GameState,MissionPhase,SystemType,Authorization},ship::{navigation,Ship}};

pub struct CommandResult{pub lines:Vec<String>,pub clear:bool,pub resolved_diagnostic:bool}
impl CommandResult{fn text(t:&str)->Self{Self{lines:t.lines().map(str::to_string).collect(),clear:false,resolved_diagnostic:false}}}
fn system(s:&str)->Option<SystemType>{match s.to_lowercase().as_str(){"engine"|"eng"=>Some(SystemType::Engine),"navigation"|"nav"=>Some(SystemType::Navigation),"power"=>Some(SystemType::Power),"life"|"life_support"|"life-support"|"lifesupport"=>Some(SystemType::LifeSupport),_=>None}}
fn parse_calibration(a:&str)->Option<(f32,f32)>{let p:Vec<_>=a.split_whitespace().collect();if p.len()!=2{return None}Some((p[0].parse().ok()?,p[1].parse().ok()?))}

fn execute_inner(input:&str,ship:&mut Ship,state:&mut GameState)->CommandResult{
 let mut p=input.trim().splitn(2,char::is_whitespace); let cmd=p.next().unwrap_or("").to_lowercase(); let args=p.next().unwrap_or("").trim(); state.player.commands_run+=1;
 match cmd.as_str(){
  ""=>CommandResult::text(""),
  "help"=>CommandResult::text("AVAILABLE COMMANDS\n\nhelp\nstatus\ndiagnose [system]\nscan [system]\nrequest <system>\nauth <system> <code>\ncodes\naccess\ntracker\ncalibrate <ra> <dec>\nlock\ndepart\nnav\nengine\ncomms\nscan\nlogs\ninspect\nvalve <id> <open|close>\npower <bus> <reset|boost>\npurge <coolant|vent|fuel>\ndock request\nclear\n\nREPAIR PROTOCOL:\n1) diagnose [system]\n2) request <system>\n3) auth <system> <code>\n4) enter the exact action shown by DIAGNOSE"),
  "seed"=>CommandResult::text(&format!("MISSION SEED\n\n{}\n\nPASS THIS VALUE TO THE PROGRAM AS THE FIRST ARGUMENT TO REPRODUCE THIS RUN.",state.seed)),
  "status"=>CommandResult::text(&format!("VESSEL STATUS\n\nMISSION PHASE....... {:?}\nPOWER............... {}%\nFUEL................ {}%\nHULL................ {}%\nREACTOR COOLING..... {:.0}%\nLIFE SUPPORT........ {:.0}%\nPOWER BUS........... {:.0}%\nENGINE INTEGRITY... {:.0}%\nENGINE TEMP........ {:.1} C\nNAVIGATION.......... {}\nAUTOPILOT........... {}\n\nACTIVE INCIDENT..... {}\nCODE ROTATION....... {:02}s",state.mission_phase,ship.power,ship.fuel,ship.hull,ship.reactor_cooling,ship.life_support_integrity,ship.power_bus_integrity,ship.engine_integrity,ship.engine_temperature,if ship.navigation_calibrated{"CALIBRATED"}else{"FAULT"},if ship.course_locked{"READY"}else{"STANDBY"},if state.active_repair.is_some(){"YES"}else{"NO"},state.code_remaining() as u32)),
  "diagnose"=>{
   state.player.diagnostics_run+=1;
   let requested=system(args);
   let mut out=format!("SYSTEM DIAGNOSTICS\n\nPOWER BUS........... {:.0}%\nREACTOR COOLING..... {:.0}%\nLIFE SUPPORT........ {:.0}%\nENGINE INTEGRITY.... {:.0}%\nSENSOR NOISE........ {:.1}%",ship.power_bus_integrity,ship.reactor_cooling,ship.life_support_integrity,ship.engine_integrity,ship.sensor_noise);
   if let Some(sys)=requested {
      let detail=match sys {
       SystemType::Engine=>format!("\n\nENGINE // DIAGNOSTIC DETAIL\nCOOLANT PUMP......... {}\nENGINE TEMP.......... {:.1} C\nFUEL FLOW............ {}\nENGINE INTEGRITY..... {:.0}%",if ship.reactor_cooling<70.0{"DEGRADED"}else{"NOMINAL"},ship.engine_temperature,if ship.fuel>15{"STABLE"}else{"LOW"},ship.engine_integrity),
       SystemType::Navigation=>format!("\n\nNAVIGATION // DIAGNOSTIC DETAIL\nSTAR TRACKER......... {}\nINERTIAL REFERENCE... {}\nSENSOR NOISE......... {:.1}%\nCOURSE LOCK.......... {}",if ship.navigation_calibrated{"CALIBRATED"}else{"NOT CALIBRATED"},if ship.sensor_noise>45.0{"BIAS / DRIFT"}else{"STABLE"},ship.sensor_noise,if ship.course_locked{"LOCKED"}else{"DROPPED / UNVERIFIED"}),
       SystemType::Power=>format!("\n\nPOWER // DIAGNOSTIC DETAIL\nMAIN BUS............. {:.0}%\nAUX FEED............. {}\nLOAD SHED............ {}",ship.power_bus_integrity,if ship.power_bus_integrity<60.0{"UNSTABLE"}else{"STABLE"},if ship.power_bus_integrity<40.0{"ACTIVE"}else{"STANDBY"}),
       SystemType::LifeSupport=>format!("\n\nLIFE SUPPORT // DIAGNOSTIC DETAIL\nREGULATOR............ {}\nCABIN PRESSURE....... {:.0}%\nOXYGEN RESERVE....... {:.0}%",if ship.life_support_integrity<70.0{"FAULT"}else{"NOMINAL"},ship.life_pressure*100.0,ship.life_support_integrity),
      };
      out.push_str(&detail);
   }
   if let Some(r)=&state.active_repair {
      let step=&r.sequence[r.current_step];
      out.push_str(&format!("\n\nACTIVE INCIDENT\nEVENT................ {:?}\nFAULT DOMAIN......... {}\nSTEP................. {} / {}\nACTION............... {} {}\nWHAT THIS FIX DOES... {}\n\nTOKEN REQUIRED: request {}",r.event_kind,r.required_system.as_str(),r.current_step+1,r.sequence.len(),step.command,step.args,step.description,r.required_system.as_str().to_lowercase()));
   } else { out.push_str("\n\nNO ACTIVE INCIDENT. SYSTEMS NOMINAL."); }
   CommandResult::text(&out)
  },
  "request"=>{let a=args.split_whitespace().next().unwrap_or(""); if let Some(sys)=system(a){CommandResult::text(&format!("ACCESS REQUEST // {}\n\nREQUEST TRANSMITTED TO MISSION SECURITY.\nTOKEN CLASS: ROTATING\nTOKEN WINDOW: {:02} SECONDS\n\nCURRENT TOKEN: {:05}\n\nUSE:\n  auth {} <code>",sys.as_str(),state.code_remaining() as u32,state.code_for(sys),a))}else{CommandResult::text("ACCESS REQUEST REJECTED\n\nSPECIFY: engine | navigation | power | life")}},
  "auth"=>{let p:Vec<_>=args.split_whitespace().collect();if p.len()!=2{CommandResult::text("USAGE: auth <system> <code>")}else if let Some(sys)=system(p[0]){match p[1].parse::<u32>(){Ok(code) if state.code_valid(sys,code)=>{let now=state.play_time;if let Some(r)=state.active_repair.as_mut(){if r.required_system==sys{r.authorization=Some(Authorization{system:sys,code,granted_at:now});return CommandResult::text(&format!("AUTHORIZATION ACCEPTED // {}\n\nTOKEN VERIFIED.\nONE REPAIR ACTION AUTHORIZED.\nAPPLY IT NOW.",sys.as_str()));}}CommandResult::text("AUTHORIZATION VALID, BUT NO MATCHING REPAIR IS REQUESTING ACCESS.")},_=>CommandResult::text("AUTHORIZATION DENIED\n\nTOKEN INVALID OR EXPIRED.")}}else{CommandResult::text("UNKNOWN SECURITY DOMAIN.")}},
  "codes"=>CommandResult::text(&format!("ACTIVE SECURITY TOKENS\n\nENGINE........ {:05}\nNAVIGATION.... {:05}\nPOWER......... {:05}\nLIFE SUPPORT.. {:05}\n\nROTATION IN.... {:02} SECONDS",state.access_codes[0],state.access_codes[1],state.access_codes[2],state.access_codes[3],state.code_remaining() as u32)),
  "access"=>CommandResult::text(&format!("SECURITY\n\nROTATION WINDOW: 60 SECONDS\nNEXT ROTATION: {:02} SECONDS\n\nREQUEST A TOKEN WITH:\n  request <system>",state.code_remaining() as u32)),
  "tracker"=>CommandResult::text(if ship.navigation_calibrated{"STAR TRACKER\nSTATUS: CALIBRATED\nCONFIDENCE: 99.8%"}else{"STAR TRACKER\nSTATUS: DRIFT DETECTED\nRA OFFSET: +0.084\nDEC OFFSET: -0.031\nCONFIDENCE: 61%"}),
  "calibrate"=>{if state.mission_phase!=MissionPhase::Preflight{CommandResult::text("CALIBRATION LOCKED\n\nSTAR TRACKER CALIBRATION IS A PREFLIGHT PROCEDURE.\nTHE VESSEL IS ALREADY IN FLIGHT.")}else if let Some((ra,dec))=parse_calibration(args){if navigation::calibration_matches(ra,dec){ship.navigation_calibrated=true;CommandResult::text("STAR TRACKER CALIBRATION\nREFERENCE MATCH........ OK\nGYRO CROSSCHECK......... OK\nRESIDUAL ERROR.......... 0.0007 DEG\n\nCALIBRATION COMMITTED.")}else{CommandResult::text("CALIBRATION REJECTED\nREFERENCE MATCH FAILED.")}}else{CommandResult::text("USAGE: calibrate <RA_OFFSET> <DEC_OFFSET>")}},
  "lock"=>{if !ship.navigation_calibrated{CommandResult::text("COURSE LOCK REJECTED\nSTAR TRACKER IS NOT CALIBRATED.")}else{ship.course_locked=true;CommandResult::text("COURSE SOLUTION VERIFIED\nWAYPOINT-01 VECTOR LOCKED\nCOLLISION CHECK........ CLEAR\nAUTOPILOT READY.\n\nTYPE DEPART TO BEGIN TRANSIT.")}},
  "depart"=>{if state.mission_phase!=MissionPhase::Preflight{CommandResult::text("DEPARTURE REJECTED\nVESSEL IS NO LONGER IN PREFLIGHT.")}else if !ship.navigation_calibrated{CommandResult::text("DEPARTURE REJECTED\nSTAR TRACKER IS NOT CALIBRATED.")}else if !ship.course_locked{CommandResult::text("DEPARTURE REJECTED\nNO VERIFIED COURSE.")}else if state.transit_active{CommandResult::text("TRANSIT ALREADY ACTIVE.")}else{state.transit_active=true;state.mission_phase=MissionPhase::Transit;state.transit_time_left=300.0;state.next_event_time=state.play_time+state.rng.range_f32(22.0,48.0);state.transit_incident_timer=state.rng.range_f32(20.0,36.0);CommandResult::text("DEPARTURE SEQUENCE\n\nWAYPOINT-01 VECTOR CONFIRMED\nBURN WINDOW OPEN\nAUTOPILOT ENGAGED\n\nTRANSIT TIME: 05:00\n\nINCIDENT DIRECTOR: ONLINE\nEVENTS WILL BEGIN AFTER DEPARTURE.")}},
  "nav"=>CommandResult::text(&format!("NAVIGATION\n\nPHASE........ {:?}\nDESTINATION.. WAYPOINT-01\nDISTANCE..... {}\nCOURSE....... {}\nETA.......... {}",state.mission_phase,if state.mission_phase==MissionPhase::Approach{"0.00 AU"}else{"4.81 AU"},if ship.course_locked{"LOCKED"}else{"UNVERIFIED"},if state.transit_active{format!("{:02}:{:02}",(state.transit_time_left/60.0) as u32,(state.transit_time_left%60.0) as u32)}else{"ARRIVED".into()})),
  "engine"=>CommandResult::text(&format!("PROPULSION\n\nTEMP........ {:.1} C\nCOOLING..... {:.0}%\nPRESSURE.... {:.0}%\nINTEGRITY... {:.0}%\nFUEL FLOW... {}",ship.engine_temperature,ship.reactor_cooling,ship.engine_pressure,ship.engine_integrity,if ship.fuel>15{"NOMINAL"}else{"LOW RESERVE"})),
  "comms"=>{state.player.comms_run+=1;CommandResult::text("COMMUNICATIONS\n\nUPLINK........ ACTIVE\nMISSION CONTROL AVAILABLE\nLATENCY........ 8.4 SEC\n\nSECURITY CHANNEL: ENCRYPTED")},
  "scan"=>{
   state.player.scans_run+=1;
   if let Some(sys)=system(args) {
      match sys {
       SystemType::Navigation=>CommandResult::text(&format!("NAVIGATION SCAN\n\nSTAR TRACKER CH-B..... {}\nIRU-2 REFERENCE....... {}\nVECTOR CONFIDENCE..... {}\nSENSOR NOISE........... {:.1}%\nCOURSE LOCK............ {}\n\nIf DIAGNOSE reports an active navigation fault, follow its exact repair step.",if ship.navigation_calibrated{"ONLINE"}else{"OFFLINE / UNCALIBRATED"},if ship.sensor_noise>45.0{"BIAS DETECTED"}else{"STABLE"},if ship.sensor_noise>55.0{"LOW"}else{"NOMINAL"},ship.sensor_noise,if ship.course_locked{"LOCKED"}else{"NOT LOCKED"})),
       SystemType::Engine=>CommandResult::text(&format!("ENGINE SCAN\n\nCOOLANT FLOW.......... {:.0}%\nCORE TEMPERATURE...... {:.1} C\nFUEL PRESSURE......... {:.0}%\nENGINE INTEGRITY...... {:.0}%",ship.reactor_cooling,ship.engine_temperature,ship.engine_pressure,ship.engine_integrity)),
       SystemType::Power=>CommandResult::text(&format!("POWER SCAN\n\nMAIN BUS.............. {:.0}%\nPOWER OUTPUT.......... {}%\nLOAD STATE............ {}",ship.power_bus_integrity,ship.power,if ship.power_bus_integrity<50.0{"UNSTABLE"}else{"STABLE"})),
       SystemType::LifeSupport=>CommandResult::text(&format!("LIFE SUPPORT SCAN\n\nREGULATOR INTEGRITY... {:.0}%\nCABIN PRESSURE......... {:.0}%\nOXYGEN RESERVE......... {:.0}%",ship.life_support_integrity,ship.life_pressure*100.0,ship.life_support_integrity)),
      }
   } else { CommandResult::text(&format!("PASSIVE SENSOR SWEEP\n\nTHERMAL........ {:.1}%\nSENSOR NOISE... {:.1}%\nHULL RETURNS... {}\nNAV REFERENCE.. {}\n\nTIP: during an incident use scan nav / scan engine / scan power / scan life.",ship.engine_temperature,ship.sensor_noise,if ship.hull<85{"IMPACT SIGNATURE"}else{"CLEAR"},if ship.sensor_noise<35.0{"STABLE"}else{"DEGRADED"})) }
  },
  "logs"=>{state.player.logs_run+=1;CommandResult::text("MISSION LOG\n\n[BOOT] REMOTE LINK ESTABLISHED\n[NAV] WAYPOINT-01 SOLUTION LOADED\n[SEC] ROTATING TOKEN AUTHORITY ONLINE\n[OPS] AUTONOMOUS INCIDENT DIRECTOR ONLINE\n[WARN] UNPREDICTABLE TRANSIT CONDITIONS ENABLED")},
  "inspect"=>CommandResult::text(&format!("MANUAL INSPECTION\n\nHULL.............. {}%\nREACTOR COOLING... {:.0}%\nLIFE SUPPORT...... {:.0}%\nPOWER BUS......... {:.0}%\nENGINE............ {:.0}%",ship.hull,ship.reactor_cooling,ship.life_support_integrity,ship.power_bus_integrity,ship.engine_integrity)),
  "dock"=>{if args.to_lowercase()!="request"{CommandResult::text("USAGE: dock request")}else if state.mission_phase!=MissionPhase::Approach{CommandResult::text("DOCKING REQUEST UNAVAILABLE\nWAYPOINT-01 HAS NOT BEEN REACHED.")}else if ship.hull<60||ship.reactor_cooling<25.0||ship.life_support_integrity<40.0||ship.engine_integrity<40.0||ship.power_bus_integrity<35.0{state.docking_requested=true;state.docking_denied=true;CommandResult::text("ORBITAL CONTROL // DOCKING REQUEST\n\nREQUEST TRANSMITTED...\nVESSEL HEALTH CHECK IN PROGRESS...\n\n*** DOCKING CLEARANCE DENIED ***\n\nREASON: CRITICAL VESSEL DAMAGE\n\nAPPROACH HOLD REMAINS ACTIVE.\nREPAIR THE DAMAGED SYSTEMS, THEN ISSUE DOCK REQUEST AGAIN.\n\nIF NO INCIDENT IS ACTIVE, USE: service <system>")}else{state.docking_requested=true;state.docking_denied=false;state.docking_granted=true;state.mission_phase=MissionPhase::Complete;ship.mission_complete=true;CommandResult::text("ORBITAL CONTROL // DOCKING REQUEST\n\nREQUEST TRANSMITTED...\nAUTHENTICATING VESSEL...\nHULL INTEGRITY........ ACCEPTABLE\nPROPULSION............ SAFE\nLIFE SUPPORT.......... STABLE\nPOWER BUS............. STABLE\n\n*** DOCKING CLEARANCE GRANTED ***\n\nWAYPOINT-01 // ORBITAL STATION\nDOCKING CLAMPS: ENGAGED\nUMBILICAL: CONNECTED\nORBITAL CONTROL HANDSHAKE: COMPLETE\n\nMISSION 01 COMPLETE\nAURORA-17 HAS ARRIVED.")}},
  "service"=>{
   if state.mission_phase!=MissionPhase::Approach { CommandResult::text("SERVICE REQUEST UNAVAILABLE\n\nMANUAL SERVICE IS ONLY AUTHORIZED IN ORBITAL APPROACH HOLD.") }
   else if state.active_repair.is_some() { CommandResult::text("SERVICE INTERLOCK\n\nAN ACTIVE INCIDENT MUST BE RESOLVED FIRST.") }
   else if let Some(sys)=system(args.split_whitespace().next().unwrap_or("")) {
      let sequence=match sys {
        SystemType::Engine=>vec![crate::engine::state::RepairStep{command:"power".into(),args:"aux reset".into(),description:"Reinitialize engine support electronics".into()},crate::engine::state::RepairStep{command:"purge".into(),args:"coolant".into(),description:"Restore reactor cooling flow".into()}],
        SystemType::LifeSupport=>vec![crate::engine::state::RepairStep{command:"valve".into(),args:"1 open".into(),description:"Rebalance habitat pressure".into()},crate::engine::state::RepairStep{command:"power".into(),args:"aux boost".into(),description:"Restore life-support power margin".into()}],
        SystemType::Power=>vec![crate::engine::state::RepairStep{command:"power".into(),args:"aux reset".into(),description:"Reset auxiliary power bus".into()},crate::engine::state::RepairStep{command:"power".into(),args:"main boost".into(),description:"Restore main bus stability".into()}],
        SystemType::Navigation=>vec![crate::engine::state::RepairStep{command:"scan".into(),args:"nav".into(),description:"Rescan the navigation reference channels".into()},crate::engine::state::RepairStep{command:"tracker".into(),args:"".into(),description:"Verify star tracker confidence".into()}],
      };
      state.active_repair=Some(crate::engine::state::ActiveRepair{event_kind:crate::engine::events::EventKind::PowerBusFailure,sequence,current_step:0,required_system:sys,started_at:state.play_time,authorization:None,attempts:0,damage_per_second:0.0});
      state.diagnostic_attention=true;
      CommandResult::text(&format!("ORBITAL SERVICE // {}\n\nMAINTENANCE CHANNEL OPEN.\nEVERY SERVICE ACTION REQUIRES A CURRENT ACCESS TOKEN.\n\nRUN: diagnose\nTHEN: request {}",sys.as_str(),args.split_whitespace().next().unwrap_or("system")))
   } else { CommandResult::text("USAGE: service engine | navigation | power | life") }
  },
  "clear"=>CommandResult{lines:Vec::new(),clear:true,resolved_diagnostic:false},
  "valve"|"power"|"purge"=>CommandResult::text(&format!("{} CONTROL ACCEPTED\nCOMMAND REGISTERED: {}\nWAITING FOR REPAIR AUTHORIZATION CONTEXT.",cmd.to_uppercase(),input.trim())),
  _=>CommandResult::text(&format!("ERROR: UNKNOWN COMMAND\n\nINPUT: {}\n\nTYPE HELP.",input.trim()))
 }
}

pub fn execute(input:&str,ship:&mut Ship,state:&mut GameState)->CommandResult{
 let normalized=input.trim().to_lowercase(); let mut p=normalized.splitn(2,char::is_whitespace); let command=p.next().unwrap_or(""); let args=p.next().unwrap_or("").trim();
 let expected=state.active_repair.as_ref().and_then(|r|r.sequence.get(r.current_step).cloned());
 if let Some(repair)=state.active_repair.as_ref(){
  if let Some(step)=&expected {
   let matches_expected=command==step.command && args==step.args;
   if matches_expected && repair.authorization.is_none(){
    return CommandResult::text(&format!("SECURITY INTERLOCK\n\n{} REPAIR REQUIRES A CURRENT ACCESS TOKEN.\n\nREQUEST: request {}\nAUTHORIZE: auth {} <code>",repair.required_system.as_str(),repair.required_system.as_str().to_lowercase(),repair.required_system.as_str().to_lowercase()));
   }
   let safe=matches!(command,"help"|"status"|"diagnose"|"request"|"auth"|"codes"|"access"|"tracker"|"nav"|"engine"|"comms"|"scan"|"logs"|"inspect"|"clear");
   if !matches_expected && !safe {state.player.repair_errors+=1;return CommandResult::text(&format!("REPAIR INTERLOCK\n\nSTEP {} OF {}\nEXPECTED: {} {}\n\nDO NOT IMPROVISE WHILE AN INCIDENT IS ACTIVE.",repair.current_step+1,repair.sequence.len(),step.command,step.args));}
  }
 }
 let is_repair_action=expected.as_ref().map(|s|command==s.command&&args==s.args).unwrap_or(false);
 let result=execute_inner(input,ship,state);
 if is_repair_action && !result.clear {
  let mut done=false; let mut desc=String::new(); if let Some(r)=state.active_repair.as_mut(){desc=r.sequence[r.current_step].description.clone();r.current_step+=1;r.authorization=None;done=r.current_step>=r.sequence.len();}
  if done {
   let reaction=state.play_time-state.player.last_event_time;state.player.repair_successes+=1;state.player.total_reaction_time+=reaction.max(0.0);
   let kind=state.active_repair.as_ref().map(|r|r.event_kind);
   let sys=state.active_repair.as_ref().map(|r|r.required_system);
   match sys {
    Some(SystemType::Engine)=>{ship.reactor_cooling=(ship.reactor_cooling+48.0).min(100.0);ship.engine_temperature=(ship.engine_temperature-28.0).max(42.0);ship.engine_integrity=(ship.engine_integrity+34.0).min(100.0);},
    Some(SystemType::LifeSupport)=>{ship.life_support_integrity=(ship.life_support_integrity+48.0).min(100.0);ship.life_pressure=(ship.life_pressure+0.25).min(1.0);},
    Some(SystemType::Power)=>{ship.power_bus_integrity=(ship.power_bus_integrity+42.0).min(100.0);ship.power=(ship.power+30).min(100);},
    Some(SystemType::Navigation)=>{ship.sensor_noise=(ship.sensor_noise-38.0).max(0.0);},
    None=>{}
   }
   if kind==Some(crate::engine::events::EventKind::MicrometeorStrike){ship.hull=(ship.hull+28).min(100);}
   state.diagnostic_attention=false;state.active_repair=None;return CommandResult{lines:[result.lines,vec!["".into(),"REPAIR ACTION ACCEPTED // SEQUENCE COMPLETE".into(),format!("RESOLVED: {}",desc.to_uppercase()),"SYSTEM STABILITY RESTORED.".into(),"INCIDENT CLEARED.".into()]].concat(),clear:false,resolved_diagnostic:true};}
  let step=state.active_repair.as_ref().unwrap(); let next=&step.sequence[step.current_step]; return CommandResult{lines:[result.lines,vec!["REPAIR ACTION ACCEPTED".into(),format!("SEQUENCE PROGRESS: {} / {}",step.current_step,step.sequence.len()),format!("NEXT: {} {}",next.command,next.args),format!("REQUEST NEW TOKEN: request {}",step.required_system.as_str().to_lowercase())]].concat(),clear:false,resolved_diagnostic:false};
 }
 result
}
