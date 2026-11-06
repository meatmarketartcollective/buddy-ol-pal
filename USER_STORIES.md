# User Stories & Coverage Map

Every user-visible behavior of `buddy`, written as a story, with what proves it.
Two markers per story:

- **[tests]** covered by the automated suite (test names in parentheses)
- **[manual]** needs a human or on-device pass (what to try is in the story)

Suite: 102 tests, all green at the latest release tag. There is no separate
`tests/` integration tree; the e2e story below is the one automated end-to-end
path. Everything glue-level is honest about being manual.

## 1. Install and first run

- **US-01 Install.** `curl | bash install.sh` puts the binary on PATH and prints the
  hook line; the Homebrew formula builds from the matching tag. **[manual]** run
  install.sh on a clean machine and check `buddy status` resolves.
- **US-02 First run hatches a pet.** First launch creates state, picks a species
  weighted by tier, and names it from that species' pool. **[tests]** (pick_is_weighted_by_tier,
  pick_weighted_honors_custom_weights, name_pools_non_empty, no_duplicate_names_across_species,
  hatched_species_is_always_known)
- **US-03 Upgrading from an older install.** The old config directory is renamed in
  place; pets and ai.toml survive. **[manual]** start from a pre-rename `~/.config/hatch`
  and check migration carried both.

## 2. Daily loop

- **US-04 Status.** `buddy status` shows the pet, mood, level, XP to next, and last
  interaction; an empty event log is not an error. **[tests]** (status_format_and_no_events)
- **US-05 XP and levels.** Feed and play grant XP with cooldowns; level-ups fire once;
  level 5 lands at 100 lifetime XP; the ladder is readable. **[tests]** (xp_gain_rules,
  interactions_cooldown_and_effects, apply_xp_detects_level_ups_once, level_boundaries,
  xp_to_next_reads_the_ladder)
- **US-06 No XP farming.** Repeated identical actions inside the cooldown pay nothing;
  the cap path never overpays. **[tests]** (interactions_cooldown_and_effects,
  cap_room_never_overpays, apply_xp_ignores_negative_gain)
- **US-07 Moods.** Petting pulses hearts; mood precedence is stable (petted over fed
  over miffed over sleeping) and decays; a miffed window has a start and an end. **[tests]**
  (petted_pulses_hearts, mood_precedence_and_decay, mood_precedence_with_miffed,
  is_miffed_windows, resting_half_closes_the_eyes, sleeping_closes_symbol_eyes_too)

## 3. Animation and art

- **US-08 Art completeness.** Every species has complete art for every mood; frames
  cycle deterministically; idle blinks every fourth tick; sleeping frames drift the
  zzz. **[tests]** (every_species_has_complete_art_for_every_mood, art_frames_cycle_per_tick,
  frames_are_deterministic, idle_blinks_every_fourth_tick, sleeping_frames_cycle_the_zzz_drift)
- **US-09 Glitch never breaks the pane.** Chaotic moods glitch text in bursts, never
  touching spaces or newlines, and nothing glitches below the chaos threshold. **[tests]**
  (glitch_never_touches_spaces_or_newlines, no_glitch_below_the_threshold,
  high_chaos_throws_bursts_not_a_constant_glitch, severity_rises_with_chaos,
  chance_and_severity_bands)

## 4. Ascension and rehatch

- **US-10 Rehatch ladder.** The first rehatch asks for 1000 total XP, each later one
  700 more (1700, 2400, 3100). **[tests]** (apply_xp_ascends_past_max_level)
- **US-11 Daily cap.** XP caps at 100 per day; the counter resets at the local day
  boundary; a backward clock resets it instead of paying out. **[tests]**
  (apply_xp_caps_every_path, day_rollover_resets_the_counter, apply_xp_survives_a_backward_clock)
- **US-12 Ascension odds.** Rehatch odds shift with each ascension and stay sane
  (weights shift and sum); rarity stays tier-weighted. **[tests]**
  (ascension_weights_shift_and_sum, pick_is_weighted_by_tier)
- **US-13 Broken state recovers.** Corrupt or version-less state rehatches a fresh pet
  and backs up the old file; unknown moods and species fall back; a torn final log line
  never crashes. **[tests]** (corrupt_state_rehatches_and_backs_up, missing_version_is_corrupt,
  unknown_mood_in_state_is_tolerated, unknown_species_in_state_falls_back_to_ember,
  torn_final_line_never_crashes)

## 5. Species, packs, dex

- **US-14 Dex hides the unknown.** `buddy dex` shows only encountered species. **[tests]**
  (dex_hides_undiscovered_species)
- **US-15 Community packs are safe.** New species append, overrides replace builtins, a
  broken pack is skipped whole, shipped packs parse, names stay unique, and every species
  can speak in every mood and carries a known tier. **[tests]** (pack_new_species_appends,
  pack_override_replaces_builtin, broken_pack_is_skipped_whole, shipped_packs_parse_and_stay_unique,
  table_loads_and_names_are_unique, every_species_can_speak_in_every_mood,
  every_species_carries_a_known_tier, every_species_has_deep_canned_lines)

## 6. AI tiers

- **US-16 Offline by default.** No config, or AI disabled, means a fully working buddy
  with no AI features and no error noise. **[tests]** (missing_config_means_no_ai,
  disabled_config_means_no_ai, gate_demands_config_and_tier)
- **US-17 Tier probe.** `ai test` grants exactly the tier the model supports
  (1 text, 2 tool calls, 3 full agent); tier 2 needs a capable model. **[tests]**
  (gate_demands_config_and_tier) **[manual]** live sweep against the provider account.
- **US-18 Ask and its edges.** `ai ask` answers; parsing handles both response shapes,
  reasoning-only models, blank content, think blocks, and server errors; transient
  provider failures retry with backoff. **[tests]** (parse_handles_both_shapes_and_garbage,
  falls_back_to_reasoning_content_when_content_null, falls_back_when_content_blank,
  strips_think_blocks_from_content, unterminated_think_block_yields_nothing,
  server_error_falls_back, transient_failure_classifies_nvidia_flakiness,
  failure_streak_counts_trailing_failures, end_to_end_over_plain_http)
- **US-19 Privacy of the payload.** Only name, species, mood, last event kind, exit
  code, and duration ever leave the machine; never file contents or credentials. **[tests]**
  (openai_body_carries_only_safe_facts, sanitize_strips_quotes_and_first_line_only,
  b64_output_is_shell_safe, anthropic_body_uses_the_messages_shape,
  any_url_endpoint_is_openai_compatible, resolve_defaults_to_nvidia)
- **US-20 Agent tier 3.** Stepwise tasks with strict JSON (bare, fenced, or prose-wrapped),
  per-step exit codes and captured output, no loop past max steps, recap shape stable.
  **[tests]** (agent_json_parses_bare_fenced_and_prose, run_capture_reports_exit_and_output,
  bare_command_is_kept, working_spins, digest_and_recap_shape, events_append_and_tail,
  tail_is_bounded_and_ordered, emit_args_parse, empty_args_survive,
  unknown_emit_kind_is_logged_but_calm)
- **US-21 Agent safety gate.** Destructive commands are caught and need explicit
  confirmation; shell writes are flagged; reads and harmless shapes pass. **[tests]**
  (destructive_commands_are_caught, shell_writes_are_flagged, reads_and_harmless_shapes_pass)
  **[manual]** the live write gate and the headless `--yes` path, once on device.
- **US-22 Voice.** Spoken replies clamp to a short length with a marker. **[tests]**
  (clip_truncates_with_marker) **[manual]** one live voice pass after any voice change.

## 7. Agent window, events, notifications

- **US-23 Own window.** The agent runs in its own tmux window with its env forwarded;
  works inside proot where the outer TMUX var is stripped. **[manual]** nested tmux
  kick on the phone, then capture the window.
- **US-24 Notifications tell the truth.** Notification messages name the outcome; a
  stale active marker expires by the staleness threshold. **[tests]**
  (notification_messages_name_the_outcome, stale_active_marker_expires, staleness_threshold)
- **US-25 Event log hygiene.** Archives rotate and cap; tails are bounded and ordered;
  a torn final line is safe. **[tests]** (rotation_at_boundary, rotation_caps_archives,
  tail_is_bounded_and_ordered, torn_final_line_never_crashes, events_append_and_tail)

## 8. Config and migration

- **US-26 State migrations.** v1 through v6 migrations keep pets, XP, and level across
  upgrades; round-trips are lossless. **[tests]** (v1_state_migrates_fully,
  v2_state_migrates_to_v3, v2_state_round_trip_keeps_level, v4_state_migrates_to_v5,
  v5_state_migrates_to_v6, state_round_trip)
- **US-27 Concurrent access.** A state lock stops two processes from corrupting state.
  **[manual]** run two instances at once and check the second waits or exits cleanly.
- **US-28 Command rename.** The command is `buddy` everywhere: every subcommand, the
  BUDDY_* env vars, shell hooks, the agent tmux window, and all docs. **[manual]**
  `buddy status`, `buddy dex`, `buddy ai test` after upgrading from a `tb` install.

## 9. Packaging and platforms

- **US-29 Releases are real.** Every merged batch gets a fresh tagged release with
  compiled binaries for all four targets and checksums. **[manual]** check the latest
  release assets against the tag.
- **US-30 Cross-platform.** Linux x86_64 (static musl) and aarch64, macOS x86_64 and
  Apple Silicon all build and run; MSRV is Rust 1.89+ (std file locking). **[manual]**
  run each released binary once on its platform.
