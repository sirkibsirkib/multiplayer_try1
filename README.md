# multiplayer_try1

As part of my never-ending attempts to create this fuzzy game idea of Emergence,
this is the first time I am really hitting it bottom-up with multiplayer in mind specifically.

This project will attempt to create a very minimal but robust multiplayer game engine.
At time of writing, the following is in place:

1. Enities are indirectly referenced via an "Entity ID". This allows for an easy means of serializing messages that can be understood accross the network
1. "Remote Informant" actor that is called once per update() tick. This actor checks for "Diffs" (changes in the game state) locally and remotely and generates Diff messages to send and receive appropriately
1. Opaque interface between world / game logic and the rest of the network engine. This creates a conceptual bottleneck for the Remote Informant to monitor
1. Contention points are protected by condition variables to avoid "busy waiting"
1. Support for the game to be playable for server AND clients. Additional mode for single-player where engine remains the same but has a stub Remote Informant that generates no Diffs.

The following features are envisioned in the near future:

1. "Meta Message" system for managing synchronization. For instance, upon receiving a Diff representing a manipulation of the game state referring to an unknown Entity ID, the engine can ask the remote informant to push a Query into the network. After a time, the response will come back asynchronously and provide the necessary information
