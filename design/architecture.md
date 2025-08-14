# A high level overview of the architecture

## RL primer

The core idea in reinforcement learning (RL) is a state-action loop. An agent observes the state of its environment, takes an action according to its policy, and gets some feedback (e.g., new state and reward) about the consequences of its actions. The goal is to update the agent's policy so that it achieves the most reward in the long run. In deep RL we represent the agent/policy as a deep neural netwok. Environmental states are inputs into that neural network and the outputs are actions. For example, if the environment is Pac-Man, the inputs may be images of the screen (RGB matrix) and the output is a vector indicating if we should move up,down, left, or right. By playing enough (generating enough trajectories/rollouts) and having a smart learning algorithm to update model weights, we'll make the neural network an expert Pac-Man player.

Nowadays the policy network is often a large language model. LLMs are huge and require lots of resources to
run and train. There are all kinds of tricks to make training more efficient, but the fundamental ideas are the same.

## Ferrum

Ferrum is a framework for doing RL. Our goals are:

- Create an intuitive API 
- Ensure a clean separation of concerns so we the core RL ideas are clear and we can swap implementations
- Keep resource utilization as high as possible
- Make it easy to experiment with various domains and training methodologies.

### Core `Ferrum` ideas

Ferrum is built around four key ideas:

- **`Actor`**: Distributed workers that run multiple environments in parallel. Actors collect observations from environments, send them to Inference for actions, step the environments, and package the resulting experience into trajectory shards for the Learner.

- **`Inference`**: Generates actions given observations. Takes batches of environment states and returns the actions the policy would take, along with the probabilities of those actions (needed for learning).

- **`Learner`**: Implements the algorithm that updates the model weights. Receives trajectory shards from Actors, runs learning updates (e.g., PPO), and produces new model checkpoints.

- **`Coordinator`**: Orchestrates the distributed system. Tracks the current model version, tells components where to find each other, and coordinates model weight updates across the system.

```markdown
Actor → observes environment → sends obs to InferenceClient
InferenceClient → returns actions → Actor steps environment  
Actor → collects TrajectoryShard → sends to Learner
Learner → updates weights → coordinates via Coordinator
```

### The challenges

#### LLM size

LLMs are huge and environments are expensive. This introduces a lot coordination and resource utilization challenges. For example, our environment (`Actor`) might be idle waiting for actions from the agent (`InferenceClient`) or training (`Learner`) might be waiting around for data from a slow environment. Also, the various components may be running on different hardware in different places. For example, the `Learner` may be training one version of the model on a GPU while we're running inference on three other GPUs and we have 1000 environments executing in parallel on 10 CPUs.

#### Context

We want to multi-turn context-rich environments. RL is naturally multi-turn. LLMs introduce specific issues with growing context/state as trajectories play out. For example, an LLM is probably going to want to know the full (ever growing) conversation history before deciding on an action.

The big "resource utilization" goal for `Ferrum` is to minimize idle time for every component in the system. We're trying to keep the interfaces clean so that we can implement optimization ideas when we have them.

```markdown
Turn 1: "What's 2+2?" → Context: ["What's 2+2?"]
Turn 2: "What about 3+3?" → Context: ["What's 2+2?", "4", "What about 3+3?"]
Turn 50: "And 50+50?" → Context: [entire conversation history...]
```

#### Notes on current implementation/optimizations

- `Actor`s will send `TrajectoryShard`s to `Learner`s so we can learn on whatever data we have without waiting for a "batch" of finished trajectories.
- Since GPU hardware is heterogeneous, and we may want to make special inference optimizations (e.g., use lower precision model weights to speed up calculations), we need to correct for that in `Learner`. In practice it means that, even for on-policy algorithms, we need to account for model differences. This means we need to implement some kind of importance sampling. (See FlashRL)
- `Learner` and `InferenceClient` GPUs need to sync weights periodically. There are many ways to do this. We want to keep our interface general enough to allow for this.
- Experimenting with "double buffer" to keep inference GPUs busy. For example: have inference running, get updated weights from `Learner`, quantize the weights while we're still generating, swap weights when quantization is done. The idea, as always, is to do as much work as we can in the background to keep resource utilization high.
- Using Rust. The idea is we'll be able to keep all the glue code and environment execution as efficient and parallelizable as possible.
