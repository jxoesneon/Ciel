//! ciel-studio — unified CLI for the autonomous-3d-studio domain skill.
//! Each subcommand maps 1:1 to a Python script in
//! `skills/autonomous-3d-studio/scripts/`.

use ciel_domain::studio;

const USAGE: &str = "usage: ciel-studio <subcommand> [options]

  blender-mcp       blender_mcp_server.py       live Blender socket RPC bridge
  pipeline-exec     blender_pipeline_executor.py headless blender -b -P runner
  unreal-bridge     unreal_engine_bridge.py     UE5 import script generator
  bake              high_to_low_baker.py        cage mesh + bake recipe
  collision         collision_hull_generator.py UBX/UCX hull generation
  geometry-qa       geometry_qa_validator.py    topological QA audit + --fix
  facs-mirror       facs_blendshape_mirror.py   ARKit/FACS symmetry mirror
  generative        generative_3d_adapter.py    AI mesh DSU post-processor
  kit               procedural_kit_generator.py modular wall panel generator
  retopo            retopology_quadriflow.py    cascading LOD decimator
  sat-bake          substance_sat_baker.py      SAT sbsbaker recipe
  turnaround        turnaround_qa_renderer.py   visual QA HTML sheet
  udim              udim_pack_analyzer.py       UDIM layout audit
  usd-bridge        usd_materialx_bridge.py     USDA + MaterialX emit
  usd-variant       usd_variant_manager.py      USD variant stage emit
  uv-texel          uv_texel_analyzer.py        texel density audit
  vnt               vertex_normal_transfer.py   CAD normal projection
  distill           distill_3d_instincts.py     instinct consolidation
  refine            autonomous_refinement_loop.py closed-loop refinement
";

fn main() {
    // Match CPython SIGPIPE handling: die quietly on closed pipes (| head).
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let code = match argv.first().map(|s| s.as_str()) {
        Some("blender-mcp") => studio::blender_mcp::run(&argv[1..]),
        Some("pipeline-exec") => studio::pipeline::run(&argv[1..]),
        Some("unreal-bridge") => studio::unreal::run(&argv[1..]),
        Some("bake") => studio::baker::run(&argv[1..]),
        Some("collision") => studio::collision::run(&argv[1..]),
        Some("geometry-qa") => studio::qa::run(&argv[1..]),
        Some("facs-mirror") => studio::facs::run(&argv[1..]),
        Some("generative") => studio::generative::run(&argv[1..]),
        Some("kit") => studio::kit::run(&argv[1..]),
        Some("retopo") => studio::retopo::run(&argv[1..]),
        Some("sat-bake") => studio::sat::run(&argv[1..]),
        Some("turnaround") => studio::turnaround::run(&argv[1..]),
        Some("udim") => studio::udim::run(&argv[1..]),
        Some("usd-bridge") => studio::usd_materialx::run(&argv[1..]),
        Some("usd-variant") => studio::usd_variant::run(&argv[1..]),
        Some("uv-texel") => studio::uv_texel::run(&argv[1..]),
        Some("vnt") => studio::vnt::run(&argv[1..]),
        Some("distill") => studio::distill::run(&argv[1..]),
        Some("refine") => studio::refine::run(&argv[1..]),
        Some("-h") | Some("--help") | None => {
            eprint!("{}", USAGE);
            if argv.is_empty() {
                2
            } else {
                0
            }
        }
        Some(other) => {
            eprintln!("ciel-studio: unknown subcommand '{}'", other);
            eprint!("{}", USAGE);
            2
        }
    };
    std::process::exit(code);
}
