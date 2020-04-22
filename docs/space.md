# TODO

- orbital travel is stil broken

    [Sector 1]
     Red Star 0.00
     - Dune 150.00
       - Light Transport 151.00 <
       - Station 2.00
     - Asteroid Field 80.00
     - Jump point 200.00



- look/examine transport do not work
- sector generator
- wait time between ship operations
- cockpit, need to sit to control the ship
- compute time travel time, now all flying move is 5 seconds
- replace coord system by orbit by and orbit distance
- replace ship moviment by burn, travel, retroburn, cyclo burn etc
- secttormap
    - show numbers
    - show coords
    - allow to move by numbers
- time to land, time to launch
- see inside a Vehicle

# Design

- build a ship like Aurora 4x
- giving the components a ship is created with many rooms
- hire a crew
- land and fly around
- as factorio, everything that player do need to be automatized 

## Orbit travel stage

- Align for mass/trust seconds
- Ejection burn DV seconds
- Drift distance / DV
- De-burn DV change
- Sync orbits mass/trust seconds
 
### Times

Maneuvering - Is moving the ship in current orbit, like align, change orbit or synchronize. 

Burn - To make ejections and de-burn.
 
### Speed modifier

A float value 0 can be defined the % of the burn.

Align burn can only be increment in second to save fuel
Ejection burn multiply the time, but divide the travel time

## Landing and Launch

Launch:
Atmosphere bypass
Taking speed
Circulization

Land:
Align
Atmosphere braking
Landing

## Orbit movement through hierarchic bodies

Find root of both places, make diff from the root, sum others.

### Case 1

Star 0
- Earth 200
  - Luna 5
    - Trade 0.1
- Mars 320

Luna -> Earth ->  Star -> Mars
0.1 + 5 + abs(200 - 320) = 125.1

### Case 2

Star 0
- Earth 200
  - Luna 5
    - Trade 0.1
  - ISS 0.2

Luna 0.1 + abs(Earth 0.5 - ISS 0.2) = 0.4

Star 0
- Earth 200
  - Luna 5
    - Trade 0.1
- Mars 320
  - Olympus 2
     - Olympus Station 0.2

Luna 0.1 + Earth 5 + abs(Star 200 - Mars 320) + Olympus 2 + Station 0.2 = 127.3 

###  Case 3 

## Galaxy

Galaxy is like a big sphere, where start systems can jump anywhere in the borders. Player can stay in the border systems
that have low probability to find someone else. Or they can jump in direction of the center, that giving th amount
of systems decrease, bigger chance to find other players.

In direction of the center, more dangerous, and more rewards.

## Start systems

Each player start into a unique start system outside of may galaxy. Only player knows its coordinates, others can only 
go there if he shares it. When your system coord are shared, now way to block access (maybe cyno jammers)

## Atmospheric landing

- each bodie has defined atmosphere pressure value
- each vehicle has a aerodynamic value
- ratio is defined by vehicle aerodynamics divided by bodie atmosphere pressure
- aerodynamics 0-1 are smoothing shaping, from 1-2 are wings like structures. 
- the ration defined a exponential how much fuel is require to land or take off

## Diplomacy

user defite a value between -10 to 10 between each other player
each player have some karma value that is defined by player actions
aggressive actions count to decrease hhe karma if any player have a positive relation
drastically if the attacker has positive (traitor)
change in relations take time. To have no penalties you need to have -10
each day you can move one point in any direction
karma increase by number of people have positive stand.
Negative karma is much strong that positive. If you mess up three times, is hard to come back

## Crew

Each group of components in a craft require crew to manage
Ship components are furniture in rooms. They can be examited, managed and repair
lack of crew cause % performance degradation in every task.

## DV require to land and move between planets

https://camo.githubusercontent.com/78adc73bf13274c230318a14f1cc34bdb7337b28/687474703a2f2f692e696d6775722e636f6d2f4141474a7644312e706e67