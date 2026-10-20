# Holo Tactics: rules

Holo Tactics is a proposed Star Wars auto-battler mini-game for SJK, in the
style of Teamfight Tactics. It is **planned, not implemented**: it exists only
as standalone HTML mockups, and these rules describe what those mockups do.

- [holo-tactics-3d.html](holo-tactics-3d.html): 3D arena, English
- [holo-taktika-3d.html](holo-taktika-3d.html): 3D arena, Slovak
- [holo-taktika.html](holo-taktika.html): flat 2D board, Slovak

The numbers come from the mockup code and were balanced only by its own
simulation; treat them as a starting point.

## Goal

You play against 7 opponents. Each one is a "ghost": another player's saved
lineup from the SJK hub. Losing a round costs health, and at 0 your run is over.
Your score is the number of rounds you survived. When the run ends, your final
lineup goes to the hub as a ghost for other players to fight.

The mockup starts mid-run at stage 3-2 with 74 health, so it does not settle the
starting values. The proposal is 100 health, as in TFT.

## Rounds

- Each **stage** has 6 rounds. Rounds 1–5 are **PvP** against a ghost. Round 6
  is **PvE** against cave creatures (mynocks, a wampa, and later rancors), and
  winning it gives you an item.
- Each round has a **Planning** phase and a **Combat** phase. Planning has a
  30-second timer, and combat starts automatically when it runs out. **Fight**
  (Space) starts it early.

## Board and bench

- The arena is a hex grid with 4 rows on each side. You place units only on your
  half.
- The number of units you can have on the board equals your **level**. The
  **bench** holds 9 more units, which do not fight.
- To move a unit, drag it to a hex or a bench slot, or click the unit and then
  click where it should go.

## Gold and income

At the start of each round you get:

- **5 gold** base income.
- **Interest:** +1 gold for every 10 gold you hold, up to +5.
- A **streak** bonus for wins or losses in a row: +1 for a 2–3 streak, +2 for
  4–5, +3 for 6 or more.
- **+2 XP** automatically.

Winning a PvP round also gives **+1 gold**.

## Shop

- The shop shows 5 units. Each costs 1–5 gold, and the card color shows its
  rarity, matching the rarity colors of SJK cards.
- **Reroll** (D) refreshes the shop for 2 gold. **Lock shop** keeps the same
  offers into the next round.
- Each unit has a limited number of copies: 18 of each 1-cost unit, 15 of each
  2-cost, 13 of each 3-cost, 10 of each 4-cost and 8 of each 5-cost. Copies you
  hold and copies in your shop are taken out of the pool.
- Your level decides how likely each cost is to appear:

| Level | 1-cost | 2-cost | 3-cost | 4-cost | 5-cost |
| --- | --- | --- | --- | --- | --- |
| 1–2 | 100% | – | – | – | – |
| 3 | 75% | 25% | – | – | – |
| 4 | 55% | 30% | 15% | – | – |
| 5 | 45% | 33% | 20% | 2% | – |
| 6 | 30% | 40% | 25% | 5% | – |
| 7 | 20% | 30% | 33% | 15% | 2% |
| 8 | 15% | 20% | 35% | 25% | 5% |
| 9 | 10% | 15% | 30% | 30% | 15% |

## Levels

**Buy XP** (F) gives 4 XP for 4 gold. Going up a level takes 2, 2, 6, 10, 20,
36, 48 and then 72 XP. Level 9 is the maximum.

## Stars

- Three copies of the same unit at the same star level merge into one stronger
  unit: three 1★ make a 2★, and three 2★ make a 3★.
- Stats are multiplied by **×1.8** at 2★ and **×3.24** at 3★. Abilities are
  multiplied by **×1.5** and **×2.25**.
- The merged unit keeps the items from all three copies, up to 3. Extra items go
  back to your item tray.

## Selling

Drag a unit to the shop, or select it and press E. A 1★ unit sells for its cost.
A 2★ or 3★ unit sells for its total cost minus 1. Its items go back to your tray.

## Items

Items drop from PvE rounds. A unit can carry up to 3.

| Item | Effect |
| --- | --- |
| Kyber Crystal | +15% damage |
| Bacta | +150 HP |
| Cortosis Vest | +25 armor |
| Holocron | +15 Force at the start of combat |
| Blaster Module | +15% attack speed |

## Combat

- Units fight on their own. Each one targets the nearest enemy, walks toward it,
  and attacks once the enemy is in range. A taunting unit within 2 hexes is
  attacked first.
- **Force (mana):** a unit gains +10 Force when it attacks and +5 when it is
  hit. When its bar is full, it uses its ability.
- **Armor:** every unit has 20 armor (Guardians have 40). Basic attacks deal
  damage × 100 / (100 + armor). Abilities ignore armor.
- **Saber deflection:** shots from Gunner units can be deflected by any unit that
  has a saber. The chance is 20%, plus the Jedi trait bonus. A deflected shot
  hits the shooter for 70% of its damage, and the deflecting unit gains +5 Force.
- **Overtime:** after 30 seconds, all damage rises by 20% every second.
- **Time limit:** at 45 seconds the fight stops. You win only if your side wiped
  out the enemy; anything else counts as a loss.

## Losing a round

You lose **2 + the stage number + the stars of every enemy unit still alive** in
health. Summoned units do not count.

## Traits

A trait counts each different unit once, so two copies of the same unit count as
one. Reaching a breakpoint turns on the bonus.

| Trait | Breakpoints | Bonus |
| --- | --- | --- |
| Jedi | 2 / 4 | Saber users deflect 25 / 40% more shots. Jedi take 10 / 20% less damage. |
| Sith | 2 / 4 / 6 | Sith heal for 75 / 90 / 90% of the damage they deal. At 6, Force Lightning hits every enemy for 60 when combat starts. |
| Empire | 2 / 4 | 1 / 2 stormtroopers join at the start of combat. |
| Rebels | 2 / 4 | Each time a Rebel falls, the remaining allies gain +80 / 60% attack speed. |
| Mercenaries | 2 / 3 | Mercenaries deal +25 / 40% damage. A won round gives +1 / +2 gold. |
| Beasts | 2 | A Beast's first attack stuns the target and its neighbors for 2.8 s. |
| Duelist | 2 / 4 | Every 3rd attack deals 2.9 / 3× damage. |
| Guardian | 2 / 4 / 6 | +50 / 80 / 140 armor. |
| Gunner | 2 / 4 | +1 range and +25 / 38% damage. |
| Mystic | 2 / 4 | Abilities are 55 / 85% stronger. |
| Assassin | 2 / 3 | Leap behind the enemy line when combat starts. 35 / 60% chance to crit for 1.8× damage. |
| Commander | 2 / 3 | The whole team starts with +20 / 40 Force. |

## Units

| Cost | Unit | Traits | Ability |
| --- | --- | --- | --- |
| 1 | Stormtrooper | Empire, Gunner | Volley: 3 quick shots at 70% damage each. |
| 1 | Reborn | Sith, Duelist | Saber Toss: 180 to the target. |
| 1 | Rebel Trooper | Rebels, Gunner | Thermal Detonator: 150 to the target and its neighbors. |
| 1 | Weequay | Mercenaries, Guardian | Shield: a 250 shield for 4 s. |
| 1 | Padawan | Jedi, Duelist | Force Push: 120 and a 1 s stun. |
| 2 | Jaden Korr | Jedi, Duelist | Lunge: leaps to the farthest enemy and strikes it for 220. |
| 2 | Rosh Penin | Jedi, Guardian | Taunt: a 300 shield, and neighbors attack only him. |
| 2 | Swamptrooper | Empire, Guardian | Flamethrower: 160 to all neighbors. |
| 2 | Rodian Sniper | Mercenaries, Gunner | Marked: 280 to the enemy with the lowest HP. |
| 2 | Cultist | Sith, Mystic | Force Drain: drains 200 HP from the target and heals itself. |
| 2 | Imperial Officer | Empire, Commander | Orders: allies within 2 hexes get +50% attack speed for 4 s. |
| 3 | Alora | Sith, Assassin | Saber Throw: 320 to the farthest enemy. |
| 3 | Shadowtrooper | Empire, Assassin | Cloak: the next attack deals 2.5× damage. |
| 3 | Chewbacca | Rebels, Guardian | Roar: stuns all neighbors for 1.5 s. |
| 3 | Wampa | Beasts, Guardian | Hurl: grabs the target for 260 and stuns it. |
| 3 | Lando Calrissian | Rebels, Gunner, Commander | Twin Blasters: 5 shots at 70% damage each. |
| 4 | Kyle Katarn | Jedi, Duelist, Commander | Tornado: 300 to all neighbors. |
| 4 | Tavion | Sith, Mystic | Force Lightning: 300 to the target, then jumps to 3 more. |
| 4 | Rancor | Beasts, Guardian | Stomp: 250 to neighbors, and eats a neighbor below 25% HP. |
| 4 | Desann | Sith, Guardian | Force Grip: lifts the target for 2 s and chokes it for 350. |
| 4 | Mon Mothma | Rebels, Commander | Bacta: heals the whole team for 350. |
| 5 | Luke Skywalker | Jedi, Mystic | Protect and Push: a 300 shield for the whole team; pushes back and stuns neighbors. |
| 5 | Marka Ragnos | Sith, Mystic | Sith Spirits: summons 2 Sith spirits. |
| 5 | Boba Fett | Mercenaries, Gunner, Assassin | Jetpack: flies over the back row and hits an area for 450 with a rocket. |

## Cosmetics

A Holo or Solar glow on a unit comes from your SJK cards and is cosmetic only. It
never changes stats.

## Controls

| Key | Action |
| --- | --- |
| Space | Fight |
| D | Reroll the shop |
| F | Buy XP |
| E | Sell the selected unit |
| Esc | Clear the selection |
