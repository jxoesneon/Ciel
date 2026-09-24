# COUNCIL DOCKET: 20260819_ASSET_ACQUISITION

**Date**: 2026-08-19
**Candidate Artifact**: Godot Asset Library Acquisition Candidates for BioGenesis-X
**Scope**: Architecture Council (SKILL_INTEGRATION — external asset acquisition)

## Project Context

BioGenesis-X is an AAA 3D biopunk starship builder and void-flight combat simulator built with Godot 4.7.1, Forward+ renderer, JoltPhysics3D. Key systems already implemented:
- Procedural biological starships, planets, asteroids, star systems
- 6-DOF Newtonian flight with mouse tethered controls
- Wave Engine (Alcubierre-like) for in-system transit
- Combat: 6 weapon types, 4 enemy classes, AI state machine, shields, VFX, hit markers, combat stats
- Procedural audio synthesis (BioAudioSynth + BioAudioDirector autoloads, 22050 Hz)
- Planetary landing, on-foot exploration, swimming, EVA
- Realistic astronomical scale with galaxy map
- Comprehensive settings system (50+ controls)
- ChunkStreamManager for gameplay streaming

## Consolidated Asset Candidates (71 unique assets, deduplicated across 3 swarm agents)

### TIER 1 — HIGH RELEVANCE (Directly addresses known gaps)

| # | Asset | ID | Author | License | Godot Ver | Category | Relevance |
|---|-------|-----|---------|---------|-----------|----------|-----------|
| 1 | Terrain3D | 3134 | TokisanGames | MIT | 4.3-4.6+ | 3D Tools | GPU clipmap terrain, 65km², LOD, foliage — planetary surfaces |
| 2 | Extremely Fast Atmosphere | 4218 | fbcosentino | unspecified | 4.4 | Shaders | Non-raymarching atmosphere, Forward+ compatible, StandardMaterial3D perf |
| 3 | 3D Planet Generator | 1615 | naejimer | unspecified | 4.0-4.5 | Shaders | Planet body + clouds + atmosphere shaders, 7 planet types |
| 4 | Starlight | 2221 | tiffany352 | unspecified | 4.3 | Shaders | 100K positional stars, PSF-based, MultiMeshInstance3D |
| 5 | LimboAI | 4852 | limbonaut | MIT | 4.6 | Tools | Behavior trees + state machines, visual editor, GDScript |
| 6 | Interactive Energy Shield | 3628 | nojoule | unspecified | 4.5 | Shaders | Shield with impact waves, intersection highlighting |
| 7 | Boujie Water Shader | 2070 | Chrisknyfe | unspecified | 4.1+ | Shaders | Gerstner waves, foam, refraction, infinite ocean, LOD |
| 8 | Procedural Planet (Chunked LOD) | 4942 | cuberact | unspecified | 4.6+ | Demo | Quadtree LOD, cube-sphere projection, atmospheric scattering, origin shifting |
| 9 | Godot Synth | 3839 | EclipsingLines | unspecified | 4.4 | Misc | Virtual analog synthesis, polyphonic, preset system |
| 10 | NeuralRegen | 4527 | unspecified | unspecified | 4.2 | Shaders | Neural Cellular Automata, self-healing biological materials, GPU compute |
| 11 | Procedural Symbiote/Fresnel | N/A | mzrlee | CC BY-NC-SA 3.0 | 4.x | Shaders | Alien symbiote organic surfaces, animated |
| 12 | Juicee | 5218 | kelpekk | MIT | 4.2-4.3+ | Tools | 99 game-feel effects: screen shake, hit-stop, damage numbers, springs |
| 13 | Dialogue Manager 3 | 3654 | Nathan Hoad | unspecified | 4.4+ | Tools | Branching dialogue, localization, gettext, CSV |
| 14 | Nexus Quest Weaver | 4548 | movec | MIT | 4.6+ | Tools | Visual quest editor, JSON saves, localization, inventory adapter |
| 15 | Debug API | 5131 | unspecified | unspecified | 4.0+ | Tools | 50+ monitors, <1ms/frame, 8 presets, 7 themes, export |
| 16 | AdaptiSound | 1983 | Mr. Walkman | MIT | 4.1+ | Tools | Adaptive music manager, dynamic intensity transitions |
| 17 | AMP (Adaptive Music Player) | 1998 | unspecified | unspecified | 4.0+ | Tools | Stem-based adaptive music, runtime stem add/remove |
| 18 | GD Audio Analyzer | 4432 | unspecified | unspecified | 4.5+ | Tools | Real-time FFT, beat detection, audio intensity analysis |
| 19 | Godot Shaders Library | 4890 | kelpekk | MIT | 4.1 | Tools | Browse 2000+ shaders from godotshaders.com in-editor |
| 20 | Procedural Saver/Loader | 439 | unspecified | unspecified | N/A | Scripts | Saves procedural scene trees, used in I,Voyager (100+ planets, 65K asteroids) |
| 21 | Procedural Texture Designer | 3335 | wakeofluna | unspecified | 4.3 | Tools | Visual procedural texture editor, export as shaders/images |
| 22 | Gaea | 3272 | gaea-godot | open source | 4.3-4.4+ | Misc | Graph-based procgen, cellular/heightmap/walker generators |
| 23 | Spatial Gardener | 2037 | dreadpon | unspecified | 4.4 | 3D Tools | Paint foliage on arbitrary 3D surfaces, thousands of instances |
| 24 | Procedural Plant Generator | N/A | vilem-janota | unspecified | 4.x | Plugin | L-System plants, 2D/3D, growth animation, wind, bark/leaf materials |
| 25 | Sunshine Clouds System | 2372 | Bonkahe | unspecified | 4.3 | 3D Tools | Flyable volumetric clouds, ray marched, procedural or paintable |
| 26 | TCA Weather System | N/A | kS222138 | MIT | 4.6+ | Plugin | Volumetric clouds, dynamic sky, water shader, wind, seasons, precipitation |
| 27 | Godot Projectile Engine | N/A | AzyrGames | unspecified | 4.x | Plugin | Thousands of projectiles, PatternComposer, TimingScheduler |
| 28 | All Projectiles | 3924 | Oscarvezz | unspecified | 4.4 | Tools | Compact projectile engine, hands-off instancing/disposal |
| 29 | LocGuard Lite | 5378 | blobsmith | MIT | 4.7+ | Tools | Finds untranslated strings, scans tr()/atr(), .tscn/.tres |
| 30 | Controller Icons | 2565 | rsubtil | MIT | 4.1.2+ | 2D Tools | Controller/keyboard icons, automatic remapping |
| 31 | InputController | 2973 | unspecified | unspecified | 4.2+ | Tools | Tap/double-tap/press/long-press/hold detection |
| 32 | I, Voyager Core | N/A | ivoyager | open source | 4.2+ | Plugin | Real-scale solar system, orbital mechanics, 70K asteroids |
| 33 | Celestial Bodies | 3683 | unspecified | unspecified | 4.3 | Scripts | Sebastian Lague solar system, procedural planets, orbital sim |
| 34 | Simplified Flight Simulation | N/A | unspecified | unspecified | 4.1+ | Addon | Planes/helis/drones/spaceships, spherical world physics |
| 35 | SaveState | 4990 | unspecified | MIT | 4.3 | Tools | Atomic saves, .bak backups, schema migration, saveable groups |

### TIER 2 — MEDIUM RELEVANCE (Useful but not critical)

| # | Asset | ID | Author | License | Godot Ver | Category | Relevance |
|---|-------|-----|---------|---------|-----------|----------|-----------|
| 36 | FlexCam | 4590 | unspecified | unspecified | 4.5 | 3D Tools | FPV/follow/observer camera modes |
| 37 | 3D Controls Toolkit | 3297 | unspecified | unspecified | 4.3-4.5 | 3D Tools | FP/TP/side-scroll/top-down controllers |
| 38 | Quality First Person Controller v2 | 2418 | Colormatic | MIT | 4.7 | 3D Tools | FPS controller with headbob, swim, fly modes |
| 39 | Interaction Kit 3D | 3409 | ninetailsrabbit | MIT | 4.3 | 3D Tools | Pick up, throw, interact with 3D objects |
| 40 | YParticles3D | 5211 | unspecified | unspecified | 4.5 | 3D Tools | Shuriken-inspired CPU GDExtension particles |
| 41 | UniParticles3D | 3741 | unspecified | unspecified | 4.3 | 3D Tools | Unity-like particle system, RenderingServer multimesh |
| 42 | Full Screen Effects | 3534 | unspecified | unspecified | 4.3 | Scripts | Radial blur, FOV shake, motion blur, color fade |
| 43 | Universal Transition Shader | 4148 | cashew-olddew | unspecified | 4.4 | Shaders | Directional wipes, dissolves, iris reveals for scene transitions |
| 44 | Extra GUI Controls | 1922 | unspecified | MIT | 4.0 | Scripts | Radial menus, drag-drop containers, scroll-zoom |
| 45 | Modular Inventory | 5186 | unspecified | unspecified | 4.0 | 3D Tools | Data-driven inventory, equipment manager, hotbars, 3D integration |
| 46 | Godot MIDI Player | 1667 | arlez80 | unspecified | 4.3+ | Scripts | Pure GDScript MIDI player, soundfont support |
| 47 | FastNoiseLite Runtime Shader | 2497 | unspecified | unspecified | 4.2 | Shaders | GPU FastNoiseLite, 99.9% accuracy to C++ version |
| 48 | NoiseLib | 3851 | unspecified | unspecified | 4.4 | Shaders | PerlinNoise3D, VoronoiNoise3D, PixelNoise3D shader nodes |
| 49 | Voronoi Texture Scattering | 4217 | unspecified | unspecified | 4.2 | Shaders | Eliminates repeating patterns in tiled textures |
| 50 | FloatableBody | 2345 | unspecified | unspecified | N/A | Misc | Buoyancy physics for floating objects in water |
| 51 | Abandoned Spaceship Demo | 1733 | Godot Engine | unspecified | 4.0 | Demos | Reference for lighting, trim sheets, volumetric fog, TAA |
| 52 | ScriptBench | 2438 | unspecified | unspecified | 4.2 | Tools | GDScript benchmarking for method comparison |
| 53 | Radial Menu Control | 3469 | unspecified | unspecified | 4.3+ | 2D Tools | Pie menu with submenus, keyboard/mouse/gamepad |
| 54 | Spotlight Search | 4570 | unspecified | MIT | 4.3 | Tools | Global search + command palette for editor |
| 55 | AssetPlus | 4714 | MoongDevStudio | unspecified | 4.3 | Tools | Unified asset browser (AssetLib + Godot Store + Shaders) |
| 56 | SO FLUFFY Fur Rendering | N/A | maxvolumedev | unspecified | 4.x | Plugin | Shell fur with physics, LODs, turbulence, curling |
| 57 | Squiggles Fur | 2339 | unspecified | unspecified | 4.2 | Materials | Shell fur tool, no-code approach |
| 58 | Glowing Border Effect | 1759 | unspecified | unspecified | 4.3 | Shaders | Glow outline for selected objects, per-object color |
| 59 | Depth Fog Screen Space PostFX | 2443 | unspecified | unspecified | 4.2 | Shaders | Depth-based fog with noise |
| 60 | Volumetric Clouds Demo v2 | N/A | clayjohn | unspecified | 4.2+ | Demo | Compute shader clouds, time-of-day integration |

### TIER 3 — LOW RELEVANCE (Nice-to-have or redundant)

| # | Asset | ID | Relevance |
|---|-------|-----|-----------|
| 61 | GDT Terrain Generator | 5160 | Redundant with Terrain3D |
| 62 | TerrainCrafter | 3173 | Redundant with Terrain3D |
| 63 | PGodot | 4144 | Redundant with Gaea + existing noise systems |
| 64 | CSG Toolkit | 3057 | Prototyping only |
| 65 | Godot AI Kit | 1477 | ML algorithms — overkill for game AI |
| 66 | OpenAI API Godot | 3185 | External API dependency, Safety risk |
| 67 | PopcornFX | 4995 | Proprietary, external editor |
| 68 | Rapier Physics | 3084/3085 | Already have JoltPhysics |
| 69 | Planet2D | 1899 | 2D only |
| 70 | Various 2D inventory systems | multiple | 2D-focused, less relevant |
| 71 | Various dialogue alternatives | multiple | Dialogue Manager 3 is industry standard |

## Evaluation Criteria for Council Members

Each member should evaluate the 35 Tier 1 assets against their lens:

1. **Coherence**: Does the asset fit BioGenesis-X's architecture (autoload pattern, signal-based, @tool, Jolt, Forward+, procedural-first)?
2. **Capability**: Does it genuinely expand capability vs. what we already have? Is it redundant with existing systems?
3. **Safety**: License compatibility (MIT/Apache OK, CC BY-NC-SA problematic for commercial, unspecified = risk), untrusted code execution, external API dependencies, supply chain risk
4. **Efficiency**: Performance impact, bloat, GDExtension vs GDScript, memory footprint
5. **Evolution**: Does it support the growth trajectory (multiplayer, modding, seamless space-to-surface)?

## Key Architectural Constraints

- BioGenesis-X uses **procedural everything** — no external audio files, procedural textures preferred
- **Forward+ renderer** required (some assets may not be compatible)
- **JoltPhysics3D** already integrated — no need for alternative physics
- **Godot 4.7.1** — assets must be compatible or easily portable
- **Autoload pattern** for global systems (BioAudioSynth, BioAudioDirector, CombatVFX, CombatStats, etc.)
- **Signal-based communication** between systems
- **@tool mode** used extensively for editor previews
- **Real astronomical scale** — assets must support large coordinate spaces
- **Biopunk aesthetic** — organic, biological, self-healing themes
