# Todo

## Features

- change body to a dead mob, it will be never possible to store all info into Item. Many things like resurrect will be broken.
- normalize inventory in more generic way
- advanced parse commands like "examine drunk body" || examine body || examine body.2 || examine all body?
- move id counters to own unsafe incrementer?
- log
- acceptance test
- items

## Refactoring

- runner become game
- runner::Output to own package
- server_runner back to root 
- server to on module
- separate domain and outputs related in own modules tree? 

# Design

## MVC

- Controller and view can have knowledge about connection id, but should not be expose internally to game.

inputs:
Socket -> ServerRunner -> Runner -> view -> action_<kind> -> <domain> -> <container>

tick:
game -> <domain> -> <container>

[] How separate better? currently most of functions are outside of container but same module. We need Business -> Repository
