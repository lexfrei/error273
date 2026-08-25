# error273

A zero-player game. A generator burns wood in the middle of a frozen disc, citizens haul wood and game in from the rim, sleep in the houses they built, eat what they hunted, vote on what to put up next, raise children, and grow old. There is no input and no way to intervene. The treelines and hunting grounds grow back in the seasons that allow it, so a colony that keeps within its means can live through winter after winter, and one that outgrows the hands it has to feed itself does not. Rust on Bevy 0.19, running headless (`default_app`, no window, no GPU); the only output is ASCII redrawn to the terminal with ANSI escapes.

## A frame

```text
              .........              
           .......T.......           
         ................Y..         
        .....................        
      .........................      
     ...........................     
    ..T.......................T..    
    .............................    
   ...............................   
  .................................  
  ........@........................  
 .Y@@..@...@......F................. 
 ................................... 
 .................H................. 
..............W.z...@................
...............@..F..................
.......F......H.......H..............
.....................................
.T...........z....#....H.@.........T.
.................W...................
................@@..W....H...........
..............H......@H..............
...............H..@.H.W..............
 .................H....W............ 
 .......................@........... 
 ..........W...................@..@. 
  ........................W........  
  .......@.........................  
   .........................W.....   
    .............................    
    ..T.......................T..    
     ...........................     
      .........................      
        .....................        
         ..Y................         
           .......T.......           
              .........              
tick   356  year 1  Spring  day 15  hour 20  air -26
pop  32  fuel   55  food   23  wood  621  game  355
House  14  Hut   0  Boiler   0  project none  vote 28/2/0
under 15   2  grown  20  over 40  10  couples  12
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

Four lines sit under the map. The first is the tick counter, the date it works out to, and the air outside the fire's reach. The second is the living population and the four quantities that decide the run: fuel in the generator, food in the granary, wood still standing, game still standing. The third counts what has been built, names the project in progress with how much timber it has swallowed so far, and gives the last ballot as votes for house, hut and boiler in that order. The fourth is the age pyramid -- children, grown citizens, the frail -- and how many couples the colony has.

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

### In a window

A terminal character box is about twice as tall as it is wide, which draws the disc as an oval. The window renderer puts the same glyphs on square cells, so the disc is round, and tints each cell's background by how warm that square is: near-black at ambient, ember close to a fed generator, cooling again as the fuel runs down. The status lines sit under the map as before.

```bash
cargo run --features window
```

The feature is off by default. Without it the build stays headless, on the dependency set and the build time it has always had.

## What the simulation does

The map is a disc of radius 18. The generator sits at the centre, the treelines and hunting grounds on the rim, and everything the colony builds goes up on rings between them.

One tick is a game hour, twenty-four hours a day, thirty days a season, four seasons a year. Every rate below is written in those units in the source and converted through the clock, so the real-time tempo and the length of a day are separate knobs.

### Needs

Warmth, rest and hunger are the same kind of thing: a level from 0 to 100 where 100 is satisfied, a rate it slips at, a rate it recovers at, and two thresholds. A citizen starts acting on a need when it falls to the low mark and stops when it climbs back to the high one; the gap between the two is what keeps anyone from oscillating on a doorstep. A citizen does whatever their most pressing need calls for, and pressure is measured the same way for every need, so the comparison is meaningful. Cold and hunger kill at zero. Tiredness does not.

Warmth gains 2 an hour in the heat and loses 1 an hour in the cold, and acts below 25 until it is back to 75. Rest wears down by 24 over a working day and comes back at 4 an hour asleep, acting below 40 until it reaches 90. Hunger slips by 7 a day and is settled in one sitting, acting below 30. Eating is the exception to one-thing-at-a-time: a hungry citizen standing at the granary takes a meal whatever else is on their mind, so nobody starves on a full store because the cold happened to be worse.

### The year

The air outside the generator's reach follows the calendar: a cosine through the year, mildest in the middle of summer and harshest in the middle of winter, drifting a little each day rather than stepping at a season boundary. A colony gets one winter to learn on -- the first is scaled back and the cold reaches full depth by the third -- and the ramp is on the cold half only, so summers do not get worse with the years.

Treelines and hunting grounds grow back a little every day towards a cap of their own, in spring, summer and autumn. Nothing grows back in winter, which is what makes winter a thing to be survived rather than worked through: whatever is standing when it starts is the whole of what there is until spring. For the same reason the colony will not break ground on a project in winter, and will not break ground at all unless it holds half again what it wants per head behind the fire.

### The fire

The generator adds up to 66 on top of whatever the air is doing at its own cell, and 3 less for every cell of distance, so on an average day a well-fed fire holds the freezing line about thirteen cells out and in deep winter closer to nine -- inside the treelines, which is when hauling starts to cost body heat. It only burns that hot when at least twenty units are stacked up; below that the grate is banked low and the warm ring shrinks with the woodpile, which is why the collapse feeds itself. Each boiler raises the ceiling by 18.

It burns one unit every four hours, plus another for every twenty citizens it has to warm and another for every boiler. Growth is paid for twice: once in timber, then forever in fuel.

Sleeping under a roof cuts the bleed of body heat by 30%. It is shelter, not a stove. Once the fire is too low to warm the centre, citizens stop walking to it and go home instead.

### Hauling

Eight treelines of 50 units and four hunting grounds of 60 stand on the rim, 640 in total, and nothing regrows.

A citizen picks which of the two to fetch by comparing each store against what the colony wants to hold per head, and keeps to that choice for the whole trip. They only change their mind when the other store has fallen a clear margin behind, because a round trip takes thirty-odd hours and a workforce that turns around every time a number crosses its target arrives too late in both directions. That margin is counted in hauls rather than units, so it widens as huts make each trip of game worth more: the slack a store gets is always about the same number of trips. The comparison is also made against what each store will hold once everyone already fetching that kind gets home, not against what is in it now, so a shortage pulls over as many haulers as it needs and no more.

Wood goes on the fire. Game goes in the granary, and every hunter's hut standing means a carcass comes back dressed rather than dragged whole, so one haul is worth more.

### Building

The colony runs one project at a time, on the lowest free plot. Work starts only when there is timber to spare and a reserve still standing behind the fire, and while a project is open the wood that would have gone on the fire goes to the plot instead: the same log is either warmth now or a roof later. Timber past what the project needs goes on the fire rather than being lost.

What goes up is voted on. Each citizen casts one voice for whatever answers whichever of their own needs has fallen furthest through its band -- a house for rest, a hut for hunger, a boiler for cold -- and the highest tally wins, with ties resolved in a fixed order so the same colony always decides the same way. With every bed taken the colony builds a house regardless: that one is not a preference but the precondition for growing at all.

The mayor's office is a weight per building type, added to the tally before it is read. It gets no vote of its own, only a thumb on the scale, and it starts neutral. There is no logic in it and nothing writes to it yet: it is the seam where a policy layer from outside the colony reaches in.

### Growing old

A citizen ages one year for every year on the calendar, so the date on screen never contradicts anybody's age. Nobody dies of being old: hardiness is whole until about forty and then falls away towards a span of their own, scattered about a tenth either side of the expected one, and every night a citizen spends out in air below freezing rolls once against whatever hardiness they have left. Winter is what does the killing, it takes the frail first, and it takes them on ordinary working days rather than all at once at the end.

Children are mouths, not hands. They answer their own needs -- they eat, they sleep, they come in from the cold -- but they do not haul, and they do not count towards the workforce a hauler is judging the stores against. How fast they grow up depends on how warm and well fed the colony is, so the shape of the age pyramid is an output of how the colony is doing rather than a number anybody set.

### Growing

Each couple gets one roll every eight hours at a chance that adds up to a set probability over a season, scaled by how many beds are standing empty, and only while the colony is not already carrying its share of children who cannot yet work. A colony with room grows; a full one tails off rather than stopping dead, which is the point: births gated on a bed arrive as a cohort, and a cohort of births is a cohort of deaths a lifetime later. A colony that is cold, worn out, hungry, or short of fuel or food has no children at all.

That is the whole tension: the same wood either keeps the square warm, or becomes the roof that lets the colony grow into needing more of it.

## Roadmap

- Seasons that actually bite.
