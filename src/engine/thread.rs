use crate::engine::killers::Killers;


pub struct ThreadData{
    pub killers: Killers,
}

impl Default for ThreadData {
    fn default() -> Self {
        Self{
            killers: Killers::default(),
        }
    }
}
