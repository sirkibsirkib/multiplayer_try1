# multiplayer_try1

## Concept

This was a first attempt at a generic network-based game engine. Instead of focusing on the graphics and game logic (which would lean heavily on rs Piston), this project concentrates on a robust server & multi-client model that gives the illusion of a local game to all participants. Under the hood, messages are being created, sent and interpreted to update the game state. 



## Discontinuation

This approach was ultimately shelved for now as a more assymetric approach would make more sense for the game I have in mind at time of writing: `Emergence`.

The next project will reuse several concepts from this project, but redesign the arrangement of components to be even more generic, this time leaning more heavily on serverside game logic along with a new client-side module specifically for compacting granular player actions into coarse updates to send to the server. Additionally, a more intuitive system will exist to support client-side actions that can be performed without server interaction.

