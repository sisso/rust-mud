# Todo

- Between Server <> Controller/View <> Domain

## Features


# Design

## Whiches

- running multiples controller at same time, local, socket and webext
- web controller should receive and reply json
- we can have multiple engines for multiple games
- for each game, we should have multiple controllers for each server type

## Modules

Main modules

- Servers
- Controllers
- Views

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


# ECS

# Engine

## Furniture

Normal furniuture

## Mob

Intelligent controlled by server

## Region
- label

## Zone
- label
- region

## Room
- zone
- label
- description
- exists

Contains  

## Container
- object list?

Can contains objects. A room, mob or container.

## AtContainer
- container_id

## Avatar

Controlled by player

## Item

General item. Can be pickup, dropped, put into a bag, etc. 

## Weapon

Can be used as weapon

## Wear

Can be wear

## SizeWeight

- size m3
- weight kg
