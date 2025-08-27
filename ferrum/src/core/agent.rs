// Dead simple agent trait
pub trait Agent {
    fn act(&mut self, obs: &str) -> String; // Action as string for now
}
