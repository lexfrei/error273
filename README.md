# error273

A zero-player game. A generator burns wood in the middle of a frozen world with no edge, citizens haul wood and game in from the ground around it, sleep in the houses they built, eat what they hunted, vote on what to put up next, raise children, and grow old. There is no input and no way to intervene. The treelines and hunting grounds grow back in the seasons that allow it, so a colony that keeps within its means can live through winter after winter, and one that outgrows the hands it has to feed itself does not. Rust on Bevy 0.19: the default build draws ASCII on square cells in a window, and building without it leaves the same simulation printing status lines to stdout and nothing else.

## A frame

```text
.................................................
....T............................................
...................................T.............
.................................................
.................................................
.................................................
.................................T...............
.................................................
.....T.......................T...................
.........................Y.......................
.................................................
.................................................
.........T......................................Y
..........W...............................Y....Y.
....T............................................
............W............@.......................
.................................................
.................................................
.................@...............................
.......Y................H........................
....Y............W....H...@......................
.......................W.......................T.
.................W..z.@.....H....................
..................W....W.........................
........T..........z....#....H...................
.................................................
.................+.....W..W....H.................
....................H.@.@...W....................
.....................H....H......................
...................F....z....H...................
...............................W.................
......................H...H......................
.................................@...............
...............F.................................
.................................................
.................................................
...........@.....................................
.................................................
..........Y............................@.........
............................T....................
.............................T..................Y
.................................................
...........T.....................................
..........T......................................
.................................................
.................................................
.................................................
.................................................
..T..............................................
tick   356  year 1  Spring  day 15  hour 20  air -26
pop  32  out  0  fuel   64  food   47  wood 1329  game  796
House  17  Hut   0  Boiler   0  project House 0/12  vote 26/1/0
under 15   2  grown  20  over 40  10  couples  12  raised 0.52/0.46/0.53
eldest #30  52y  str fair   wit good   hard fair    watched  11d (a guess)
```

That is the whole frame: forty-nine cells across, the hearth in the middle, and world in every direction past its edges. The glyphs are the window's, listed below; the five lines under the map are exactly what the headless build prints.

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

Five lines sit under the map. The first is the tick counter, the date it works out to, and the air outside the fire's reach. The second is the living population, how many of them are working past the edge of the frame, and the four quantities that decide the run: fuel in the generator, food in the granary, and the wood and game still standing on the near ground -- the twenty-four cells around the hearth the colony lives off. A total over the whole world would only report how far the colony has wandered, so those two columns are the ground the colony works rather than the ground that exists. The third counts what has been built, names the project in progress with how much timber it has swallowed so far, and gives the last ballot as votes for house, hut and boiler in that order. The fourth is the age pyramid -- children, grown citizens, the frail -- how many couples the colony has, and the middle of the colony for each of the three stats. That last is an aggregate and not a reveal: what any one citizen is stays hidden.

## Running it

Needs a stable Rust toolchain.

```bash
rustup toolchain install stable
cargo run
```

That opens a window and draws the map on square cells, twelve and a half ticks a second, with the status lines under it. A terminal character box is about twice as tall as it is wide, which made a step sideways read as shorter than a step down; on square cells distances read true. Every cell is tinted by how warm it is, so the fire's reach is a thing you can watch shrink through autumn. It exits on its own once everyone is dead; Ctrl+C ends it early.

### The headless instrument

```bash
cargo run --no-default-features
```

The same simulation at the same step with no window and no map: the status lines go to stdout, one set per tick, and nothing else does. This is what the quality gates and the balance log read, which is why its output is treated as an interface rather than a display.

```bash
ERROR273_TURBO=1 cargo run --no-default-features
```

Same again with the wait between ticks dropped, so it runs as fast as the machine will go. A citizen takes fifteen game years to grow up and a life is about fifty-five; at eighty milliseconds a tick that is hours, and here it is minutes.

### The tempo

```bash
ERROR273_TICK_MS=40 cargo run
```

How long a tick lasts in real time, in milliseconds, default 80 and held inside a sane range. It changes only how fast you watch: a tick is one game hour whatever it is set to, and nothing in the simulation is measured against the wall. Ticks fall on an absolute grid -- the nth tick is due n steps after the colony was founded -- so a slow frame is made up rather than pushing everything after it late.

### The tests

```bash
cargo test
```

The simulation's decisions are pushed into pure functions so they can be checked without running a colony, which is why the suite finishes in a moment. Building without the window drops the window's dependencies and runs a handful of tests the windowed build has nothing to check.

## What the simulation does

The generator sits at the hearth and everything the colony builds goes up on rings around it. The ground beyond carries the treelines and hunting grounds, and it does not stop. The window is a frame on that world rather than the whole of it: it holds the twenty-four cells around the hearth that the colony lives off, and anyone who goes further is off the frame and counted on the status line instead of drawn.

One tick is a game hour, twenty-four hours a day, thirty days a season, four seasons a year. Every rate below is written in those units in the source and converted through the clock, so the real-time tempo and the length of a day are separate knobs.

### Needs

Warmth, rest and hunger are the same kind of thing: a level from 0 to 100 where 100 is satisfied, a rate it slips at, a rate it recovers at, and two thresholds. A citizen starts acting on a need when it falls to the low mark and stops when it climbs back to the high one; the gap between the two is what keeps anyone from oscillating on a doorstep, and it is the gap rather than the marks themselves that is held fixed. A citizen does whatever their most pressing need calls for, and pressure is measured the same way for every need, so the comparison is meaningful. Cold and hunger kill at zero. Tiredness does not.

Warmth gains 2 an hour in the heat and loses 1 an hour in the cold. Its two marks are the exception to being fixed: they are the cost of walking home from where the citizen is standing, plus a margin of 25, so at the fire they are the 25 and 75 they always were and they rise with every cell walked away from it. A mark that does not know how far the fire is has no term for the walk: at fifty cells out, turning for home at 25 is turning on schedule, walking correctly, and dying twenty-five ticks short. The gap between the marks never moves, which is the part that stops anybody oscillating on a doorstep. Rest wears down by 24 over a working day and comes back at 4 an hour asleep, acting below 40 until it reaches 90. Hunger slips by 7 a day and is settled in one sitting, acting below 30. Eating is the exception to one-thing-at-a-time: a hungry citizen standing at the granary takes a meal whatever else is on their mind, so nobody starves on a full store because the cold happened to be worse.

### The year

The air outside the generator's reach follows the calendar: a cosine through the year, mildest in the middle of summer and harshest in the middle of winter, drifting a little each day rather than stepping at a season boundary. A colony gets one winter to learn on -- the first is scaled back and the cold reaches full depth by the third -- and the ramp is on the cold half only, so summers do not get worse with the years.

Treelines and hunting grounds grow back a little every day towards a cap of their own, in spring, summer and autumn. Nothing grows back in winter, which is what makes winter a thing to be survived rather than worked through: whatever is standing when it starts is the whole of what there is until spring. For the same reason the colony will not break ground on a project in winter, and will not break ground at all unless it holds half again what it wants per head behind the fire.

### Looking

A colony spares a few hands for looking rather than working, and the qualification is having nothing the matter: a citizen the survival tier of the vote has nothing to say about is a citizen whose day the colony is not already spending. They walk a leg at a time in a direction and a distance drawn from the world seed, and the distances are heavy-tailed -- half the legs are longer than twice the shortest, a tenth longer than ten times it, and the rare one runs a very long way. That shape is the search-theory answer for sparse targets worth coming back to; it is the mathematics that is borrowed and not the animal behaviour it was once argued from, because the tracks behind that argument turned out to be an artefact of the sensor.

Nothing orders a scout home. What brings them back is the same leash every citizen carries: warmth's marks rise with every cell they put between themselves and the fire, so a scout turns for home when the walk back is all the warmth they have left, and no rule about expeditions had to be written to make that happen. A scout brings back the map and nothing else -- ground they have stood in becomes ground the colony can work, and the colony never unlearns it, even after the square itself is dropped from memory and drawn again.

### The fire

The generator adds up to 66 on top of whatever the air is doing at its own cell, and 3 less for every cell of distance, so on an average day a well-fed fire holds the freezing line about thirteen cells out and in deep winter closer to nine -- inside the treelines, which is when hauling starts to cost body heat. It only burns that hot when at least twenty units are stacked up; below that the grate is banked low and the warm ring shrinks with the woodpile, which is why the collapse feeds itself. Each boiler raises the ceiling by 18.

It burns one unit every four hours, plus another for every twenty citizens it has to warm and another for every boiler. Growth is paid for twice: once in timber, then forever in fuel.

Sleeping under a roof cuts the bleed of body heat by 30%. It is shelter, not a stove. Once the fire is too low to warm the centre, citizens stop walking to it and go home instead.

### Hauling

The world has no edge. It is generated in squares of sixty-four cells, each one worked out from the world seed and its own coordinates alone, so a square can be asked for in any order and the same world comes back. Treelines and hunting grounds are scattered through it at about the density the old founding rings ran at, so the ground around the hearth holds what it always did. They are not a fixed store either: they grow back towards their own caps in every season but winter, which is what makes the year rather than the map decide how long a colony lasts.

What is further out is worth more. A patch's cap is flat out to the old rim and rises past it, until about a hundred and eighty cells out where it stops. A trip out there does not bring home more at once, since what a citizen can carry is set by the citizen; it means the ground gives up more before it is bare, and grows back to more.

A citizen looking for work asks for the nearest patch of the kind they want, and the world answers by opening squares in the order they could possibly hold something close. Each square knows how much of each kind stands in it, so a stripped one is passed over without being looked inside, and a square whose nearest cell is further away than what has already been found is never opened at all. What a search costs is therefore set by how far the answer turns out to be, not by how far a citizen is willing to walk or by how much world exists -- which is what keeps the cost of a tick from following the size of the map. Nobody looks past a hundred and eighty cells, because past there the ground stops getting better for the walk.

The colony works only ground somebody has stood in. A founding party knows the squares its near ground touches and nothing further, and a square nobody has walked to holds nothing anyone can fetch, however rich it is and however close the search would otherwise say it is. And the colony only draws the world it looks at: a square is worked out the first time somebody looks inside it and dropped again once nobody is near, because the seed can always redraw it. What the colony took is the one thing the seed does not know, so every cut is kept apart from the squares and put back on top -- a stripped treeline is still stripped when the colony walks away and comes back.

A citizen picks which of the two to fetch by comparing each store against what the colony wants to hold per head, and keeps to that choice for the whole trip. They only change their mind when the other store has fallen a clear margin behind, because a round trip takes thirty-odd hours and a workforce that turns around every time a number crosses its target arrives too late in both directions. That margin is counted in hauls rather than units, so it widens as huts make each trip of game worth more: the slack a store gets is always about the same number of trips. The comparison is also made against what each store will hold once everyone already fetching that kind gets home, not against what is in it now, so a shortage pulls over as many haulers as it needs and no more.

Wood goes on the fire. Game goes in the granary, and every hunter's hut standing means a carcass comes back dressed rather than dragged whole, so one haul is worth more.

### Building

The colony runs one project at a time, on the lowest free plot. Work starts only when there is timber to spare and a reserve still standing behind the fire, and while a project is open the wood that would have gone on the fire goes to the plot instead: the same log is either warmth now or a roof later. Timber past what the project needs goes on the fire rather than being lost.

What goes up is voted on, and a citizen votes in two tiers that are never weighed against each other. First survival: whichever need went unanswered longest and deepest since the last ballot -- unanswered meaning the hours it stayed short after they were already doing something about it, so a tiredness slept off and a hunger eaten cost nothing, and only control failing shows up at all. If anything shows there, that is the vote. A citizen the colony is holding up for reaches the second tier, efficiency, and votes for whatever wastes the most of their day: the hours spent walking back to the fire instead of working, which is what a boiler is worth. A citizen with nothing the matter abstains, and a contented colony can leave the docket empty. The highest tally wins, with ties resolved in a fixed order so the same colony always decides the same way. With every bed taken the colony builds a house regardless: that one is not a preference but the precondition for growing at all.

The mayor's office is a weight per building type, added to the tally before it is read. It gets no vote of its own, only a thumb on the scale, and it starts neutral. There is no logic in it and nothing writes to it yet: it is the seam where a policy layer from outside the colony reaches in.

### Growing old

A citizen ages one year for every year on the calendar, so the date on screen never contradicts anybody's age. Nobody dies of being old: hardiness is whole until about forty and then falls away towards a span of their own, scattered about a tenth either side of the expected one, and every night a citizen spends out in air below freezing rolls once against whatever hardiness they have left. Winter is what does the killing, it takes the frail first, and it takes them on ordinary working days rather than all at once at the end.

Children are mouths, not hands. They answer their own needs -- they eat, they sleep, they come in from the cold -- but they do not haul, and they do not count towards the workforce a hauler is judging the stores against. How fast they grow up depends on how warm and well fed the colony is, so the shape of the age pyramid is an output of how the colony is doing rather than a number anybody set.

### Who a citizen is

Every citizen has three things the colony never prints a number for: strength, wits and hardiness. They are not rolled at birth. A child banks the warmth and the food the colony actually had, hour by hour, and that ledger is settled at four named ages -- two, six, eleven, and the day they are grown. The first years weigh most, because that is where a hungry childhood does its lasting damage, and the settlement is multiplicative: the same good year is worth more to a child who already has something to build on, and a bad start cannot be bought back at full price.

What a body lost it can partly make back. Strength and hardiness keep a catch-up that runs out over the years after adulthood, so a colony can repair some of a cohort it half-starved; wits close on the day the child is grown and do not reopen. Each stat is raised on what actually feeds it -- the body eats, hardiness is warmed, wits take whatever there was of either.

Hardiness is the one a citizen keeps working on. A body left out in air below freezing learns the cold over a season and forgets it again over three, on top of whatever the childhood set, so a colony that works its haulers through winters is a colony of people who stand them better -- and a mild few years takes it back. What stands between a citizen and a cold night is therefore three things at once: what age has left them, what they were raised on, and what the winters since have taught them.

How long a citizen lives is settled the day their childhood ends, not the day it began: what the colony raised in them, and a part it cannot account for, weighed against each other by how comfortable the colony was while raising them. A colony that was short of everything can read the differences between its people straight off the granary; a comfortable one raises people who differ for reasons it never saw. So two children of one good decade still come out different, and somebody who walks in already grown has only that unaccountable part, since the colony never saw their childhood -- which is what the founding party is made of.

### What a citizen does

Every grown citizen has a trade -- the treelines, the hunting grounds, or none, which means going wherever the colony is shorter. Roughly a fifth of the workforce is deliberately kept in that last group, because a colony with nobody spare has no way to fill a vacancy, and nobody ever quits a trade of their own accord. A vacancy therefore goes to one of the best few of the spare hands rather than to the best of them, scored on how far they are from the work, how much they have practised it, and whichever way the mayor's office happens to be leaning. Distance is measured in king moves, which is exactly the travel time here and deliberately the wrong distance for anything else.

Nobody quits, but a store that has gone short for long enough stops being anybody in particular's problem: once a posting has stood unfilled for twenty days the colony will take somebody out of another trade for it, which is the only way a first bad guess at a workforce ever gets corrected. Practice is kept up by doing the work and rusts when it stops, so churning people between trades costs something. A child takes the trade of the oldest grown citizen in their house on the day they are grown, with a fraction of that person's practice already in hand -- which is how a trade can quietly die out with a household.

What a citizen brings to the work is what the colony raised in them, multiplied by how well they fit the trade, and one day multiplied by how well they are holding up. That last is wired in and set to one until there is a mood to put in it. It decides what a trip brings home, and it is the only place any of this turns into capacity rather than just into who survives a winter. The part of a load somebody is owed but cannot carry is kept for their next trip rather than rounded away, so a small difference in what the colony raised is a small difference in what comes back.

### What the colony thinks of somebody

None of those three numbers is ever printed. What the colony prints is one word per stat, and the word is measured against the middle of this colony rather than any fixed scale, so the same person is described differently in a different colony. It is also explicitly an opinion. Before the colony has watched somebody work, the word is a guess drawn from the childhood it did watch -- a child of a fat decade starts described better than they may turn out to be, which is a better kind of wrong than starting from nothing -- and the guess is marked as one. As the days of watched work add up the description converges on what the citizen actually is, and stops being marked.

The fifth line under the map is one such card, for the oldest citizen in the colony.

### Growing

Each couple gets one roll every eight hours at a chance that adds up to a set probability over a season, scaled by how many beds are standing empty, and only while the colony is not already carrying its share of children who cannot yet work. A colony with room grows; a full one tails off rather than stopping dead, which is the point: births gated on a bed arrive as a cohort, and a cohort of births is a cohort of deaths a lifetime later. A colony that is cold, worn out, hungry, or short of fuel or food has no children at all, and neither does one whose stores are sliding rather than holding, nor one that could not fetch enough for another mouth with a winter's margin left over. Stores say what has been fetched; only the last of those asks whether anyone is left to keep fetching it, which is how a colony can starve with its treelines untouched.

That is the whole tension: the same wood either keeps the square warm, or becomes the roof that lets the colony grow into needing more of it.

## What it does not do yet

- Nobody weighs the walk home against the cold before setting out. A citizen acts on warmth when it falls to the low mark, wherever they are standing, so the far ground the world now generates is ground it would be lethal to work.
- The boiler is in the ballot and has never been built. Across a run of six hundred game years the second tier of the vote -- the one a boiler answers -- never carried a ballot.
- The frame only ever watches the hearth. There is nothing else worth following yet, and a camera that follows an expedition arrives when there are expeditions.
