# TODO

[] surface map and stuff instead sector and planets
[] 2d/3d surface maps (for aquatic and space)

# Design

- build a ship like Aurora 4x
- giving the components a ship is created with many rooms
- hire a crew
- land and fly around
- as factorio, everything that player do need to be automatized 

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