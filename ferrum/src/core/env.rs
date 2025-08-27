// Dead simple environment trait
pub trait Env {
    fn reset(&mut self) -> String; // Observation as string for now
    fn step(&mut self, action: String) -> (String, f32, bool); // obs, reward, done
}
