use crate::core::{Agent, Env};

// The basic RL loop
pub fn train_episode(env: &mut dyn Env, agent: &mut dyn Agent) -> f32 {
    let mut total_reward = 0.0;
    let mut obs = env.reset();

    loop {
        let action = agent.act(&obs);
        let (next_obs, reward, done) = env.step(action);
        total_reward += reward;

        if done {
            break;
        }
        obs = next_obs;
    }

    total_reward
}
