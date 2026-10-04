//! ciel-audio — unified CLI for the procedural-audio domain skill.

use ciel_domain::audio;

fn usage() -> ! {
    eprintln!("ciel-audio — procedural audio domain toolkit");
    eprintln!();
    eprintln!("Subcommands:");
    eprintln!(
        "  synth            procedural_audio_generator.py  (--sfx/--format/--out/--sr/--duration)"
    );
    eprintln!("  aaa              aaa_audio_generator.py         (--sfx/--out/--depth/--duration)");
    eprintln!("  humanize         advanced_humanizer.py          (test-bench output)");
    eprintln!("  microtonal       microtonal_pitch_engine.py     (demo output)");
    eprintln!("  tala             indian_tala_engine.py          (grids + tihai demo)");
    eprintln!("  lsystem          lsystem_schenker_generator.py  (demo output)");
    eprintln!("  tonnetz          neo_riemannian_tonnetz.py      (transforms + paths)");
    eprintln!("  motif            motif_dna_generator.py         (demo + --out MIDI export)");
    eprintln!("  scene            scene_context_analyzer.py      (--file/--mood/--out)");
    eprintln!("  manifest         audio_manifest_extractor.py    (<input.wav>)");
    eprintln!("  audiogram        audiogram_generator.py         (<input.wav> [output.png])");
    eprintln!(
        "  cues             analyze_music_cues.py          (input --output-json --output-md)"
    );
    eprintln!("  extract          extract-audio-data.py          (input -o json --fps --bands)");
    eprintln!("  hook-auto-wire       auto_wire_hook.py          (<scene file>)");
    eprintln!("  hook-post-synthesize post_synthesize_hook.py    ([file.wav] | stdin)");
    eprintln!("  hook-pre-invocation  pre_invocation_hook.py     (stdin)");
    eprintln!("  hook-pre-synthesize  pre_synthesize_hook.py     (stdin)");
    eprintln!("  hook-stop            stop_check_hook.py         (stdin)");
    std::process::exit(2);
}

fn main() {
    // Match CPython SIGPIPE handling: die quietly on closed pipes (| head).
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let sub = match argv.first() {
        Some(s) if !s.starts_with('-') => s.as_str(),
        Some(s) if s == "-h" || s == "--help" => usage(),
        _ => usage(),
    };
    let rest = &argv[1..];
    let code = match sub {
        "synth" => audio::procgen::run(rest),
        "aaa" => audio::aaa::run(rest),
        "humanize" => audio::humanizer::run(rest),
        "microtonal" => audio::microtonal::run(rest),
        "tala" => audio::tala::run(rest),
        "lsystem" => audio::lsystem::run(rest),
        "tonnetz" => audio::tonnetz::run(rest),
        "motif" => audio::motif::run(rest),
        "scene" => audio::scene::run(rest),
        "manifest" => audio::manifest::run(rest),
        "audiogram" => audio::audiogram::run(rest),
        "cues" => audio::cues::run(rest),
        "extract" => audio::extract::run(rest),
        "hook-auto-wire" => audio::hooks::run_auto_wire(rest),
        "hook-post-synthesize" => audio::hooks::run_post_synthesize(rest),
        "hook-pre-invocation" => audio::hooks::run_pre_invocation(rest),
        "hook-pre-synthesize" => audio::hooks::run_pre_synthesize(rest),
        "hook-stop" => audio::hooks::run_stop_check(rest),
        other => {
            eprintln!("ciel-audio: unknown subcommand '{}'", other);
            usage();
        }
    };
    std::process::exit(code);
}
