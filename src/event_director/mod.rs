#[derive(Clone)]
pub struct ShipEvent {
    pub id: &'static str,
    pub title: &'static str,
    pub tier: u8,
    pub cooldown: f32,
    pub resolved: bool,
}

pub struct EventDirector {
    pub active: Vec<ShipEvent>,
    pub history: Vec<String>,
    pub difficulty_tier: u8,
}

impl EventDirector {
    pub fn new() -> Self {
        Self { active: Vec::new(), history: Vec::new(), difficulty_tier: 1 }
    }

    pub fn remember(&mut self, id: &str) {
        self.history.push(id.to_string());
    }

    pub fn seen(&self, id: &str) -> bool {
        self.history.iter().any(|x| x == id)
    }
}
