use crate::env::errors::EnvError;
use async_trait::async_trait;
pub trait Env: Send {
    type Obs: Send + Clone + 'static;
    type Act: Send + Clone + 'static;
    type Info: Send + Clone + 'static;

    fn reset(&mut self) -> Result<Self::Obs, EnvError>;
    fn step(&mut self, act: Self::Act) -> Result<(Self::Obs, f32, bool, Self::Info), EnvError>;
    fn close(&mut self) -> Result<(), EnvError>;
}

#[async_trait]
pub trait AsyncEnv: Env {
    async fn reset(&mut self) -> Result<Self::Obs, EnvError>;
    async fn step(
        &mut self,
        act: Self::Act,
    ) -> Result<(Self::Obs, f32, bool, Self::Info), EnvError>;
}

/// A trait for environments that support contextual interactions.
///
/// This trait extends the basic `Env` trait to add support for managing
/// context across multiple steps.
///
/// The `Context` type represents the state of the environment that persists
/// across multiple steps.
///
/// ## Example
/// ```
/// struct PokerContext {
///     // Game history
///     betting_rounds: Vec<BettingRound>,
///     revealed_cards: Vec<Card>,
///     
///     // Opponent modeling
///     opponent_tendencies: HashMap<PlayerId, OpponentModel>,
///     recent_bluff_patterns: Vec<BluffEvent>,
///     
// Strategic state
///     table_image: TableImage,  // how others perceive you
///     current_strategy_mode: StrategyMode,
/// }
/// ```
pub trait ContextualEnv: Env {
    type Context: Send + 'static;

    fn reset_with_context(&mut self, context: Self::Context) -> Result<Self::Obs, EnvError>;
    fn step_with_context(
        &mut self,
        action: Self::Act,
        context: &mut Self::Context,
    ) -> Result<(Self::Obs, f32, bool, Self::Info), EnvError>;
    fn get_context(&self) -> &Self::Context;
}
