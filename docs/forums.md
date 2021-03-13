# Obj removal and referencing

When a obj is removed many references can be leaking like, avatar_id, item_id, owner_id, parent, chield, etc.

1. Clean up always properly
2. Ignore invalid references during load

# Error handling

Each method could return

- Ok
- Fail but handled the issue internally (log and user message)
- Fail and issue is not handled (expect the super guy to handle)
- Warning something is wrong, and it is handled
- Warning something is wrong and is not handled
- Error we should explode

# Mining

Player can buy equipment and deliver to planets/asteroids for mining. Once deliver in some place they will keep
mining. 

Maybe will require some maitenance like add fuel/power cells

Ore need to be collected from time to time

Surface miners for basic minerals

Deep miners for more rare ones

# Command / AI

Kind of AI
- None
- Protective
- Hauler
- Search

Player can command crew and robots to automate tasks. Or command himself to automate some task.

- Move to place 
- Load cargo
- Unload cargo

## Where other AI like mobs fit here?

### Hostlie mobs

ai { command_aggressive: true }

### Mercenary

ai { command_protect: "1" }

### Haulers

ai { 
  command_haul: {
    from:
    to: 
    wares
  }
}


## Examples

$ command
command what? candidates: robot1, robot2
$ command robot1
what command? move, rebase, attack, follow, hauler
$ command robot1 hauler
from where? (how hell someone will know? should we display ids?)
$ command robot1 hauler U32d
what? goods
$ command robot1 hauler U32d goods
to where?
$ command robot1 hauler U32d goods SH23
finally, good, let me move
robot1 picked goods from floor
robot1 enters in the little transport

$ command robot2 protect me
robot2 take your side and take alert position

$ commmand robot1 info
robots 1 command list:
0: haul goods from U32d to SH23 ( delivering 2 x goods)
1: stay  at SH23

## Model

```
pub enum AiCommandHaulerFailure {
    FailedFromRoomNotFound,
    FailedToRoomNotFound,
    ObjectNotFound,
}

pub enum AiCommandHaulerState {
    Idle,
    PickUp,
    Deliver,
    Complete
    FailedFromRoomNotFound,
    FailedToRoomNotFound,
    ObjectNotFound,
}

pub enum ObjSelector {
    // a single object
    One(obj_id),
    // list of objects
    Multiple(Vec<ObjId>),
    /// all in the room or in a container
    AllIn(obj_id),
    Tags(Vec<TagId>),
    And { a: Box<ObjSelector>, b: Box<ObjSelector> }
    Or { a: Box<ObjSelector>, b: Box<ObjSelector> }
}

pub enum AiCommand {
    None,
    /// hauler objects between locations
    Hauler {
        from: ObjSelector,
        to: ObjSelector,
        state: AiCommandHaulerState,
        wait_until_full: bool,
        repeate: bool,
    },
    /// will follow and protect the mob_id, used for hired units
    Protector {
        mob_id: MobId,
    },
    CanBeCommanded {
        command: Box<AiCommand>,
    },
    /// Will attack anything that didn't like
    Aggressive,
    Stay { room_id: RoomId },
    Sequence { commands: Vec<AiCommand> },
    Stack { default: AiCommand, current: AiCommand },
}

#[derive(Clone, Debug)]
pub struct Ai {
    pub id: ObjId,
    pub command: Command
}

ai.command = AiCommand::Stack {
    default: AiCommand::Stay { room_id: home_id },
    current: AiCommand::Sequence {
        commands: vec![
            AiCommand::Hauler { from: ObjSelector::AllIn(room_id), to: ObjSelector::AllIn(room_2), state: Default::default(), wait_full: false, repeat: false },
            AiCommand::MoveTo { room_id: home_id } 
        ]
    }
}

```

# Load and unloading cargo

- Each item will have weight
- Large cargo can be dragged by player to move to the Ship
- Each item should have a drag flag or number
- Initially player must manually drag its cargo, one by one
- A crew member or a drone can be commanded to load/unload the cargo
- Mob can have a "wield" amount, where you can hold a object and move without use your inventory. Wear an exo skeleton
  increase you wield max

# Vendor trade and economy

## Simplification

Just buy and sell by tag, with infinite

## Idea

Each vendor will have a list of trade goods. 

Price is a mult from the base item price

The min and max price depends of amount of stock if full (min price) or empty (max price)

We should be able to define:

- for each item tag
    - buy min mult and max mult
    - max stock
    - current stock
    - stock change over time

As each trade list can be very complex and have many entries, will be an indirection with empty objects containing
vendor store configuration, and a vendor can reference it. 

    
    {
        id: 1
        mob: {}
        vendor: {
            market_id: 2,
            stock: [
                {
                    "tag": "ore_iron",
                    "current": 300.0
                }        
            ]
        }
    }
    
    {
        id: 2
        market: {
            trades: [
                {
                    "tags": ["cloth"],
                    "sell": 1.0
                },
                {
                    "tags": ["ore_gold"],
                    "buy": {
                        min_mult: 0.9,
                        max_mult: 2.0
                    },
                    "max_demand": 100.0,
                    "change_per_cycle": -10
                },
                {
                    "tags": ["ore_iron", "ore_rust_iron"],
                    "buy": {
                        min_mult: 0.9,
                        max_mult: 2.0
                    },
                    "max_demand": 1000.0,
                    "change_per_cycle": -100
                },
                {
                    "tags": ["goods"],
                    "sell": {
                        min_mult: 0.5,
                        max_mult: 1.1,
                    },
                    "max_demand": 1000.0,
                    "change_per_cycle": 10
                }
            ]
        }
    }

# REST Server

A rest server need to be created to allow easy management of game state without directly json manipulation.

/api/v1/objects
GET  - list objects
POST - add new object and return new id
/api/v1/objects/<id>
GET - get a object
PUT - overwrite obectj
PATCH - update a object

# Room.exit

Use room.can_exit to get out of a ship or a ship get in/out of space looks mixing. But not totally wrong, a character
can exit of space ship, a vehicle can exit of space ship, a ship can exit from landdock.

# Timed actions

Conclusion: We keep option A) as it show it is less error prone and the flexibility provided by B) is to shallow to 
be usefull.

A)

    struct Obj {
        action: Action,
    }
    
    impl Obj { 
        fn is_complete(&self, time: TotalTime) -> bool { 
            self.action.is_complete(time)
        }
    }
    
    enum Actions {
        Move { end_time: TotalTime },
        Wait { end_time: TotalTime },
        Idle { },
    }

    impl Actions {
        fn is_complete(&self, time: TotalTime) -> bool { 
            match self {
                Action::Move { end_time } => end_time < time,
                Action::Wait { end_time } => end_time < time,
                _ => true,
            }
        }
    }
    
The more correct model, timing is attach to the action, only actions with time should deal with it. 

B)

    struct Obj {
        action: Action,
        end_time: Option<TotalTime>
    }

    enum Action { Idle, Move, Wait }

    impl Obj {
        fn is_complete(&self, time: TotalTime) -> bool { 
            self.end_time.map(|t| t < time).unwrap_or_else(false)
        }
    }
    
Dummy and flexible approach. The concept of end_time leaked outside of actions, actions that should have no time 
can have time, timed task can have no time.

This model can make sense if end_time is not part of action. As a busy time.

As usually we want all tasks to take a little time, it is a temptation. 

What if want to introduce a stunned time?
- we can just use the end_time to represent it even in the case of non timed actions
    - what happens if I am still doing some timed action?
        - only if we have it in different fields self.time and self.action.time allow full customization
        - still will be broken, since we stored total time, if a stunned effect is active it will cause a delay until
          we check for action timer, but the timer will finish in next tick
        - this completes kill the idea of use any scheduler
    - 

C)

    enum ActionKind {
        Timed { action: Action, end: TotalTime },
        Instant { action: Action }
    }

    struct Obj {
        action: ActionKind,
    }

    impl Obj {
        fn get_action(&self) -> Action { 
            self.action.get_action()
        }
    
        fn is_complete(&self, time: TotalTime) -> bool { 
            match self.action {
                ActionKind::Timed { action, end } => end < time,
                _ => true,
            }
        }
    }

Looks like the magic... and engineering. To fetch a action you need to deal with time, while mostly of times you want
to know the action and are not interested in the timing.

# UID

This is an admin identifier used in files for development reference some objects.

This code is user defined to easy access or reference.

Maybe this is a good use for tags. Or better, EditorTags?

!admin ls enemy fly

# Persistent State

Much of cause by trying to have a mix between static and persistent data. As mostly of MUD, is easy to 
just a persistent state.

## Prefabs

Any object can have a reference to a prefab. 
When persist, both object data and prefab data will be generated and only difference will be persisted. 
When load, first load the prefab object and then overwrite with field data.

- Removals are not supported !!!!
- prefabs can be recursive

## Use case

- I need to change all vendors to add a new attribute
    - A) using prefab, just update the base object
    - B) run a code to find all vendors and synchronize
    - C) respawn vendors: admin command to replace any spawnned object

## Children configuration

- children is used to reuse references during creation like a vendor or a ship
    - this is one usage feature, as with persistence files once initialize there is no reference anymore until a 
      parent/prefab mode is implemented.

## Plan

- A script will convert a current configuration into a default save game. If a profile has no save game, the default save
  game will be used.
  
- prefabs will still be just configuration for now. 
  
- No static/namespace kind of IDS. 

- any save game will be discard in model changes or need to be migration

## Load

## Prfab 

## Save

A main persistent like "Loader", can query different systems to fill some data. 

The model is the same as the loader, so if any internal data can be persistent (like next spawn), will be available 
for inital tool.

## Migration

- Load read savegame as JValue

- Load read the version, send to a migration function
    - migration function can manipulate directly JValue or have a complete new structure ObjV1 that manually copy
      to the new version

# Spawn in region

Instead spawn affect one or multiple rooms, it could have a ZoneId and spawn in all rooms in the regions.

- will not be used for random room generation since it require fine control, like "distance" from the door.

## Kinds of Spawns

a) spawn by a list of locations_id

b) spawn by parent room/zone

- become a bit painful to keep adding new zones exclusively to add a spawn
    - worse in case of dungeon that you can create new spawns

c) spawn by location and radius

- a lot of complication
- extra complication when try to tune the affected area

# Team & Relations

Should be hierarchic. A mob belong to a band that belong to a clan that work for a kingdom that just enter in war.

But for current use case, just don't attack the same type is enough. 

## Sample

    objects {
      team_nature_predator {
        id: 30
        label: "Team Nature Predators"
        team: {}
      }
    
      team_nature{
        id: 31
        label: "Team Nature"
        team: {}
      }
      
      team_undead {
        id: 32
        label: "Team Nature"
        team: {}
      }
    }


# Random dungeons 

- a pre-defined room and direction, can spawn a full random zone like dungeon of forest. Sam can be applied for searching.

- these zones are temporary and can be re-randomize or removed in case of no player is active there

  zone_1_dungeon_entrance: {
    id: 9,
    label: "Dungeon entrance"
    desc: ""
    room: {
      exits: [
        {dir: "n", to: ${objects.zone_0_forest.id} }
      ]
    }
    parent: ${objects.zone_1.id}
  }
  
  zone_1_dungeon_zone: {
    id: 10
    random_zone {
        entrance {
            id: ${objects.zone_0.forest.id}
            dir: "s"
        }
        
        size: [5,5]
        
        num_spawn_points: [3,5]
        
        spawn_points: [
            {
                prob: 1.0
                min_distance_entrance: 2
                space_distance: 2
                max_spawns: 2
                spawn {
                  prefab_id: [{prefabs.skeleton.id}, {prefabs.sekeleton_sword.id}]
                  max: 4
                  time_min: 10
                  time_max: 60
                }
            }
            {
                prob: 1.0
                min_distance_entrance: 4
                space_distance: 4
                max_spawns: 1
                spawn {
                  prefab_id: [{prefabs.spider_small.id}]
                  max: 8
                  time_min: 10
                  time_max: 60
                }
            }
        ]
    }
  }

This will generate a random map. We need to populate spawn points. Each spawn can be added with some min/max distance 
from entrance, and a radius of affect.

With the previous defined config. A new zone will be created with 5x5 dimension. The entrance will be through the
south of forest in zone 0.

Now it is time to create spanws.

It will sort a number between 3 or 5 spawns to be created.

It will sort one of spawns giving the prob.

Will search a place far away of `min_distance_entrance` from the entrance of the random zone and distance enough of
of any other spawn in distance of `max(this.space_distance, i.space_distance)`

# Modules support

Would be nice to organize bigger concepts into specific modules like Space, Fantasy, Room, SpaceShip, etc.

A new module require

- components and a repository
- input handling
- new events and triggers
- output handling

# Timers and Ticks

There are 2 type of timers. 

1) Timers: Are timers that need to happens in a specific point in feature, should not be postone

2) Ticks: Use delta time from last tick 

# Spawn with trigger

Check for trigger, then check for constraints, if succeed create the stuff, in any case, add to timer a new schedule

- how to deal with initial state? 
  - All spawn need to be initialize and create the default timer. 
  - If load a game, the timer will persist the timers
    - if a new spawn was added between reloads?
  - what to do if someone create a new spawn?
  - anyway I need maintenance task to check if a spawn was already initialize, how at least all spawns should have
    at least one schedule active
    
A central check every tick/time will still be require. Even if the spawn by scheduler, a 
check will still be require to confirm all spawns have timers.

Would be possible to have a trigger every time a new spawn is created? Both in bootstrap and dynamically? Spawn 
repository need to store all new entries that can be take.


# Event format

It is ququire to have a at least two collections to represent a Trigger. 

impl Trigger {
    fn schedule(event: Event);
    fn query(kind: Kind) -> Vec<Event>; 
}


Since rust dont give TypeId for Enum, it is not possible to use static code to map the type. 

The possible representations are:

a) 

const DecayKind: Kind = 0;
struct Decay {
    obj_id: u32,
}

b) 

enum Kind { Decay };
struct Decay { obj_id: u32  }

c)

enum EventKind { Decay };
enum Events {
    Decay { obj_id: u32 },
}

If mostly of cases Event will be just a obj_id, why I can not just use Event { kind: Kind, obj_id }



# Game receive independent commands v Input -> Output

A common flow simplify how things works. For instance, you don't need one advanced listener/trigger if we know that all messages are always processed later.

To create a commons flow, we should back to previous implementation where tick is Input -> Output. In previous impl Input contains new connections, inputs, time elapsed, everything. Output contains diconnects, commands and container.outputs.

Some operations like receive inputs and collect outputs are more user friendly by have direct commands, like connect()

Game can have some utilities mehtods, but controler, servvices and game loop should repsect the flow.

System should only run if there is a time updade.

## Multithread

Does this method will be still useful if we do multithread?

Yes, it will create a central point of control that will know how to split into parallel tasks. Anyway, receive random input anytime during gaming process will require locking

## Sequence

- disconnects
- connections
- inputs
- pre-system
- systems
- pos-system
- outputs

## Actions

Current actions are trigger direct from input like Pickup or Move. 

Ideally we should brake it int 2 steps, parsing and scheduling the action, a second flow to run the action.

Considering that any task will require time, everything will need to run into system anyways. 
- Need to be easy to schedule actions 

# Test layers 

Containers, allow to test actions

Game allow to test commands, login, loader, etc

What layer should be used to test containers + serivces? 

Looks like is missing the right abstraction to contain all game logic.

Game
- controller
- services
- repository

# Modules / Project Structure

Current:
game
game.loader
game.system

Desired:
game.controller - handle inputs and outputs, is used by view and interact with logic
game.view - is a "screen" that use can use, interact with controllers to get things done
game.models
game.logic
game.system

---------------|-------|
View           | model |
Controller     |       |
Logic | System |       |
---------------|-------|


controller
- input and output
game
- model and repositories
logic

## Why

Things like timer and trigger will only work well if some structured is well-defined. Currently, send_input is triggering
direct changes into the game model.

## Notes

- logic modules an independent code or traits added into that receive continaer or specific services in arguments

## Flow

- read incoming messages from server
- controller processes incoming messages into inputs or outputs
- game process inputs
- game process tick
- game process timer
- game process triggers
- controller.generated outputs from triggers
- send outputs to server

## How to achieve it:

- split logic from repository and moving to new module
- create in messages, inputs, outputs and triggers

## It was before?

This was exactly what we have before. The idea was break the big method into a small and more concise functions.

We can keep the same by just buffering events until tick is run

# Group money

Until a merge object is implemented, how to with collecting gold? 
- It could receive a especial treatment exactly how we are already dealing in vendor sell

# Merge objects

How do we deal with situation of merge objects? Like 10 gold coins, 2 swords,
1000 tons of Iron?

Can it be gui only?
- take iron.10 from 1000 iron is insane?
- can cause performance overhead

What should be attribute used to group it?
- label, or code?
  - since code is a array, we need to define priority and use the first one. Could work for ["sword", "broad"] and ["sword", "long"], but not for ["key","transport"], ["key", "house"].  Honestly, none of then look good.
    - only if use all codes
  - label could be used, but this mean that how I can distinguish two label if they code are the same? bringing back that it should be code.
 
Sometime dont't make sense to merge objects that re equals, same code, same label and same original prefab, it can contain different attributes like "use", "num of kills" or magic.
- real grouping should only happens for identical objects
  - not easy to solve in current multi component system

# Business logic

Currently mostly of logic is spread throuh handle inputs, execute commands and extra other operations. Put extra 
functions in the package is not a good idea because we want to reduce dependency from Repositories and Container. 

The idea is to split Component/Repository from bussines logic. This mean we need

1) create a new _bussines classes lilke

mob_service 
mob

2) move current model and repository into a new class and use the main one for business

mob
mob_repository

The option 2) will require more work, but adapt easy to logic that don't belong to any component or repository.

# Ctx or merge Outputs into container?

c) 

Considering that only few places are using Ctx (systems). Keep it or remove it is the lower effort to change for
now and if we decide otherwise. 

b)

Still CTX make sense, only with CTx we can move dyn references around without hardcode into container. Considering we
want to move container to hold just Repository. How will keep all references for services? If we decide to 
create a service holder?

a)

increase usage of CTX, or maybe just merge? In the long run outputs will be replaced 
by events. Events will belong to container. The Event -> string will be implemented
in another location. There is no point to ind

# When execute an action should the handler receive target or full list of raw argument?

If I want to send full arguments I would use just the main handle. If I decide to use a input handly directly is because
I already know what I want, I don't want to keep repeating.

# Long naming

Some entities can have long names like "John der Kleiner' or "Light Transport". Normally each entity will have a list of
code that can be used to reference. 

One idea was to accept long names when issue commands, like "pick john head from small pot". This mean that  mostly of
command needs to deal with some advanced parser instead accept just a Vec<&str>

Probably will just easy to wrap the input in some new type like Input.

    struct <'a> Input(pub &'a str)

    impl Input {
    }

    input.has_command(input, "enter") -> bool
    input.has_commands(input, &["enter", "e"]) -> bool
    let say_msg = input.plain_arguments()
    let list_ids = input.argumetns_pieces()
    let (from, to) = input.parse_two_arguments("from")
    

Required functions to operate with long naming:



# Internal Items, Invisible objects

b) 

At current situation, is not about be invisible or internal. Spawn just should never be visible. The same would
be apply for different objects. Some are always visible (mobs, items, furniture, etc) and some are always invisible 
(spawn, other rooms, doors, etc), or better, is not about be visible, is non existent for player.

This mean that will be easy to check case by case. Have a function that define if a entity should be visible? can be
pick up? can be fight? etc.

a) 

Where it should be define? Global as object or in label?

The situations where internal objects need to be filter are? Always??
- we should never have any method that list/give to user whatever is there, we always should check what we try to show.
- this means that look/examine/get/put/etc must always know what type of object is manipulating. 
  - Get/Put always operate on Item
  - Look? Mobs, Items, Carft,???
    - not very extensible. every time we need to add a new thing, look and examine need to be changed.
    
What if Label only contains label for look and desc for examine?
- ok, but we need some label to track objects. We can use code, but code is designed to user interaction.
- only objects with description can be describe?
  - ok, but a bit confuse
- if we are considering to use label to define what can be seeing. Make sense to just flag in label that this
  object is internal, can not be used with look or examine
- still don't look good :/

# Item type 

How very specifc items like GOLD?

This type of items require internal logic in many different parts of the systems, so they require to be referenced by
config files or have code flag.

Options to add int he code:

a) Boolean flags

Simple but verbose

can have issues to enforce mutual exclusivity.
- not a really issue, since all situations we have to choose, mutual exclusivity was always abandoned

b) Tags

It is basically same as A, but a more flexible and less optimized 

c) ItemKind

Dismissed since require mutual exclusivity

Conclusion:

Use flags and then move to Tags 

# Loot 

Loot is Items children that spawn 

To add random loot, we need to support some random children during spawn. 

The more flexible spawn system would be a probability tree. For instance:

    loot {
        2: {
          50: { item_id: [${reare_item_1_id}] }
          50: { item_id: [${rare_item_2_id}] }
        }
        
        28: {
          50: { item_id: [${uncommun_item_1_id}] }
          50: { item_id: [${uncommun_item_2_id}] }
         }
        
        70: {
          100: { item_id: [${common_item_1_id}, ${common_item_2_id}] }
         }
    }

# Timers

All non instantaneous tasks will require some timer to keep track. Currently almost every long
operation require a check every tick by nextTime < totalTime.

Event machine is easy. The main issues are:

1) Track of what need to execute when trigger

a) Store the closure to execute

b) Run a trigger

2) Serialize scheduled tasks


## Implementation

Each system can schedule tasks by (namespace_uuid) -> arguments.

namespace_uuid is used to cancel, check, notify the owner that timer has trigger. 

argument is used to forward variables.

namespace_uuid and all arguments need to be serialize. So the timer will restrict the types.

# Triggers

Triggers require that a event dispatcher and subscriber. 

Don't necessary means that they are callback. Triggers can be processed by a systems that iterate though all generated
events in normal execution.

In a ideal use case, the full architecture need to be model in a way that all messages become events. Triggers respond
to events. All player messages are basically a trigger listener.

# Even/trigger model

Each input get added is a trigger. Each input convert into a command by a listener. Each command trigger actions. 
Each action trigger stuff (move room, create body if kill). Each stuff trigger a output. Each output is write to 
players.

# Sequential vs lazy triggers

A trigger system can process any event existent in same tick. The order of systems is very important to process all
executed.

A lazy trigger will process all events generated in last tick. This is usually more stable but impossibility many
uses cases like "send messages to users when something happens" without using a high fps.

# Surface deprecation

In terms of MUD would be better to just a list of planets and movement between celestial objects by "injection burn" ->
"cycling burn" -> "orbital align" -> "dead transfer" ,etc.

Starmap 

$ sm
Sum
- Earth
  - YOU 
  - ISS (Station)
  - Luna (Moon)
    - Orbital (STATION)
    - Dragon (Cruiser)
- Cargo 1 (in flight to Luna)
- Venus

$ move Orbital
Calculating transfer.
burn cost 5.000 DV, new fuel is 15000 DV
Executing alignment burn.
Executing launch burn.
Launch complete, arrival in 38 seconds.
Computing re-orbiting
Executing cycling burn.
Transfer complete.

$ move sol earth luna orbital 200k 3
Calculating transfer.
burn cost 15.000 DV, new fuel is 5000 DV
Executing alignment burn.
Executing launch burn.
Launch complete, arrival in 12 seconds.
Computing re-orbiting at 200km
.
.
.
## Solar system structure

Negative                                               Positive
64 - 32 - 16 - 8 - 4 - 2 - 1 - 0 - 1 - 2 - 4 - 8 - 16 - 32 - 64
                              Sum
                               Mercury
                                 Venus
                         Earth
                       Moon
               Mars
                                            Netuno
       Urannu
       
# Posses, Connect to Matrix or VR a mech

These are all commands that will require change temporary or permanently the avatar of a player.

Player will need to keep a set of IDs:
- avatar: MobId - Represent the player in the world, the way to kill you. A human or the hive mind, it is your.
- monitor: Vec<MobId> - List of mobs that you activilly listen, all messages send to those mobs you will receive as 
  your own. Including commands failure like, you can not move N
- controlled: MobId - Is the mob_id that own any action you type

In general case, avatar, monitor and controlled are same id. A posses command will switch both monitor and controlled. A
camera system can add monitor. 

Maybe: Outputs will need to be extend to split private outputs between "controller only" and monitor

# Vendors

Vendors should contain a list of ID to sell, or should items have a tag that vendor can sell?
- each vendor will keep its own list. But the list can be centralized by reference files in hocon

# Error Codes

Lets just use generic: String

Argument - you send wrong stuff
Invalid  - something is not what should be, maybe just using the wrong argument
Warning  - something is not what was expected to be, something is wrong and need to be verify
Error    - something is failing miserably

For each of those categories, when require, we could create custom exceptions to be catch.

ParserArgument { cause: ParseArgumentCause } 

---

In general we really don't care about the error, we just want to easy return from the block and the caller decide 
if want to deal or not with the issue.

Sometimes we generate warn logs

Sometimes we use to give different messages

---

Categories

complete failures, like fail to write file
unexpected failures, like a expected reference is invalid
illegal argument, you provide illegal argument


|command                        | type|
|-------------------------------|-----|
|fail to write save game        | IOError    |
|expected configuration or id don't exists | NotFoundError |
|'get' without argument                    | InvalidArgument|
|'get xxx' where xxx dont exists                       | NotFoundArgument |
|'get item', item was found but Item type don't exists | InvalidState |
|'get item', but item is stuck | InvalidArgument |
|'get item', but character is in limbo| InvalidState |
| can not attack because is resting ||
| item can not be used as weapon ||


----

As first implementation is already showing, a centralized Error can get chaotic very shortly. A clear structure needto be defined, that can be easily recognize and only hold useful values. 

Most errors we want to propagate and exploded with meaningful error message. This case we will just use Generic. 
- and how it improves by simple unwrap? At least panic contains a full stack trace.

General policy:
- panic for unexpected errors
- any other case that can expected to have a success/failure we return a error code
- error codes need to be obvious, a Conflict(u32) moved 3 layers out where it was produced will not bring any useful
  information. If objects insert will return a Conflict, it will need to be map to Generic(String) to be propagated

# Loader / Prefab / Init / Serialization / Save

# Example of implement disarm in Circle mud

As example how a code could be much simple https://www.circlemud.org/pub/CircleMUD/contrib/snippets/skills/disarm.txt

The following code all required to a AI pick up a lost weapon after player cause it to disarm.

    if (LOST_WEAPON(ch)) {
      if (IN_ROOM(LOST_WEAPON(ch)) == IN_ROOM(ch)) {
        if (perform_get_from_room(ch, LOST_WEAPON(ch)))
          do_wield(ch, OBJN(LOST_WEAPON(ch), ch), 0, 0);
      }
      LOST_WEAPON(ch) = NULL;
    }

The same code will be require using our model

    if let Some(item_id) = container.lost_weapons.has_lost(mob_id) {
        if container.items.get_room_id(item_id) == container.items.get_room_id(mob_id) {
            if (actions::items::pickup(container, mob_id, item_id)) {
                actions::items::wield(container, mob_id, item_id);
                container.lost_weapons.remove(mob_id, item_id);
            }
        }
    }

# Reusable actions 

Currently all inputs are forward to actions that parse input, forward to internal command and map into container.outputs. For instance
equip(player, arguments), forward to do_equip
do_equip receive again (player, arguments) -> Result
equip then match the result and output messages.

most of the code is just searching, then ask the repository to execute the real actions.

This impl have the following issues:

- very inflexible, can only be used by players doing inputs
    - only for player_id
    - receive <args> instead a well-defined arguments
- the actions itself cannot be reused too, since will not generate output messages.

Solution

All inputs should be forward to a parser with the user, and arguments. The parser should then forward to a action with
the avatar mob, exactly require arguments. The action will be responsible to apply the change and generate the container.outputs.
- require what? like item_id? how should do a search the search?
  - probably a different parser method, in normal input we chain both. Internally we could decide between one
    or other.

# Body as item, or mob?

Feature: change body to a dead mob, it will be never possible to store all info into Item. Many things like resurrect will be broken.

Have a body as a mob will create difficulties like carrie a body? should we carry a mob?

In the end, is not about be a Mob or a Item, but both elements need to be able to be mixed. A item is a mob, a mob is item. You can carrier a mob, or you can kill a item. This bring us back to the component based discussion.

# DoAction and messages

Testing with do_pickup can improve a lot. But how to deal with comm messages? Most of time all Error enuns will need
to fetch the information again just to show the error. For instance.

PickError.ItemNotFoundIn { inventory_id }, we will need to fetch the inventory again just to show whats options
are available and why we can not found it.

# ECS Container, At Room

We can use a generic Container to hold other objects. Any object that can contain other object ( like all ??? ) will
implement it.

How do we represent rooms, inventory and mobs? How do we restrict a player not pick up other? 
- Weight? Volume?
    - so we can easy pick up small enemies?
        - if you have skill, what the problem if a giant pick you up?
    - we can say only pick items
    
# 3D MAP


....... 1) x - You    z+2 
.x..... 2) y - Planet z-1
.......
....y..
.......

1) You    ( 2, 2,  2)
2) Planet ( 5, 4, -1)

....... 1) x - You    z+2 
.x..... 2) y - Planet z-1
.......
....y..
.......

1) You    ( 2, 2,  2)
2) Planet ( 5, 4, -1)


# Ship Aerotinamic

To land into atm places

0.1-0.8 - smoth shape
0.8-2.0 - wings (bad for very dense? -% fuel?)

Atm thrusters
+100% fuel

Aquatic thruster

# Surfaces

Sectors, planets, mons and asteroids will implement surfaces.  Where space and aquatic reagions have 3d space.

Childrens of a Surface need to have position

SurfaceBody define peroperits of a body in surface. Like can be landaded? 

How to split planet atmosefpher and gravity from a city in a planet?

I have sector, that have planet, that have surface, that have city, that have rooms

I have sector, that have space station/craft, that have room

As code is just

if  me.get_location.is_surfac {
    locations.list(me.get_lcation).zip(positions) { obj , pos {
    }
} else if m.location.is_room {
}

sector {
    sector1 {
        label: "Sector 1"
        surface: { 
            kind: "space", 
            dimensions: 3, 
            bounds: 10 
            objects: {
                dune: {
                    location: "${surfaces.sector1.id}"
                    label: "Dune",
                    position: [2,3,0]
                    planet: {
                        atm: 0.8
                    }

                    surface: {
                        kind: "desert",
                        objects: {
                            palace_city: {
                                pos: [1, 4],
                                rooms: {
                                    palace {}
                                    landing-pad {}
                                    city {}
                                }
                            }
                        }
                    }
                }
            }
            station-1: { 
                station: {}
                rooms {
                    docking {}
                    airlock {}
                    quarters {}
                    bridge {}
                    galley {}
                }
            }
        }
    }
}


{
    "1" {
        surface {
        }
    }
    
    "2" {
        location: 1
        planet {
        }
    }
}

# Piloting

Many vehicles are planed to be available and allow player to move around.

Craft - to fly in space and land into planets
Vehicels - to drive/fly/dive into planets, moons, asteroids
Feets - to walk around rooms
Mechs - Personal Mech are wearing, Large mech are vehicles

# Msg to playerid

We never want to send message to players, we always want o send message to avatars. A player can monitor many avatars. 
For example, the person itself, a drone, the current vehicle and current ship.

A message to the ship will hit all people inside, a message to vehicle only for who is inside, for the character, only himself.

This means that we want to attach player_id, or better, connections, to receive any message to the attached resources.

Conclusion: 
- each mob can be watched by one or more player_id, messages are never to player only to mobs. 
- e should process all messages and map into mob_id

# Modules

Repository
- update indexes and check for consistency
- do not generate outputs

Actions
- execute operations like pickup or attack
- generate output

Handlers
- parse and execute actions
- interact directly with the user

View
- control user session

# Separation

Input -> View -> Handler -> Action
                  
Handler - parse player input,
Action - execute the actions and publish events       

# Tag

Tags can be used to identify objects to create relations. For instance, all items that are ore can have tag "ORE", and
a vendor can be created to sell "ORE" items tags.

While this can be done by tag, it make mostly pointless as it will have no logic impact.

Same could be used for crafting and production.

Tags will not be accessed directly by code like tags.contains("something"), if code need any information, it should
be defined by a flag.

## Implementation

String vs ID

As internally logic do not use strings, it can just operate over ids. When serialize it needs to be converted back to 
text, will be a pain, but intersting to see.

