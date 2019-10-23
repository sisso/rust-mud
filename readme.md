# Todo

## Features

- weapon
- equip
- armor
- buy 
- sell
- experience
- level up
- potions
- multiples commands like: s e n f or sssee
- hire companions
- advanced parse commands like "examine drunk body" || examine body || examine body.2 || examine all body?
- load predefined world from file
- persistence save and load
  - pre-requiste to define a proper load and save format

## long improvements

- there is no server buffering to send and receive messages, if user can not read, is possible that we lose output or stuck

## Refactoring

- acceptance test do not need to use server
- normalize inventory in more generic way
- better layering between view commands, game logic, container, etc.
  - simple channel of Commands in (including ticket) and outputs out. All parsing and serialization of messages need to 
    be done by controller layer
- entity based, no Mob or Item, just Entity

# Design

## Whiches

- running multiples controller at same time, local, socket and webext
- web controller should receive and reply json
- we can have multiple engines for multiple games
- for each game, we should have multiple controllers for each server type

## Modules

- core-engine: share between all engines, basic domain classes and utilities
- $game-engine: game specific rules and running code
    - mud-engine
    - mud-share-engine
    - factorio-engine
    - space-engine        
- core: share between all modules, basic domain classes, macros and utilities
- socket-server: socket handling, can return new connections, disconnects, send bytes and receive bytes
- web-server: http connection, session handling, new session, timeout session, send json, receive json
- core-controller:  share between controlers
- $game-$server-controller
    - mud-http-controller
    - mud-socket-controller
    - space-http-controller
- server: import everything and runs. from command configuration read servers, controllers and engines to run.

## MVC

### Global

Main code
- run multiples controllers (current, socket, web)
- current and socket share most of functionally in command line module
- messages from controller and engine are Events

### Messages types

Messages are very raw. From input usually will contain just slitted strings. From return some useful things like label, 
but the controller should be able to look up into engine if needed.

### Flow inside controlers

Go to a view, view execute parsing and dispatch message

### Inside engine

action_<kind> -> <domain> -> <container>

tick:
game -> <domain> -> <container>

[] How separate better? currently most of functions are outside of container but same module. We need Business -> Repository


###  Tick flow

Main -> controller 1..* -> engine

### Message flow

Using ECS, main class just add all events into the system. Then run all systems  to process messages.

None ECS, the main class must route into proper business/action to execute the actions 


# Forum

## Send Msg(targets:vec<connection>, msg: String)

Send messages for multiple clients only work in very small scale, most of times the final string will be always 
a custom message that include things like player $hp$ anyways.
  
Easy to just simplify and optimize later (as always)

## Body as item, or mob?

Feature: change body to a dead mob, it will be never possible to store all info into Item. Many things like resurrect will be broken.

Have a body as a mob will create difficulties like carrie a body? should we carry a mob?

In the end, is not about be a Mob or a Item, but both elements need to be able to be mixed. A item is a mob, a mob is item. You can carrier a mob, or you can kill a item. This bring us back to the component based discussion.

## File format and Serialization

Considering that we will need to have save and write anyways. Why not just use save format for definition? 

A game is just a collection of configuration files, init or load we just load default list of files, then merge, them read.

Hocon
- is a mergeable by default
- support variables and references

## Changes in persistent world

In the case we have a long running persistent. How can I add new areas? obs and objects?
- the way will be use the data files that will always be loaded before the persistent files.

 

## DoAction and messages

Testing with do_pickup can improve a lot. But how to deal with comm messages? Most of time all Error enuns will need
to fetch the information again just to show the error. For instance.

PickError.ItemNotFoundIn { inventory_id }, we will need to fetch the inventory again just to show whats options
are available and why we can not found it.
