pub struct ZenState {
    pub active: bool,
}

impl ZenState {
    pub fn new() -> Self {
        Self { active: false }
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }
}

pub fn render_help() -> String {
    "Press 'z' to toggle Zen Mode".to_string()
}
