# Todo

## Features

- normalize inventory in more generic way
- advanced parse commands like "examine drunk body" || examine body || examine body.2 || examine all body?
- load predefined world from file
- entity based, no Mob or Item, just Entity

## Forum

### Body as item, or mob?

Feature: change body to a dead mob, it will be never possible to store all info into Item. Many things like resurrect will be broken.

Have a body as a mob will create difficulties like carrie a body? should we carry a mob?

In the end, is not about be a Mob or a Item, but both elements need to be able to be mixed. A item is a mob, a mob is item. You can carrier a mob, or you can kill a item. This bring us back to the component based discussion.

### File format and Serialization

A experiment we can use toml format to defined world, entities and objects.

The main issue is not a friendly format for writing.

Considering that we will need to have save and write anyways. Why not just use save format for definition? 

A game is just a collection of configuration files, init or load we just load default list of files, then merge, them read.

Hocon
- is a mergeable by default
- support variables and references

## Refactoring

- better layering between view commands, game logic, container, etc. 

# Design

## MVC

- Controller and view can have knowledge about connection id, but should not be expose internally to game.

inputs:
Socket -> ServerRunner -> Runner -> view -> action_<kind> -> <domain> -> <container>

tick:
game -> <domain> -> <container>

[] How separate better? currently most of functions are outside of container but same module. We need Business -> Repository
