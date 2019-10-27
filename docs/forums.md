# ECS

## Indexes

AtContainer is one of components that need to be indexed for fast access for many 
commands like: list at room, list my inventory, look in chest.

The system responsible to change AtContainers need to keep the index up-date.

Example of systems:

ItemSystem 
- get, put, drop

MoveSystem
- n, s, e, w
