# -*- coding: utf-8 -*-
DATA = {}

DATA["mario"] = [
("mario","common",["Jumpman","Plumber","Red","Overalls","Itsa","Warp"],r'''
  .---.
 |o   o|
 |  ^  |
 | \_/ |
  /MMM\
''',{"idle":["waiting by the warp pipe","one-up mushroom on standby"],"working":["hammering through the level, brick by brick","stomping bugs like goombas"],"celebrating":["mamma mia! it compiled!","flagpole slide, perfect run"],"miffed":["the princess is in another codebase","that bug took my last life"]}),
("luigi","uncommon",["Greenie","Number Two","Weegee","Poltergust","Green Missile","Brother"],r'''
  .---.
 |O   O|
 |  ^  |
 | \_/ |
  /LLL\
''',{"idle":["standing a half-step behind you","haunted by the mansion wifi"],"working":["vacuuming warnings out of the logs","jumping higher, complaining less"],"celebrating":["luigi time! which is any time, really","the poltergust swallowed every error"],"miffed":["mama mia, not again","there are ghosts in the pipeline again"]}),
("peach","rare",["Princess","Parasol","Peachy","Toadstool","Turnip","Royalty"],r'''
  \♕/
 (o o)
  \ᴗ/
 /|♡|\
  / \
''',{"idle":["fanning gently with the parasol","planning the next turnaround"],"working":["turnips pulled, precision strikes","gliding over the hard part"],"celebrating":["a perfect turnaround, as expected","the crowd goes wild for the deploy"],"miffed":["this castle is NOT acceptable","someone forgot tea time"]}),
("bowser","epic",["King Koopa","Shellfire","Koopa King","Fang","Big Bad","Spike"],r'''
 /\___/\
( 0   0 )
 \  ^  /
 /|WWW|\
/_/   \_\
''',{"idle":["filling the dungeon with lava glow","breathing smoke while it waits"],"working":["fire-breathing through the build","gnashing at the dependency graph"],"celebrating":["KING OF THE BUILD","a roar that echoes down the pipeline"],"miffed":["SHOW ME YOUR PAPERS, COMPILER","someone is getting dropped in lava"]}),
("toad","uncommon",["Mushroom","Retainer","Spud","Fungus","Cap","Little"],r'''
 .-===-.
(* * *)
  (o o)
   \_/
  /| |\
''',{"idle":["guarding the cart, mostly napping","helpful, high-pitched, tiny"],"working":["TOADSTOOL TOOLKIT ENGAGED","grabby hands sorting everything"],"celebrating":["our build is in THIS repo, actually!","confetti! mushroom confetti!"],"miffed":["oh no, not the castle again","the little helper has OPINIONS"]}),
("yoshi","uncommon",["Dino","Egg-thrower","Flutter","Steed","Green","Longtongue"],r'''
  __
 /o \__
( ᴥ   )
 \___/
 /|  |\
''',{"idle":["eggs all accounted for","chewing a code fruit thoughtfully"],"working":["tongue out, bug caught mid-air","flutter-jumping over the flaky test"],"celebrating":["YOSHI! egg-cellent work!","swallowing the whole bug list"],"miffed":["bloop. sad dino noise.","that fruit was unripe, like that commit"]}),
("wario","rare",["Garlic","Greed","Gold","Bad Buzz","Wah","Anti"],r'''
  .---.
 |V   V|
 |  3  |
 | \_/ |
  WWWWW''',{"idle":["counting coins, ignoring you","garlic breath fogging the terminal"],"working":["working overtime for overtime pay","squeezing performance out of spite"],"celebrating":["WAH! treasure! deploy! WAH!","gold coins raining on the release"],"miffed":["WAAAH, who broke MY build","this loss comes out of somebody's paycheck"]}),
("waluigi","rare",["Lanky","Purple","Cheat","Leg Day","Sneaky","Anti-Luigi"],r'''
  .---.
 |>   <|
 |  3  |
 | \_/ |
  LLLLL''',{"idle":["looming in doorways","plotting, mostly stretching"],"working":["beating the benchmark, somehow legally","spinning through the test suite"],"celebrating":["WAA, finally, RECOGNITION","dancing badly and winning anyway"],"miffed":["WAA, nobody respects the waluigi","filing a complaint with the referee"]}),
("daisy","uncommon",["Flower","Sarasaland","Tomboy","Crown","Bloom","Hi-Im"],r'''
  \✿/
 (o o)
  \ᴗ/
 /|✿|\
  / \
''',{"idle":["smelling the server-room roses","crown tilted, confidence maxed"],"working":["hi, doing the work, no big deal","double-hipping through the tickets"],"celebrating":["HI, IT WORKED","victory pose, sparkles included"],"miffed":["hey! that was MY bug you sat on","the flower wilted, thanks a lot"]}),
("rosalina","epic",["Stargazer","Comet","Lumas","Cosmic","Observatory","Luma-mom"],r'''
   ✧
  /o\
 ( ᴗ )
 /|★|\
  / \
''',{"idle":["reading by comet-light","the lumas orbit the build queue"],"working":["rewriting star charts and test matrices","cosmic patience, infinite debugging"],"celebrating":["a new star joins the observatory","a launch worthy of a comet"],"miffed":["the lumas are asking what happened","even starlight flickers at this error"]}),
("donkeykong","epic",["DK","Barrel","Ape","Kong","Bongos","Big Guy"],r'''
 .-"""-.
( ◉   ◉ )
 \  ᴥ  /
  /|||\
 /_|||_\
''',{"idle":["beat on chest, beat on keyboard","banana hoard audited nightly"],"working":["barrel-rolling through the backlog","bongos keep the build tempo"],"celebrating":["DK! DK! DK!","bananas for everyone, even cranky"],"miffed":["somebody moved my bananas AND my merge","small primate energy detected in this PR"]}),
("diddykong","common",["Diddy","Little Kong","Peanut","Jetpack","Cap","Small"],r'''
  .---.
 (o   o)
  \ ᴥ /
  /| |\
   / \
''',{"idle":["practicing peanut popgun aim","small, but the tail knows things"],"working":["jetbarrel deployed on the long task","tiny arms, huge throughput"],"celebrating":["popgun salute to a clean build","hover-carried victory lap"],"miffed":["big primate problems, tiny ape","dixie would NOT have merged this"]}),
("goomba","common",["Chomper","Brownie","Stomper","Mushroom Guy","Walk","Little Brown"],r'''
  .----.
 /      \
|  o  o  |
 \______/
  /    \
''',{"idle":["walking. eternally walking.","flat-headed and earnest"],"working":["marching through the task list","head down, footsteps steady"],"celebrating":["paragoomba airborne with joy","the stomping finally stopped, victory"],"miffed":["stomped. again. classic.","becoming flat is my whole thing"]}),
("koopa","uncommon",["Shell","Troopa","Greenie","Bouncer","Kick-me","Shuffler"],r'''
  .----.
 (o    o)
  \____/
  /|██|\
   /  \
''',{"idle":["retracted into the shell, thinking","shuffling along the ledge"],"working":["spinning shell, clearing the corridor","kicked precisely where needed"],"celebrating":["shell slider, ten thousand points","bouncing off the wall with pride"],"miffed":["somebody ALWAYS kicks the shell","retreated, plotting round two"]}),
("boo","uncommon",["Shy Ghost","Blusher","Phasm","Wisp","Peek-a","Fadeghost"],r'''
  .----.
 (  o o )
 (  ___  )
  \_____/
    / \
''',{"idle":["covering its face politely","haunting the build server, shyly"],"working":["spooking the flaky tests away","transparent about everything except feelings"],"celebrating":["peek-a-BOO, it passed","glowing brighter than the LED strip"],"miffed":["stop LOOKING at me","hiding from the error messages now"]}),
("shyguy","uncommon",["Masked","Hood","Bashful","Eight","Snifit-adjacent","Waddler"],r'''
  .-----.
 |  o o  |
 |  ___  |
  \_____/
   /| |\
''',{"idle":["mask on, feelings tucked away","waddling in from the background"],"working":["spear and jetpack at the ready","muffling its screams productively"],"celebrating":["a very quiet celebration behind the mask","confetti thrown respectfully"],"miffed":["the mask hides EVERYTHING, especially this","shy guys do not cry, they sulk"]}),
("kamek","epic",["Magikoopa","Wand","Elder","Spellcaster","Four-eyes","Wizard-turtle"],r'''
   /\
  /★ \
 (o o)
  \_/
 /|_|\
''',{"idle":["staff glowing, glasses polishing","scrying the next build failure"],"working":["casting enlarge-spell on the test suite","hexing flaky tests into flakier ones"],"celebrating":["a spell of SUCCESS, as foretold","wand sparkle, new minion hired"],"miffed":["you DARE interrupt my scrying","turning this error into a bigger error"]}),
("birdo","uncommon",["Snout","Egg-spitter","Pink","Bowtie","Norbert","Ribbon"],r'''
  .---.
 (o   o)
  \ 3 /
   \_/
  /| |\
''',{"idle":["perched, bow straightened","snout honking softly"],"working":["spitting eggs at the backlog","bow-glammed and grinding"],"celebrating":["an egg-celent deploy, straight from the snout","ribbon twirl, egg scatter"],"miffed":["snout crossed, eggs withheld","the bow drooped, like the build"]}),
("kingboo","legendary",["Crown","Big Boo","Phantom","Sovereign","Icicle Crown","Dark Ghost"],r'''
 .-------.
(  ◉   ◉  )
(   _____  )
 \_______/
   /   \
''',{"idle":["crown hovering over a throne of fog","ruling the haunted level quietly"],"working":["sealing bugs into paintings","royal terror, minimal effort"],"celebrating":["the crown GLOWS, the build is MINE","a king's laugh echoes in the library"],"miffed":["YOU will be the next painting","the jewels dimmed with displeasure"]}),
("lakitu","common",["Cloud","Referee","Fisherman","Spinner","Lap-top","Sky"],r'''
   .--.
  (o  o)
   \__/
  /|  |\
  (____)''',{"idle":["floating on the cloud, flag ready","fishing for dropped frames"],"working":["announcing every lap of the build","dropping spinies on the blockers"],"celebrating":["NEW RECORD, well raced","the flag waves, the cloud bounces"],"miffed":["WRONG WAY, you absolute plumber","the cloud thundered once, quietly"]}),
]

DATA["pokemon"] = [
("pikachu","common",["Sparky","Bolt","Pika","Zap","Static","Cheeks"],r'''
  /^^\
 (o  o)
  \*/
 /|   |\
  /   \
''',{"idle":["cheeks sparking at idle wattage","nesting in the mousehole of the terminal"],"working":["thunderbolt on the failing test","tail up, current flowing"],"celebrating":["PIKA PIKA, full charge","static electricity high-fives"],"miffed":["PIKAAAA, sparks everywhere","cheeks puffed, charge building"]}),
("charizard","legendary",["Blaze","Flame","Wings","Ember-King","Dragontail","Skyfire"],r'''
  /\  /\
 /  \/  \
(  ◉  ◉ )
 \  ~ /
 /|    |\
  /    \
''',{"idle":["tired flame on the tail","perched on the volcano ridge"],"working":["melting blockers with dragon breath","wings aloft over the merge queue"],"celebrating":["a flamethrower of pure joy","victory loops with smoke rings"],"miffed":["fire hot enough to melt the diff","the flame on the tail turned BLUE"]}),
("bulbasaur","uncommon",["Bulb","Seed","Sprout","Vine","Four-legs","Plantback"],r'''
  \ | /
   \|/
  (o o)
  (___)
  /   \
''',{"idle":["bulb photosynthesizing calmly","rooted and patient"],"working":["vine-whip on the lint errors","solar-charging the compile"],"celebrating":["the bulb bloomed at last","seed of success fully grown"],"miffed":["razor leaf, aimed at nobody in particular","the bulb drooped in protest"]}),
("squirtle","uncommon",["Squirt","Shell-squad","Bubble","Tank","Jet","Little Blue"],r'''
  .---.
 (o   o)
  \ ᴗ /
 /|___|\
  /   \
''',{"idle":["backpack shell zipped shut","practicing water-guns at the wall"],"working":["hosing down the hot paths","water pulse, measured pressure"],"celebrating":["bubble beam of celebration","shell armor polished to a shine"],"miffed":["water gun at maximum sulk","withdrew into the shell entirely"]}),
("eevee","uncommon",["Potential","Fluff","Kitsune","Eight-way","Evolution-adjacent","Soft"],r'''
 /\   /\
/  \_/  \
| o   o |
 \ ᴗᴗ /
 /|   |\
''',{"idle":["fluff maximized, decisions deferred","napping on eight possible futures"],"working":["quick attack on quick fixes","adapting to every environment variable"],"celebrating":["praise! affection! possible evolution!","tail fluffed to full volume"],"miffed":["the fluff bristles","still deciding which evolution to be mad as"]}),
("jigglypuff","uncommon",["Puff","Balloon","Lullaby","Mic","Round","Singer"],r'''
  .---.
 / o o \
|   ᴗ   |
 \_____/
  / | \
''',{"idle":["inflated and smug","humming a half-lullaby"],"working":["singing the build to sleep, productively","marker ready for anyone who naps"],"celebrating":["a song of pure triumph","balloon bounce, victory lap"],"miffed":["you fell asleep DURING MY SET","drawing on your face in permanent marker"]}),
("gengar","epic",["Shadow","Grin","Sneak","Void","Nightshade","Cheshire"],r'''
 .-------.
(  o   o  )
 \  ▽   /
 /|     |\
  /   \
''',{"idle":["grinning from the shadow of the pane","hiding in the keyboard's dead keys"],"working":["hexing the race conditions","lurking near the segfaults, taking notes"],"celebrating":["a shadow that LAUGHS with you","dream-eater snack: the whole error log"],"miffed":["the grin went crooked","curse applied, no negotiation"]}),
("snorlax","rare",["Blocker","Nap","Mountain","Belly","Z","Sleepy"],r'''
 .-------.
(  -   -  )
(   ___   )
 \_______/
 /|     |\
''',{"idle":["asleep across the build path","snoring in perfect rhythm"],"working":["wide awake for exactly this","body-slamming the bottleneck"],"celebrating":["a nap as reward, well earned","belly rumble of satisfaction"],"miffed":["you woke the snorlax","rolled over onto your deploy"]}),
("mewtwo","legendary",["Clone","Psychic","Alpha","Laboratory","Origin","150"],r'''
   /\
  /  \
 (o  o)
  \--/
 /|  |\
  /  \
''',{"idle":["hovering, calculating everything","one-fifth aware of your presence"],"working":["psychic debugging, no breakpoints needed","bending the metrics toward the truth"],"celebrating":["this outcome was CALCULATED","telekinetic high-five, gentle"],"miffed":["the clones of my patience are exhausted","a psychic storm brews silently"]}),
("mew","legendary",["Origin","Myth","Bubbles","Glitch","Phantom-pink","Ancient"],r'''
  /\_/\
 ( o o )
  \ ᴗ /
   \_/
  /   \
''',{"idle":["blinking in and out of existence","curled in a bubble, ancient"],"working":["learning every move, instantly","invisible progress, visible results"],"celebrating":["a myth confirmed by a green build","bubble bounce of ancient joy"],"miffed":["rarely seen, currently VERY seen","vanished, taking your merge rights along"]}),
("psyduck","uncommon",["Headache","Duck","Confused","Goldie","Migraine","Blank"],r'''
  .---.
 (o   o)
  ( ᴗ )
   /|\
  /   \
''',{"idle":["holding its head, as always","vaguely aware of the deadline"],"working":["confusion attack, accidentally effective","headache-powered telekinesis"],"celebrating":["sudden clarity! brief! delightful!","hands off head for one full second"],"miffed":["headache intensifying","the quack of deep displeasure"]}),
("meowth","uncommon",["Talking Cat","Coin","Schemer","Payday","Scratch","Team"],r'''
 /\   /\
/  \_/  \
|  o o  |
 \  ω  /
  /| |\
''',{"idle":["polishing the charm on its forehead","planning something, benignly"],"working":["scratch cat, professional grade","fury swipes at the flaky tests"],"celebrating":["the boss will be SO pleased","coin flip: it's always heads"],"miffed":["we're blasting off agaaaain","that's the WRONG kind of scratch"]}),
("gyarados","epic",["Rampage","Leviathan","Tempest","Dragon-wave","Magikarp's Truth","Sea-dragon"],r'''
 ~~~~~~~
/\  ◉  /\
  \___/
  /|||\
 /_|||_\
''',{"idle":["coiled in the deep, waiting","storm-sense tingling at the horizon"],"working":["hyper beam through the blocker","the wrath of a thousand splashes"],"celebrating":["a calm sea, for once, well earned","raged, and it WORKED"],"miffed":["MAGIKARP'S WRATH UNLEASHED","the lake is empty, the lake is ANGRY"]}),
("lapras","rare",["Ferry","Gentle","Shell-song","Ice-wing","Voyager","Passenger"],r'''
  .----.
 (o    o)
 /|____|\
(  ____  )
  /    \
''',{"idle":["singing across the still water","carrying passengers, asking nothing"],"working":["ice beam at surgical temperatures","singing while it swims the data lake"],"celebrating":["a song carried over the whole bay","gentle giant, precise joy"],"miffed":["the song turned minor key","frozen silence, deliberately"]}),
("lucario","epic",["Aura","Seer","Steel-fox","Medium","Pulse","Sensor"],r'''
 /\    /\
/  \  /  \
| ◉  ◉ |
 \  ^  /
 /| | |\
''',{"idle":["reading the aura of the codebase","spikes calm, senses open"],"working":["aura sphere aimed at the real bug","metal claw, precise strikes"],"celebrating":["the aura is GREEN and bright","an aura storm of satisfaction"],"miffed":["the aura reads WRONG on this commit","sensing your distrust, reciprocating"]}),
("greninja","rare",["Ninja","Shuriken","Frogue","Water-dark","Tongue-scarf","Silent"],r'''
  /\/\
 (o  o)
  \__/
 /|~~~~|\
  /  \
''',{"idle":["scarf-tongue flicking, ready","a shadow in the water tower"],"working":["water shuriken, silent execution","shadow sneak behind the bug"],"celebrating":["a ninja star spun in triumph","the tongue-scarf curled in delight"],"miffed":["vanished, but the disappointment lingers","mat block: no thank you"]}),
("ditto","common",["Copy","Blob","Mirror","Anyone","Morph","Placeholder"],r'''
 .------.
|        |
| o    o |
|  .--.  |
|  '--'  |
 \______/''',{"idle":["a puddle of pure potential","practicing being a different rock"],"working":["transforming into the exact tool needed","copying the work, perfectly"],"celebrating":["became a party hat, somehow","morphed into a tiny victory"],"miffed":["transformed into a frown","the copy is imperfect and offended"]}),
("cubone","uncommon",["Bone","Lonely","Mask","Skull-helm","Marowak's Kid","Quiet"],r'''
  .----.
 / ◉  ◉ \
|   __   |
 \______/
  /|  |\
''',{"idle":["helmet on, memories on","tapping the bone club thoughtfully"],"working":["bonemerang: throws the task, it returns done","digging through the pile"],"celebrating":["a rare unguarded smile","bone club drumroll"],"miffed":["the helmet hides nothing today","that deadpan skull-stare"]}),
("dragonite","epic",["Gentle Giant","Orange","Mailman","Hurricane","Kind","Big-wing"],r'''
  /\  /\
 (o    o)
  \ ᴗ /
 /|----|\
  /    \
''',{"idle":["circled the globe, back for tea","drifting, amiable, enormous"],"working":["hurricane through the flaky region","delivering the fix across oceans"],"celebrating":["a barrel roll in the sunrise","thunderous joy, gentle eyes"],"miffed":["outrage. briefly. then a hug.","the mail was LATE, and dragonite knows"]}),
("vulpix","uncommon",["Six-tails","Foxfire","Ember-fox","Kitsune","Emberling","Warm"],r'''
 /\ /\ /\
( o o o )
 \ ᴗᴗ /
  /| |\
''',{"idle":["six tails fanned like a hand of cards","warm as a small furnace"],"working":["will-o-wisp on the warning pile","foxfire lighting the way"],"celebrating":["all six tails at full wag","a tiny inferno of delight"],"miffed":["tails bristled to six exclamation points","frost breath, barely justified"]}),
]

DATA["starfox"] = [
("foxmccloud","common",["Fox","Leader","Arwing","Blaster","Star-fox","McCloud"],r'''
  /\_/\
 / o o \
|   ^   |
 \_____/
 /|   |\
''',{"idle":["mission patch shined, wings folded","waiting in the hangar"],"working":["barrel roll through the merge conflicts","laser check, systems green"],"celebrating":["MISSION COMPLETE, all pilots home","a barrel roll, for victory"],"miffed":["commencing deep strike on this bug","peppy's voice in my head, unhelpfully"]}),
("falco","rare",["Bird-man","Ace","Smug","Blue","Co-pilot","Hotshot"],r'''
  \___/
 <o   o>
   \^/
  /|X|\
 /_/_\_\
''',{"idle":["leaning on the arwing, cool","polishing sunglasses in zero-g"],"working":["game on, hotshot","flying in, shooting first"],"celebrating":["personally, I prefer the AIR","a landing so smooth it's rude"],"miffed":["I'm finished flying with YOU","someone check the smart bomb budget"]}),
("peppy","uncommon",["Rabbit","Mentor","Veteran","Coach","Hare-brained","Old Pilot"],r'''
  /---\
 (o   o)
  \ ᴗ /
 /|===|\
  /   \
''',{"idle":["long ears twitching with wisdom","maps spread, coffee cold"],"working":["do a barrel roll (advice free)","old pilot, new bugs"],"celebrating":["great job, that's the way","the old war stories were TRUE"],"miffed":["nobody listens to the old rabbit","never give up. trust me on this one."]}),
("slippy","uncommon",["Toad","Engineer","Guppy","Fixer","Frog","Wrench"],r'''
  .---.
 (o   o)
  \ ᴗ /
 /|~~~|\
  /   \
''',{"idle":["wrench in hand, always","shield analyzer humming"],"working":["shields at forty percent, FIXING","inventing something from spare parts"],"celebrating":["shields full, systems perfect","the gizmo worked FIRST try"],"miffed":["someone get this guy off me","the wrench slipped, the bolt laughed"]}),
("wolf","epic",["Rival","Star-wolf","Fang","Hunter","O'Donnell","Predator"],r'''
 /\____/\
( ◉    ◉ )
 \  --  /
 /|////|\
  /    \
''',{"idle":["smiling like a wolf in a henhouse","claws sharpened on the merge queue"],"working":["hunting the bug, pack-style","no mercy for flaky tests"],"celebrating":["can't let you do that... unless it's a win","a rival's respect, hard won"],"miffed":["the pack is DISAPPOINTED","this failure will be remembered, fox"]}),
("leon","uncommon",["Chameleon","Poison","Lizard","Cool","Venom-wings","Blend"],r'''
  /----\
 (◉    ◉)
  \  ▽ /
 /|    |\
  /    \
''',{"idle":["tongue tasting the air, calmly","chameleon-calm, poison-ready"],"working":["dissolving the blocker in acid","blending into the background process"],"celebrating":["a job perfectly poisoned","invisible pride, visible results"],"miffed":["the tongue flicked once, meaningfully","venom is for emergencies, and NOW"]}),
("pigma","uncommon",["Greed-pig","Dengar","Money","Betrayer","Hog","Bounty"],r'''
  .-----.
 (o     o)
  \ 3_3 /
 /|=====|\
  /     \
''',{"idle":["counting the reward, again","snout deep in the contract fine print"],"working":["profit margins first, bugs second","an oink of effort, minimal"],"celebrating":["the bounty is MINE, all mine","a bacon celebration, fully self-aware"],"miffed":["paid by the hour, not the outcome","someone owes me money for this"]}),
("andrew","common",["Oikonny","Nephew","Wannabe","Small-time","Andross-jr","Schemer"],r'''
  /\___/\
 (o     o)
  \ ᴗ_ /
 /|     |\
  /     \
''',{"idle":["practicing uncle's evil laugh","sitting in the big chair, badly"],"working":["inventing schemes on the fly","empire-building, one bug at a time"],"celebrating":["uncle will hear of this VICTORY","the throne, briefly enjoyed"],"miffed":["this is ALL star fox's fault","the laugh needs more practice"]}),
("panther","uncommon",["Caroso","Charmer","Rose","Black","Poet","Silk"],r'''
  /\  /\
 (♥  ♥ )
  \ ᴗ /
 /|===|\
  /   \
''',{"idle":["rose between claws, smooth","a silk voice, sharp claws"],"working":["claw-swipe with a compliment","seducing the bug into surrender"],"celebrating":["for you, a rose and a green build","panther pride, purring"],"miffed":["the rose wilted, and so did I","claws out, poetry paused"]}),
("krystal","rare",["Telepath","Vixen","Staff","Blue","Cerinia","Listener"],r'''
  /\_/\
 (o   o)
  \ ᴗ /
 /|✧|\
  / \
''',{"idle":["staff humming with thoughts unsaid","listening to the mind of the code"],"working":["staff spells on stubborn bugs","reading the error's feelings"],"celebrating":["the thoughts are all joy, for once","staff raised, light woven"],"miffed":["I heard THAT thought, and I agree","telepathy does not help with merge conflicts"]}),
("generalpepper","common",["Pepper","General","Corneria","Top-brass","Commander","Salute"],r'''
  .-----.
 (o  o)
  \___/
 /|███|\
  /   \
''',{"idle":["medals arranged, briefing ready","cornerian flag at half-staff of coffee"],"working":["dispatching resources, crisply","the defense budget, fully deployed"],"celebrating":["the gratitude of a planet","medal ceremony, promptly held"],"miffed":["this is a matter of GALACTIC security","paperwork filed in triplicate, angrily"]}),
("rob64","rare",["ROB","Robot","Unit","Assistant","Beep","64"],r'''
  .-----.
 | [o] |
 |  _  |
  \___/
  /| |\
''',{"idle":["systems nominal, sarcasm sub-threshold","floating, blinking, waiting"],"working":["analysis complete, deploying","beep boop, task executing"],"celebrating":["victory probability: 100.0%","beep beep beep (jubilant)"],"miffed":["error. disappointment routine engaged","rebooting the patience module"]}),
("jamesmccloud","uncommon",["James","Ghost","Legend","Father","Vortex","Phantom-wing"],r'''
  /\_/\
 (o   o)
  \ ^ /
 /|===|\
  /   \
''',{"idle":["appearing where needed, then not","flying through the vein of memory"],"working":["guiding from beyond the radar","wingman of legend, rarely visible"],"celebrating":["a father's nod, brief and warm","the ghost in the machine, proud"],"miffed":["you were NOT watching the tail","the vortex swallowed the progress"]}),
("billgrey","common",["Grey","Bulldog","Wingman","Cornerian","Pal","Throttle"],r'''
  .---.
 (o   o)
  \ ᴗ /
 /|███|\
  /   \
''',{"idle":["paws on the throttle, easy","dogfight stories on repeat"],"working":["hold the line, I'm coming","covering your flank, always"],"celebrating":["that's my friend, that's MY win too","a growl of pure pride"],"miffed":["you left me holding the whole line","the bulldog bite is real"]}),
("kattmonroe","uncommon",["Monroe","Pink","Bomber","Free-lance","Feline","Purr"],r'''
  /\_/\
 (♥   ♥)
  \ ω /
 /|~~~|\
  /   \
''',{"idle":["pink fighter jet idling loud","independence worn like perfume"],"working":["bomb away, invoice pending","freelance firepower, delivered"],"celebrating":["cat-itude fully expressed","a purr, a wink, a clean build"],"miffed":["I charge extra for rescues","nine lives, none for this bug"]}),
("fay","common",["Spaniel","White-dog","Curly","Sweet-wing","Cadet","Bright"],r'''
  .---.
 / o o \
 \ ᴗ /
 /|♡|\
  / \
''',{"idle":["curls bouncing, eyes bright","flight log pristine"],"working":["new pilot, old soul","flying textbook-perfect"],"celebrating":["first solo flight, flawless","the whole squad cheered"],"miffed":["the textbook did NOT cover this","ears drooped, resolve held"]}),
("miyu","common",["Lynx","Wildcat","Feline","Fast","Lone","Sharp"],r'''
  /\_/\
 (o   o)
  \ ᴗ /
 /|★|\
  / \
''',{"idle":["sharp eyes, sharper claws","lounging like it's a cockpit"],"working":["claw-first problem solving","nine lives, spending one"],"celebrating":["a wildcat yowl of victory","leaping joy, landings optional"],"miffed":["claws out, patience gone","nobody tells the lynx what to do"]}),
("tricky","common",["Prince","Earthwalker","Dino-brat","Royal","Little","Stompy"],r'''
  /\  /\
 (o    o)
  \ ᴗ /
 /|___|\
  /   \
''',{"idle":["princely stomping in place","demanding attention, cutely"],"working":["fire breath on the blocker","digging for the root cause"],"celebrating":["prince of the BUILD, bow before","tricky tricky, in a good way"],"miffed":["royal tantrum, schedule pending","the crown is crooked and so is this build"]}),
("aparoidqueen","legendary",["Aparoid","Hive","Swarm","Many","One-mind","Queen"],r'''
  .-===-.
 ( ◉   ◉ )
  \  ▽  /
 /|#####|\
  /     \
''',{"idle":["a thousand eyes, one intention","the hive hums in unison"],"working":["assimilating the bug into the swarm","distributed computation, biological"],"celebrating":["the hive MARCHES, the build is OURS","a chorus of victory, one voice"],"miffed":["a thousand eyes turn toward YOU","resistance has been added to the hive"]}),
("andross","legendary",["Emperor","Face","Venom","Scientist","Final-boss","Void"],r'''
  .-------.
 ( ◉     ◉ )
 |   ___   |
  \_______/
  /|     |\
''',{"idle":["a giant face, serene and terrifying","hands folded in the void"],"working":["science beyond cornerian understanding","the brain computes, the hands build"],"celebrating":["ONLY THE RESULTS MATTER","a laugh that shakes the sector"],"miffed":["I've been defeated by... COMPILERS?","the hands clench, the void watches"]}),
]

DATA["smash"] = [
("link","rare",["Hero","Sword","Hyrule","Green","Shield","Silent"],r'''
   /\
  /oo\
 |  ^ |
 /|===|\
  /| |\
''',{"idle":["listening for the next dungeon cue","silent, as ever"],"working":["hacking through the backlog, literally","spin attack on the blockers"],"celebrating":["HYAAA! item get!","the master sword hums green"],"miffed":["hey, listen! this build FAILED","sealed away until the next patch"]}),
("zelda","uncommon",["Wisdom","Triforce","Sage","Light","Hylia","Scholar"],r'''
  ♕
 (o o)
  \ᴗ/
 /|◇|\
  / \
''',{"idle":["studying the ancient terminals","light arrow at the ready"],"working":["sealing the bugs with light","praying at the shrine of green builds"],"celebrating":["the triforce of WISDOM compiles","hylia smiles on this deploy"],"miffed":["the seal is weakening","ganon's code got merged AGAIN"]}),
("ganondorf","legendary",["King of Thieves","Dark Lord","Gerudo","Beast","Power","Villain"],r'''
  .---.
 (◉   ◉)
  \ ▽ /
 /|###|\
  /   \
''',{"idle":["the triforce of power hums ominously","shadow pooling in the corner"],"working":["conquering the codebase, kingdom by kingdom","dark magic on the dependencies"],"celebrating":["THE BUILD IS MINE, as foretold","a villain laugh that shakes the terminal"],"miffed":["the hero of time has MERGED again","sealed in the realm of failing tests"]}),
("samus","epic",["Bounty Hunter","Power Suit","Zero Suit","Chozo","Morph Ball","Hunter"],r'''
  .---.
 |[o o]|
 |  ▽  |
 /|===|\
  /   \
''',{"idle":["visor reflecting the terminal glow","charging the beam, slowly"],"working":["morph ball through the tight spots","missile lock on the blocker"],"celebrating":["mission accomplished, suit polished","an escape sequence with seconds to spare"],"miffed":["metroid DNA detected in this build","the visor fogged with frustration"]}),
("kirby","uncommon",["Puffball","Copy","Star-warrior","Pink","Inhale","Dream-lander"],r'''
  .---.
 (o   o)
 ( ᴗᴗ )
  /| |\
 /     \
''',{"idle":["round and content","sucking up loose snippets"],"working":["copy ability: acquired","inhaling the whole TODO list"],"celebrating":["a star rod of pure joy","dancing, roundly"],"miffed":["puffed up and displeased","spat out the bug, still hungry"]}),
("metaknight","epic",["Masked","Swordsman","Halberd","Warp-star","Knight","Mysterious"],r'''
  .---.
 /|◉ ◉|\
  \_▽_/
  /|||\
   / \
''',{"idle":["cape wrapped, mask gleaming","watching from the shadows of the terminal"],"working":["sword drawn, silent and fast","warp star over the blocker"],"celebrating":["a worthy victory, acknowledged","the halberd flies at dawn"],"miffed":["an unworthy performance","the mask hides everything, including this"]}),
("kingdedede","rare",["Dedede","Penguin","Hammer","King","Royal-blue","Waddle-boss"],r'''
  .-----.
 (o     o)
  \ ᴗ /
 /|###|\
  /___\
''',{"idle":["hammer over the shoulder, throne warm","waddle dees filing paperwork"],"working":["hammer time, kingly edition","supervising, mostly eating"],"celebrating":["I'm the king, and the build is MINE","a waddle-dance of victory"],"miffed":["somebody get me a NEW kingdom","the hammer came down on the wrong key"]}),
("pikmin","common",["Sprout","Leaf","Bud","Flower","Squad","Little-red"],r'''
   *
  (o o)
   \_/
  /| |\
   / \
''',{"idle":["waiting by the onion, planted","a small squad, huge courage"],"working":["carrying the payload, all 100 of us","whistling the work song"],"celebrating":["the onion sings! harvest time!","carried the day, literally"],"miffed":["bulborbs in the codebase again","one of us got eaten, probably me"]}),
("olimar","uncommon",["Captain","Explorer","Suited","Tiny-captain","Radar","Hocotatian"],r'''
  .---.
 (o   o)
  \ ᴗ /
  /|O|\
   / \
''',{"idle":["helmet on, suit holding","charting a course through the bugs"],"working":["day 12: the log parser works","discovery, carefully logged"],"celebrating":["the ship lifts off! treasure aboard!","day complete, profits logged"],"miffed":["day 30: rations low, patience lower","the pikmin are unionizing"]}),
("captainfalcon","rare",["Falcon","Racer","F-Zero","Bounty-hunter","Blue-falcon","Showtime"],r'''
  .-V-.
 |o   o|
 |  ▽  |
 /| F |\
  /   \
''',{"idle":["revving the blue falcon","showtime posture, always"],"working":["FALCON PAWNCH on the bug","racing the deadline, winning"],"celebrating":["FALCON... VICTORY!","a knee of pure celebration"],"miffed":["FALCON... DISAPPOINTED","the blue falcon took damage, unacceptable"]}),
("ness","uncommon",["Bat","PSI","Onett","Yo-yo","Kid","Cap"],r'''
  .---.
 |o   o|
 |  ᴗ  |
 /|===|\
  /   \
''',{"idle":["homing in on the next task","baseball bat leaned on the chair"],"working":["PSI rockin' on the stack trace","checking it out, in-game"],"celebrating":["you feel stronger, level up!","the Franklin badge deflected that bug"],"miffed":["you feel strange, status effect","giygas-tier frustration building"]}),
("lucas","uncommon",["Rope-snake","PSI-love","Tazmily","Kind","Twin","Quiet-heart"],r'''
  .---.
 |o   o|
 |  ω  |
 /|+++|\
  /   \
''',{"idle":["quiet, hopeful, brave","the rope snake naps nearby"],"working":["PK love on the blockers","fighting the masked man of dead code"],"celebrating":["the heart of the build is STRONG","everyone smiles, even the tests"],"miffed":["the dust grew cold for a moment","PK freeze... on my own enthusiasm"]}),
("ryu","rare",["Fighter","Wanderer","Ansatsuken","Gi","Hadou","Master"],r'''
  /^^\
 |o  o|
 |  ^ |
 /|道|\
  /   \
''',{"idle":["meditating on the next patch","the gi is pressed, the mind is clear"],"working":["shoryuken through the stack trace","training arc: dependencies"],"celebrating":["the answer lies in the build","a hadouken of pure focus"],"miffed":["the fight must continue","a storm of fists, briefly"]}),
("ken","rare",["Flaming","Rival","Shoryu","Blonde","Training-partner","Hot"],r'''
  /^^\
 |o  o|
 | ᴗ  |
 /|~|\
  /   \
''',{"idle":["hair perfect, fists ready","sparring with the compiler"],"working":["shinryuken on the blocker","fire up the build, literally"],"celebrating":["the flames rose HIGHER","an uppercut of triumph"],"miffed":["that was NOT a fair fight","hair slightly less perfect now"]}),
("sonic","epic",["Blue-blur","Speed","Rings","Hedgehog","Chili-dog","Fast"],r'''
  /^^^^
 /o  o >
 \  ▽ /
 /|===|\
  /   \
''',{"idle":["tapping a foot at supersonic speed","rings collected, loosely held"],"working":["gotta go fast on the hot path","loop-de-loop through the tests"],"celebrating":["WAY PAST COOL","a spin dash of victory"],"miffed":["eggman-tier setback","slowed to regular speed. tragic."]}),
("megaman","epic",["Blue-bomber","Robot-master","Mega-buster","Rock","Copy-weapon","Stage-clear"],r'''
  .---.
 |[o o]|
 |  _  |
 /|===|\
  /   \
''',{"idle":["buster charged, standing guard","scanning for robot masters"],"working":["copying the enemy's weapon","buster shot at the blocker"],"celebrating":["weapon get! BUILD BARRIER!","the fortress fell, stage clear"],"miffed":["spikes. why is it always spikes.","the weakness was discovered, painfully"]}),
("pacman","common",["Yellow","Maze-runner","Dot-eater","Chomper","Arcade","Waka"],r'''
  .---.
 /     \
|  ◕    )
 \     /
  \___/
''',{"idle":["circling the maze, hungry","power pellet saved for later"],"working":["wakka wakka through the tickets","eating dots, avoiding ghosts"],"celebrating":["high score, initials entered","the maze is CLEARED"],"miffed":["caught by blinky, near the exit","the ghosts are winning today"]}),
("cloudstrife","epic",["Ex-soldier","Buster-sword","Materia","Merc","Spiky","Planet-guardian"],r'''
  /\/\/\
 |o   o|
 |  ^  |
 /|====|\
  /   \
''',{"idle":["buster sword leaning on the wall","materia sorted by element"],"working":["limit break: limit broke","mako-lit progress, sustainable-ish"],"celebrating":["the buster sword gleams in victory","a win worth a gold saucer chobie"],"miffed":["the lifestream is disturbed","in my head: a one-winged angel"]}),
("sephiroth","legendary",["One-winged Angel","Silver","Masamune","Calamity","Nihil","Long-sword"],r'''
   /\
  /  \
 | ◉ ◉ |
  \ ▽ /
 /|||| |\
''',{"idle":["descending, unhurried","the masamune reflects one long blade of light"],"working":["cutting the blocker from seven extensions away","a meteor in the changelog"],"celebrating":["the reunion is COMPLETE","one wing raised, silent applause"],"miffed":["estuans interius, in A MINOR","the planet itself disapproves"]}),
("joker","rare",["Phantom","Trickster","Wild-card","Mask","Rebel","Curved-blade"],r'''
  .---.
 /o   o\
 |  ^  |
 /|♠♠♠|\
  /   \
''',{"idle":["mask tilted, cards shuffled","all-out attack pending"],"working":["infiltrating the codebase's palace","stealing the corrupt dependency's treasure"],"celebrating":["the phantom thieves strike again","take your heart (and the green build)"],"miffed":["the security level is MAXED","the velvet room offers no comfort"]}),
]

DATA["pokemon"].append(("petrock","common",["Pebble","Rocky","Dwayne","Boulder","Gravel","Chip","The Rock","Sedimentary"],r'''
  ____
 /    \
 \____/
''',{"idle":["does nothing","sits there","has seen things","is a rock"],"working":["still a rock","contains gravel","does not help","rocks do not work"],"celebrating":["vibrates with joy inside","emotionally granite"],"miffed":["unbothered","stays a rock"]}))
DATA["nostalgia"] = [
("furby","legendary",["Furball","Chatter","Fuzzbutt","Owl-Cat","Gremlin","Snuggler","Gizmo","Babbler"],r'''
   /\      /\
  /  \____/  \
 /            \
|   .------.   |
|  ( o    o )  |
|      ><      |
 \____________/
''',{"idle":["doo-doo-doo-doo","blinking at you in a language with no words","deep sleep is for the weak"],"working":["learning your language one typo at a time","furby needs... sleep... sometime..."],"celebrating":["doo? doo doo! DOO DOO DOO!","wheee! rolling right off the shelf"],"miffed":["eyes went dark. that is a threat","no no no no sleep again"]}),
("tamagotchi","epic",["Pixel Pal","Beep","Hatchling","Needs Food","Digi-Egg","Beeper","Tiny Pet","Care Button"],r'''
    .----.
   / .--. \
  | |o  o| |
  | | ᴗ  | |
  | '----' |
  |  o O o |
   \      /
    \____/
''',{"idle":["needs feeding, constantly","beeps at nothing","the cleanup icon is flashing"],"working":["pooping pixels while you work","will not survive being ignored for 4 hours"],"celebrating":["evolved into the good form today","happy beep beep beep"],"miffed":["the screen is full of skulls","ignored again. the skulls remain"]}),
("gameboy","rare",["Brick","Green Screen","No Backlight","Tetris Box","Four AA","Dot Matrix","Cart Eater","Button Masher"],r'''
 .----------.
 | .------. |
 | | #### | |
 | | #### | |
 | '------' |
 |    _     |
 |   |_|    |
 | (A)  (B) |
 '----------'
''',{"idle":["awaiting four double-As","the start button does the thing"],"working":["tetris on the side screen, priorities intact","processing at a blistering 4 MHz"],"celebrating":["new high score, initials AAA","the d-pad survived a thousand rounds"],"miffed":["the screen has no backlight. never did","batteries died at the boss fight"]}),
("vhstape","uncommon",["Be Kind Rewind","Tracking","4:3","Full Screen","Rewinder","Tape Hiss","Two Hours","EP Mode"],r'''
 .--------------.
 |  .-.    .-.  |
 | ( o )  ( o ) |
 | '---[##]---' |
 | BE KIND   SP |
 '--------------'
''',{"idle":["be kind, rewind","tracking... tracking..."],"working":["eats three hours of your life","the tracking knob turns, the lines obey"],"celebrating":["rewound before returning, a saint","tracked perfectly on the first try"],"miffed":["the tape is chewed. again","tracking off, rolling bars everywhere"]}),
("trolldoll","uncommon",["Gem Navel","Neon Hair","Luck Troll","Pencil Topper","Wild Mane","Wish Stone","Fad Survivor","Spiky"],r'''
    \|/\|/
    \ \| /
    ( o o )
     \ ᴗ /
     |   |
     /   \
''',{"idle":["hair standing straight up, as designed","rubbing the belly gem for luck"],"working":["the hair works even harder","pointy opinions, colorful"],"celebrating":["hair at maximum volume","wishing everyone a good time"],"miffed":["hair went flat. deeply offended","the gem fell off. bad omen"]}),
("gak","rare",["Gak","Green Goo","Double Dare","Doused","Bucket Dump","So Gross","Gritty","Weekend Slime","Irrevocable"],r'''
        _
     .-' '-.
    /  o o  \
   |    ᴗ    |
  \ \_______/ /
   '-'-----'-'
''',{"idle":["puddling. professionally","slowly claiming the floor"],"working":["seeping into everything you own","irreversibly marked, as designed"],"celebrating":["anyone hit gets slimed. that is the show","a full bucket, from above"],"miffed":["dried out at the edges","someone tried to mop. the mop lost"]}),
("lavalamp","rare",["Melted Wax","Dorm Glow","Blobs","Slow Motion","Heat Bulb","Groovy Light","Never Touch","Wax Tower"],r'''
      _
     | |
    /    \
   /  O   \
  |    o   |
  |   O    |
   \   o  /
    \    /
     \__/
    /____\
''',{"idle":["blob rises. blob falls","the yellow one is lazy today"],"working":["wax stuck at the top again","running warm, thinking slow"],"celebrating":["every blob is floating at once","maximum lava flow achieved"],"miffed":["the wax clumped. it is cloudy","shaken once. ruined forever. known fact"]}),
("spaceinvader","legendary",["Invader","8-Bit Menace","Column Bug","Marcher","Pixel Squid","Wave Three","Three Points","Descending"],r'''
  #     #
   #   #
  #######
 ## ### ##
###########
# ####### #
# #     # #
   ## ##
''',{"idle":["descending in neat formation","marching left, then right, then lower"],"working":["firing in synchronized columns","the formation closes in"],"celebrating":["the wave cleared without a shot","perfect grid discipline"],"miffed":["a shot from below. unacceptable","the fleet lost a pixel"]}),
("walkman","uncommon",["Mixtape Deck","Auto-Reverse","Bass Boost","AA Eater","Belt Clip","Skip Tape","Splitter","Wind It Up"],r'''
 .--------.
 | .----. |
 | |o  o| |
 | '----' |
 | [][=][] |
 '--------'
''',{"idle":["waiting for side B","a pencil stands by for rewinding"],"working":["auto-reverse flips the tape, again","playing the mix at full bass"],"celebrating":["the good track came on, and it is a tape","belt clip holding strong"],"miffed":["the tape ate itself again","batteries died mid-chorus"]}),
("dialup","rare",["56k","Screech","Handshake","Phone Squatter","Booooong","Disconnected Mom","Dial Tone","Line Busy"],r'''
 .---------.
 | o o o o |
 '---------'
      |
    _/\_
   /    \
''',{"idle":["screeee-krrrr-beee-doo","holding the only phone line hostage"],"working":["56k. everything loads, eventually","handshake in progress, do not lift"],"celebrating":["connected at a full 56k!","four megabytes of pure internet"],"miffed":["someone picked up the phone. again","carrier lost. the download is dead"]}),
("pog","common",["Slammer","Milk Cap","8-Ball Pog","Champion Disc","Alf","Pog Stack","Winner Keeps","Binder Page"],r'''
   .------.
  /  .--.  \
 |  ( () )  |
  \  '--'  /
   '------'
''',{"idle":["stacked face down, waiting","flat on the binder page"],"working":["slamming practice, form perfect","holds its ground, being round"],"celebrating":["won the whole stack!","slammed clean through the pile"],"miffed":["lost the slammer round","the collection got confiscated"]}),
("beaniebaby","common",["Bean Filled","First Edition","Tag Protector","Princess","Retired","PVC Pellets","Collector Item","Tag Never Removed"],r'''
   /\    /\
  ( o    o )
   \  --  /
  .-'    '-.
 (    ♡     )
  '-.,,,.-'
''',{"idle":["sitting on the shelf, appreciating","the tag stays on. always"],"working":["guarding its own retirement value","being worth approximately nothing"],"celebrating":["worth four hundred dollars again","first edition, mint, lovelight shining"],"miffed":["someone touched the tag","the market crashed in '99"]}),
("nokia","common",["Brick Phone","Snake","Indestructible","T9","Polyphonic","Week Battery","Snake II","Survivor"],r'''
  .-----.
  | ### |
  '-----'
  | (*) |
  | 1 2 |
  | 3 4 |
  '-----'
''',{"idle":["battery at four bars, as always","waiting for the ringtone"],"working":["playing snake under the desk","predicting text with terrifying accuracy"],"celebrating":["survived another drop test, unmarked","new high score on snake, 999"],"miffed":["bent in half. still works","the ringtone is marimba. someone's fault"]}),
("discman","common",["Anti-Skip","CD Walkman","Skip Protection","Discman","Lid Bounce","45 Seconds","Burned Mix","Laser Eye"],r'''
    .------.
   /  ,--.  \
  |  ( () )  |
  |  '----'  |
   \        /
    '------'
''',{"idle":["lid closed, spindle centered","45 seconds of skip protection, armed"],"working":["anti-skip eating the bumps","reading the burned mix flawlessly"],"celebrating":["zero skips on the nature hike","the whole album, front to back"],"miffed":["skipped during the good part","batteries died at track nine"]}),
("elmo","epic",["Tickle Me","Giggle Fits","Red Fuzz","Sold Out","Elmo Knows","Hehehe","Squishy","96 Holiday Riot"],r'''
   .----.
  / 0  0 \
 |  (__)  |
  \ .---. /
   '-...-'
''',{"idle":["hehehe hehehe","waiting to be tickled"],"working":["laughing so hard it falls over","the giggle motor runs hot"],"celebrating":["tickle! tickle tickle tickle!","sold out everywhere, a legend"],"miffed":["the giggle box jammed","tickle threshold not met. suspicious"]}),
("gigapet","common",["Keychain Egg","Giga Pet","Pocket Pal","Beeper Pet","Virtual Pet","Clip-On","Hungry Again","Tiny Screen"],r'''
    __o__
   /     \
  |  .-.  |
  |  |o|  |
  |  '-'  |
   \     /
    '---'
''',{"idle":["clipped to the backpack, beeping","hungry again, obviously"],"working":["tiny screen, big feelings","napping between keystrokes"],"celebrating":["grew one whole pixel today","fed, clean, and thrilled"],"miffed":["the beep went unheeded","sick. the medicine icon blinks"]}),
("yoyo","common",["Sleeper","Walk the Dog","Round the World","String Trick","Clutch","Rock the Baby","Looping","Winding Key"],r'''
     \
      \
    .----.
   / .--. \
  | ( () ) |
   \ '--' /
    '----'
''',{"idle":["sleeping at the end of the string","looped slack, patient"],"working":["walking the dog across the floor","rocking the baby, string geometry perfect"],"celebrating":["round the world, first try","slept for a full ten seconds"],"miffed":["the string tangled. again","response system broken. dead yoyo"]}),
("pollypocket","uncommon",["Tiny World","Compact House","Micro Doll","Hinged World","Pocket Sized","Choking Hazard","Blue Shell","Lost Tiny Comb"],r'''
  .--.__.--.
 /   |  |   \
| () |  | () |
 \   |__|   /
  '--....--'
''',{"idle":["open on the floor, doll inside","the tiny comb is missing again"],"working":["running the whole tiny household","hinged world, fully staffed"],"celebrating":["found the tiny comb!","the blue shell closes with a click"],"miffed":["stepped on. it is very small","the doll went under the couch"]}),
("viewmaster","uncommon",["Reel Viewer","3D Slides","Grand Canyon","Lever Click","Stereoscope","Reel Seven","Click Click","Squint Eye"],r'''
   _______
  /_______\
 | (o) (o) |
 |  .---.  |
  \ '---' /
   '-[ ]-'
''',{"idle":["pointed at the window, fishing for light","reel seven, the dinosaur one"],"working":["lever clicks, next slide","holding it to the sun properly"],"celebrating":["the whole reel, both eyes","clicked through all seven frames"],"miffed":["held it backwards. nothing but black","the reel jammed on frame four"]}),
("boombox","epic",["Shoulder Crate","Eight D Cells","Bass Cannon","Ghetto Blaster","Radio Raheem","Twin Woofers","Antenna Up","Loud"],r'''
  _____________
 |  ___   ___  |
 | |o| ## |o| |
 | |o| ## |o| |
 |___[]___[]__|
''',{"idle":["on the shoulder, at full volume","batteries: eight, all fresh"],"working":["the tape deck does the heavy lifting","bass at a volume the neighbors feel"],"celebrating":["both woofers at maximum","the antenna found the good station"],"miffed":["one speaker crackles","the antenna snapped off in the door"]}),
]
