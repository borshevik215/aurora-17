use super::{
    events::{self, Event},
    state::GameState,
};

pub struct EventScheduler;

impl EventScheduler {
    pub fn new() -> Self {
        Self
    }

    pub fn update(
        &mut self,
        state: &mut GameState,
    ) -> Option<Event> {
        events::choose_event(state)
    }
}
