# error273

A zero-player game. A generator burns wood in the middle of a frozen disc, citizens haul wood and game in from the rim, sleep in the houses they built, eat what they hunted, and vote on what to put up next. There is no input and no way to intervene. Nothing regrows, so every run ends with the colony frozen. Rust on Bevy 0.19, running headless (`default_app`, no window, no GPU); the only output is ASCII redrawn to the terminal with ANSI escapes.

## A frame

```text
              .........              
           .......T.......           
         ................Y..         
        .....................        
      .........W...............      
     ...........................     
    ..t...........@...........t..    
    ..............@..............    
   ...............................   
  .................................  
  .................................  
 .Y................................. 
 ................................... 
 .................z................. 
................H...H................
.....................................
..............z.......V..............
..............W......................
.T....@......z....#....z...........t.
..............W......................
...W.............@@......H...........
..............z.......z.@............
.............@.z....z@...............
 ............+....z..@.z............ 
 ....................@.............. 
 ...............H.@.z.............Y. 
  .................................  
  ............W....................  
   ...............................   
    ..............@..............    
    ..t.......................t..    
     ..........W................     
      ............@............      
        ........W............        
         ..Y................         
           .......T.......           
              .........              
tick   380  year 1  Spring  day 16  hour 20
pop  42  fuel   52  food   48  wood   73  game  200
House  15  Hut   1  Boiler   0  project House 1/12  vote 31/11/0
```

| Symbol | Meaning |
| --- | --- |
| `#` | the generator, at the centre of the map |
| `H` | a house, sleeping up to three citizens |
| `V` | a hunter's hut |
| `B` | a boiler, raising what the generator can burn to |
| `+` | the plot the current project is going up on |
| `T` | a treeline with wood still standing |
| `t` | a treeline cut out |
| `Y` | a hunting ground with game left |
| `y` | a hunting ground hunted out |
| `@` | a citizen, empty-handed |
| `W` | a citizen carrying wood |
| `F` | a citizen carrying game |
| `z` | a citizen asleep at home |
| `.` | open ground |

Three lines sit under the map. The first is the tick counter and the date it works out to. The second is the living population and the four quantities that decide the run: fuel in the generator, food in the granary, wood still standing, game still standing. The third counts what has been built, names the project in progress with how much timber it has swallowed so far, and gives the last ballot as votes for house, hut and boiler in that order.

## Running it

Needs a stable Rust toolchain.

```bash
rustup toolchain install stable
cargo run
```

It redraws in place about twelve times a second and exits on its own once everyone is dead. Ctrl+C ends it early.

```bash
cargo test
```

## What the simulation does

The map is a disc of radius 18. The generator sits at the centre, the treelines and hunting grounds on the rim, and everything the colony builds goes up on rings between them.

One tick is a game hour, twenty-four hours a day, thirty days a season, four seasons a year. Every rate below is written in those units in the source and converted through the clock, so the real-time tempo and the length of a day are separate knobs.

### Needs

Warmth, rest and hunger are the same kind of thing: a level from 0 to 100 where 100 is satisfied, a rate it slips at, a rate it recovers at, and two thresholds. A citizen starts acting on a need when it falls to the low mark and stops when it climbs back to the high one; the gap between the two is what keeps anyone from oscillating on a doorstep. A citizen does whatever their most pressing need calls for, and pressure is measured the same way for every need, so the comparison is meaningful. Cold and hunger kill at zero. Tiredness does not.

Warmth gains 2 an hour in the heat and loses 1 an hour in the cold, and acts below 25 until it is back to 75. Rest wears down by 24 over a working day and comes back at 4 an hour asleep, acting below 40 until it reaches 90. Hunger slips by 7 a day and is settled in one sitting, acting below 30.

### The fire

Ambient is -30 everywhere. The generator adds up to 66 on top of that at its own cell and 3 less for every cell of distance, so a well-fed fire holds the middle of the map at +36 and the air reaches zero about twelve cells out. It only burns that hot when at least twenty units are stacked up; below that the grate is banked low and the warm ring shrinks with the woodpile, which is why the collapse feeds itself. Each boiler raises the ceiling by 18.

It burns one unit every four hours, plus another for every twenty citizens it has to warm and another for every boiler. Growth is paid for twice: once in timber, then forever in fuel.

Sleeping under a roof cuts the bleed of body heat by a third. It is shelter, not a stove. Once the fire is too low to warm the centre, citizens stop walking to it and go home instead.

### Hauling

Eight treelines of 50 units and four hunting grounds of 60 stand on the rim, 640 in total, and nothing regrows.

A citizen picks which of the two to fetch by comparing each store against what the colony wants to hold per head, and keeps to that choice for the whole trip. They only change their mind when the other store has fallen a clear margin behind, because a round trip takes thirty-odd hours and a workforce that turns around every time a number crosses its target arrives too late in both directions. That margin is counted in hauls rather than units, so it widens as huts make each trip of game worth more: the slack a store gets is always about the same number of trips.

Wood goes on the fire. Game goes in the granary, and every hunter's hut standing means a carcass comes back dressed rather than dragged whole, so one haul is worth more.

### Building

The colony runs one project at a time, on the lowest free plot. Work starts only when there is timber to spare and a reserve still standing behind the fire, and while a project is open the wood that would have gone on the fire goes to the plot instead: the same log is either warmth now or a roof later. Timber past what the project needs goes on the fire rather than being lost.

What goes up is voted on. Each citizen casts one voice for whatever answers whichever of their own needs has fallen furthest through its band -- a house for rest, a hut for hunger, a boiler for cold -- and the highest tally wins, with ties resolved in a fixed order so the same colony always decides the same way. With every bed taken the colony builds a house regardless: that one is not a preference but the precondition for growing at all.

The mayor's office is a weight per building type, added to the tally before it is read. It gets no vote of its own, only a thumb on the scale, and it starts neutral. There is no logic in it and nothing writes to it yet: it is the seam where a policy layer from outside the colony reaches in.

### Growing

Every eight hours, a warm, rested and fed colony with fuel and food put by and a bed standing empty takes in one newcomer. That is the whole tension: the same wood either keeps the square warm, or becomes the roof that lets the colony grow into needing more of it.

## Roadmap

- Seasons that actually bite.
