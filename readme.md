# Todo

- change body to a dead mob, it will be never possible to store all info into Item. Many things like resurrect will be broken.
- normalize inventory in more generic way
- advancaed parse commands like "examine drunk body" || examine body || examine body.2 || examine all body?
- move id counters to own unsafe incrementer?
- log
- acceptance test

# Design

## MVC

- Controller and view can have knowledge about connection id, but should not be expose internally to game.
