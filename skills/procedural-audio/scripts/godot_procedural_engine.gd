# godot_procedural_engine.gd
# Universal Autonomous Procedural Audio Engine for Godot 4.x
# Implements 4-Layer Soundscape, Lock-Free Streaming, Modal Music, and Physics/UI SFX.
class_name GodotProceduralEngine
extends Node

@export var tempo_bpm: float = 120.0
@export var tension_index: float = 0.0 # DTI: 0.0 to 1.0
@export var master_volume_db: float = 0.0
@export var active_mode: String = "DORIAN"

const SAMPLE_RATE: float = 44100.0

# Audio Players & Playback Generators
var player_ambience: AudioStreamPlayer
var player_music: AudioStreamPlayer
var gen_ambience: AudioStreamGeneratorPlayback
var gen_music: AudioStreamGeneratorPlayback

# Ambience DSP State
var phase_drone_1: float = 0.0
var phase_drone_2: float = 0.0
var phase_lfo: float = 0.0

# Music Engine State
var scale_intervals: Array[int] = [0, 2, 3, 5, 7, 9, 10] # Dorian
var root_midi_note: int = 50 # D3
var current_chord_root: int = 50
var step_timer: float = 0.0
var current_step: int = 0
var voice_freq: float = 0.0
var voice_env: float = 0.0
var voice_phase: float = 0.0

# UI & Physics SFX Pool
var sfx_bus_name: String = "ProceduralSFX"

func _ready() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS
	_setup_audio_buses()
	_init_stream_players()
	_auto_wire_scene(get_tree().current_scene)
	print("[GodotProceduralEngine] Procedural soundscape initialized and wired.")

func _setup_audio_buses() -> void:
	var bus_idx = AudioServer.get_bus_index(sfx_bus_name)
	if bus_idx == -1:
		AudioServer.add_bus()
		bus_idx = AudioServer.bus_count - 1
		AudioServer.set_bus_name(bus_idx, sfx_bus_name)
		var reverb = AudioEffectReverb.new()
		reverb.room_size = 0.6
		reverb.damping = 0.35
		reverb.wet = 0.2
		AudioServer.add_bus_effect(bus_idx, reverb)

func _init_stream_players() -> void:
	# Ambient Generator
	var stream_amb = AudioStreamGenerator.new()
	stream_amb.mix_rate = SAMPLE_RATE
	stream_amb.buffer_length = 0.1
	player_ambience = AudioStreamPlayer.new()
	player_ambience.stream = stream_amb
	player_ambience.bus = "Master"
	add_child(player_ambience)
	player_ambience.play()
	gen_ambience = player_ambience.get_stream_playback()

	# Music Generator
	var stream_mus = AudioStreamGenerator.new()
	stream_mus.mix_rate = SAMPLE_RATE
	stream_mus.buffer_length = 0.1
	player_music = AudioStreamPlayer.new()
	player_music.stream = stream_mus
	player_music.bus = "Master"
	add_child(player_music)
	player_music.play()
	gen_music = player_music.get_stream_playback()

func _auto_wire_scene(node: Node) -> void:
	if node == null:
		return
	if node is BaseButton:
		node.pressed.connect(func(): play_ui_click(node))
		node.mouse_entered.connect(func(): play_ui_hover(node))
	elif node is RigidBody2D:
		node.contact_monitor = true
		node.max_contacts_reported = 4
		node.body_entered.connect(func(body): _on_collision_2d(node, body))
	elif node is RigidBody3D:
		node.contact_monitor = true
		node.max_contacts_reported = 4
		node.body_entered.connect(func(body): _on_collision_3d(node, body))

	for child in node.get_children():
		_auto_wire_scene(child)

func _process(delta: float) -> void:
	_render_ambience()
	_update_music(delta)
	_render_music()

func _render_ambience() -> void:
	if gen_ambience == null: return
	var frames = gen_ambience.get_frames_available()
	while frames > 0:
		phase_lfo += (2.0 * PI * 0.12) / SAMPLE_RATE
		if phase_lfo > 2.0 * PI: phase_lfo -= 2.0 * PI
		
		var lfo_val = sin(phase_lfo)
		var f1 = 55.0 + lfo_val * 1.5
		var f2 = 110.0 + (tension_index * 25.0)
		
		phase_drone_1 += (2.0 * PI * f1) / SAMPLE_RATE
		phase_drone_2 += (2.0 * PI * f2) / SAMPLE_RATE
		if phase_drone_1 > 2.0 * PI: phase_drone_1 -= 2.0 * PI
		if phase_drone_2 > 2.0 * PI: phase_drone_2 -= 2.0 * PI
		
		var drone = (sin(phase_drone_1) * 0.3) + (sin(phase_drone_2) * (0.05 + tension_index * 0.15))
		var noise = (randf() * 2.0 - 1.0) * 0.02
		var frame = Vector2.ONE * (drone + noise)
		gen_ambience.push_frame(frame)
		frames -= 1

func _update_music(delta: float) -> void:
	var step_dur = 60.0 / (tempo_bpm * 4.0)
	step_timer += delta
	if step_timer >= step_dur:
		step_timer -= step_dur
		current_step = (current_step + 1) % 16
		
		if current_step == 0:
			var roots = [50, 53, 48, 45]
			current_chord_root = roots[randi() % roots.size()]
			
		if current_step % 2 == 0 or tension_index > 0.4:
			var note_idx = current_step % scale_intervals.size()
			var midi = current_chord_root + scale_intervals[note_idx] + (12 if tension_index > 0.7 else 0)
			voice_freq = 440.0 * pow(2.0, (float(midi) - 69.0) / 12.0)
			voice_env = 1.0

	voice_env = max(0.0, voice_env - (delta * 5.5))

func _render_music() -> void:
	if gen_music == null: return
	var frames = gen_music.get_frames_available()
	while frames > 0:
		if voice_env > 0.001:
			voice_phase += (2.0 * PI * voice_freq) / SAMPLE_RATE
			if voice_phase > 2.0 * PI: voice_phase -= 2.0 * PI
			
			var mod = sin(voice_phase * 2.0) * (2.0 * voice_env)
			var carrier = sin(voice_phase + mod)
			var s = carrier * voice_env * 0.18
			gen_music.push_frame(Vector2(s, s))
		else:
			gen_music.push_frame(Vector2.ZERO)
		frames -= 1

func play_ui_click(_btn: Node) -> void:
	var p = AudioStreamPlayer.new()
	var g = AudioStreamGenerator.new()
	g.mix_rate = SAMPLE_RATE
	g.buffer_length = 0.04
	p.stream = g
	p.bus = sfx_bus_name
	add_child(p)
	p.play()
	var pb = p.get_stream_playback()
	var total = int(SAMPLE_RATE * 0.03)
	var p1 = 0.0
	var p2 = 0.0
	for i in range(total):
		var env = 1.0 - (float(i) / float(total))
		p1 += (2.0 * PI * 2400.0) / SAMPLE_RATE
		p2 += (2.0 * PI * 4800.0) / SAMPLE_RATE
		var s = (sin(p1) * 0.6 + sin(p2) * 0.4) * env * 0.3
		pb.push_frame(Vector2(s, s))
	get_tree().create_timer(0.08).timeout.connect(func(): p.queue_free())

func play_ui_hover(_btn: Node) -> void:
	var p = AudioStreamPlayer.new()
	var g = AudioStreamGenerator.new()
	g.mix_rate = SAMPLE_RATE
	g.buffer_length = 0.03
	p.stream = g
	p.bus = sfx_bus_name
	add_child(p)
	p.play()
	var pb = p.get_stream_playback()
	var total = int(SAMPLE_RATE * 0.015)
	var ph = 0.0
	for i in range(total):
		var env = 1.0 - (float(i) / float(total))
		ph += (2.0 * PI * 1800.0) / SAMPLE_RATE
		var s = sin(ph) * env * 0.12
		pb.push_frame(Vector2(s, s))
	get_tree().create_timer(0.05).timeout.connect(func(): p.queue_free())

func _on_collision_2d(body_a: RigidBody2D, _body_b: Node) -> void:
	var vel = body_a.linear_velocity.length()
	if vel < 20.0: return
	var energy = clamp(vel / 400.0, 0.0, 1.0)
	var freq = clamp(700.0 / sqrt(max(0.1, body_a.mass)), 90.0, 2800.0)
	_play_impact_synth(freq, energy, body_a.mass)

func _on_collision_3d(body_a: RigidBody3D, _body_b: Node) -> void:
	var vel = body_a.linear_velocity.length()
	if vel < 1.0: return
	var energy = clamp(vel / 20.0, 0.0, 1.0)
	var freq = clamp(550.0 / sqrt(max(0.1, body_a.mass)), 70.0, 2200.0)
	_play_impact_synth(freq, energy, body_a.mass)

func _play_impact_synth(base_freq: float, energy: float, mass: float) -> void:
	var p = AudioStreamPlayer.new()
	var g = AudioStreamGenerator.new()
	g.mix_rate = SAMPLE_RATE
	var dur = clamp(0.08 * mass, 0.05, 0.35)
	g.buffer_length = dur + 0.02
	p.stream = g
	p.bus = sfx_bus_name
	add_child(p)
	p.play()
	var pb = p.get_stream_playback()
	var total = int(SAMPLE_RATE * dur)
	var ph = 0.0
	for i in range(total):
		var t = float(i) / SAMPLE_RATE
		var env = exp(-t * (20.0 / max(0.1, mass)))
		ph += (2.0 * PI * base_freq) / SAMPLE_RATE
		var tonal = sin(ph)
		var noise = (randf() * 2.0 - 1.0) * exp(-t * 70.0)
		var s = (tonal * 0.75 + noise * 0.25) * env * energy * 0.6
		pb.push_frame(Vector2(s, s))
	get_tree().create_timer(dur + 0.04).timeout.connect(func(): p.queue_free())
