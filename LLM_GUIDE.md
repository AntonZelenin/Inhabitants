# Architecture summary (for LLMs)

This is a strategy game written using Bevy game engine. For now, it is developed as a single-player game, 
but the architecture is designed to add support to multiplayer features easier.

Rules:
- **Systems must be thin**  
  Bevy systems only:
  - read input / ECS state
  - call business logic
  - apply results to ECS  
  No real decisions inside systems.

- **Business logic location**  
  - Default: `logic.rs`
  - If large: `logic/` module with multiple files
  - Logic lives next to the feature, not globally. 
    E.g. `combat/logic.rs`, `economy/logic.rs`.

- **Business logic rules**
  - Must NOT use: `Commands`, `Query`, `Res`, `World`, ECS scheduling
  - May use: Bevy math/types (`Vec3`, `Quat`, `Transform`) if convenient
  - Operates on data, returns decisions / effects.

- **Future multiplayer / SpaceTimeDB**
  - Logic should be callable without ECS.
  - Adapters apply logic results (Bevy now, server later).
  - Event/effect mindset for persistent world changes.