# music-forge

A Rust CLI tool for managing music projects with git-native version control, automated file watching, and project lifecycle tracking.

## Motivation

Music production suffers from a dependency and recoverability problem that software engineering solved years ago. DAW sessions reference scattered sample libraries, plugins that may not exist on future machines, and file paths that break across systems. There is no standard for version control, progress tracking, or project organization in music production.

music-forge treats each song as a self-contained git repository with all dependencies local, version history native to git, and project lifecycle tracked through git tags. The goal is to bring the reliability and traceability of software engineering workflows to music production without adding friction to the creative process.

## Design Principles

- **Git is the source of truth.** All versioning, history, and progress tracking lives in git. No parallel bookkeeping systems, no separate databases, no sync problems.
- **Zero-friction project creation.** Starting a new project is one command and under 10 seconds. The tool should never make you hesitate to start something new.
- **Audio files are the primary artifact.** DAW sessions are transient workspaces. The durable, recoverable output is audio — bounces, stems, field recordings. Everything needed to reconstruct or revisit a project should be self-contained in the project directory.
- **Minimal metadata, maximal queryability.** Keep structured data to an absolute minimum (`song.yaml` with title, tags, description). Derive everything else — timeline, status, activity, velocity — from git history and tags.
- **Progressive complexity.** Seeds are nearly zero-ceremony. Full projects have structure. The dashboard aggregates everything. Each layer is optional and additive.

## Project Structure

### Full Project (created via `music-forge init <name>`)

```
<name>/
├── .git/
├── .gitattributes          # LFS tracking rules
├── .gitignore              # DAW cache files, OS cruft
├── song.yaml               # Minimal metadata (title, tags, description)
├── notes.md                # Freeform: lyrics, concepts, ideas
├── *.rpp                   # Reaper session (always present — rendering backend)
├── *.bwproject             # Bitwig session (optional — instrument/sound design layer)
├── assets/
│   ├── stems/              # Bounced stems from Bitwig (DAW handoff point)
│   ├── samples/            # Samples used in the project
│   ├── field-recordings/   # Phone recordings, tape transfers, etc.
│   └── references/         # Reference tracks, inspiration
├── exports/                # Rendered audio (tagged with commit hash)
└── promo/                  # Artwork, press materials, social content
```

### song.yaml Schema

```yaml
title: "song name"
created: 2026-03-03
tags: []
description: ""
```

This file is intentionally minimal. Status is tracked via git tags. History is tracked via git log. The only purpose of `song.yaml` is identity metadata that doesn't belong in git's own metadata.

### Seeds Monorepo (created via `music-forge seed <name>`)

```
~/Music/Seeds/
├── .git/
├── .gitattributes
├── 2026-03-03-sketch-idea/
│   ├── notes.md
│   └── (whatever files)
├── 2026-03-05-weird-synth-thing/
│   └── ...
```

Seeds are low-friction sketches. Date-prefixed subdirectories in a single git repo. No ceremony, no YAML, no structure beyond a notes file. Seeds that grow into real projects get promoted.

## CLI Commands

### Project Lifecycle

#### `music-forge init <name>`
Create a new full project.

- Creates directory structure as defined above
- Runs `git init` and `git lfs install`
- Configures LFS tracking for audio file extensions (wav, flac, mp3, aif, aiff, ogg, mp4, mov)
- Copies Reaper template into the project root as `<n>.rpp` (every project gets a Reaper session by default)
- Generates `song.yaml` and `notes.md` from templates
- Creates `.gitignore` with DAW-specific patterns (reapeaks, bwproject cache, OS files)
- Makes initial commit
- Opens Reaper with the new project file
- Opens configured editor (e.g., Zed) on the project directory

#### `music-forge seed <name>`
Create a lightweight sketch in the seeds monorepo.

- Initializes the seeds monorepo on first use (git init, LFS setup)
- Creates a date-prefixed subdirectory (`YYYY-MM-DD-<name>`)
- Drops in a blank `notes.md`
- Does not commit automatically (user commits when ready)

#### `music-forge promote <seed-path>`
Graduate a seed into a full project.

- Runs the full `init` workflow for a new project
- Copies seed contents into the new project's `assets/` directory
- Prints reminder to delete the seed from the monorepo when ready

### Version Control

#### `music-forge log "<message>"`
Shorthand for a descriptive commit.

- Must be run from within a project directory
- Equivalent to `git add -A && git commit -m "<message>"`
- Validates that there are actually changes to commit

#### `music-forge snapshot ["<message>"]`
Commit with an associated audio render.

- Commits current state with a `snapshot: ` prefixed message
- Calls Reaper CLI to render the current session into `exports/` (if Reaper project exists and Reaper is available)
- Names the export with date and short commit hash (e.g., `2026-03-03-a1b2c3d.wav`)
- Falls back to commit-only if no DAW project or CLI is available
- The `snapshot:` prefix allows the tracker/dashboard to distinguish render points from regular commits

#### `music-forge watch [dir]`
Auto-commit on file changes with debounce.

- Watches the project directory (or specified directory) for file changes
- Excludes `.git/` from watching
- Debounce period: 5 minutes (configurable via `--debounce <seconds>`)
- On trigger: `git add -A && git commit -m "auto: YYYY-MM-DD HH:MM"`
- Only commits if there are actual changes (`git status --porcelain` is non-empty)
- Prints a notice to the terminal on each auto-commit
- Runs until interrupted (Ctrl-C)

### Status Tracking

#### `music-forge status <status>`
Tag the current commit with a project lifecycle stage.

- Valid statuses: `idea`, `drafting`, `arranging`, `mixing`, `mastering`, `released`, `abandoned`
- Creates a git tag: `status/<status>` (e.g., `status/mixing`)
- If a previous tag with the same status prefix exists, it remains (history is preserved — you can see when a project entered and re-entered stages)
- Tags are timestamped, giving the tracker full lifecycle data
- Validates that the status is one of the allowed values

### Asset Management

#### `music-forge ingest <file>...`
Import files into the current project.

- Copies files into the appropriate `assets/` subdirectory based on file type or a `--to` flag
- Renames with date prefix: `YYYY-MM-DD-<original-name>`
- Stages the files in git
- Supports glob patterns

### Dashboard & Tracking

#### `music-forge timeline`
Show the git history of the current project as a formatted timeline.

- Reads `git log` for the current project
- Highlights `snapshot:` commits (render points)
- Shows status tags at the commits where they were applied
- Formatted for terminal output with colors and structure

#### `music-forge dashboard`
Aggregate view across all projects.

- Walks the configured projects directory
- For each project: reads `song.yaml`, reads git log, reads git tags
- Displays: project name, current status (latest `status/*` tag), last activity date, commit count, time in current stage
- Sorted by last activity (most recent first)
- Renders as a TUI table via ratatui, or as formatted terminal output as a simpler initial implementation

#### `music-forge stats`
Analytics across all projects.

- Average time per lifecycle stage
- Projects started vs. completed over time
- Activity heatmap (commits per day/week)
- Longest-idle projects
- Output format: terminal, or JSON for consumption by external tools

## Configuration

Configuration lives in `~/.config/music-forge/config.toml`:

```toml
# Required
projects_dir = "~/Music/Projects"
seeds_dir = "~/Music/Seeds"
reaper_template = "~/Templates/default.rpp"  # Copied into every new project

# Optional
editor = "zed"                          # Editor to open on project creation
reaper_command = "reaper"               # Reaper CLI path (auto-detected if on PATH)
watch_debounce_seconds = 300            # Auto-commit debounce (default: 300)

# LFS file extensions to track
lfs_extensions = ["wav", "flac", "mp3", "aif", "aiff", "ogg", "mp4", "mov"]
```

First run without a config file should either generate a default or prompt interactively.

## Dependencies (Rust Crates)

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing with derive macros |
| `git2` | libgit2 bindings — all git operations without shelling out |
| `notify` | Cross-platform filesystem watcher |
| `serde` + `serde_yaml` | song.yaml and config.toml parsing |
| `toml` | Config file parsing |
| `chrono` | Timestamp handling |
| `ratatui` + `crossterm` | TUI dashboard (can defer to v2) |
| `colored` | Terminal output formatting |
| `dirs` | XDG-compliant config/data paths |

## Implementation Phases

### Phase 1: Core CLI
- `init`, `seed`, `promote`
- `log`, `watch`
- `status` (git tagging)
- `ingest`
- Config file loading
- This replaces the bash script entirely

### Phase 2: Tracking & Visualization
- `timeline` (single project)
- `dashboard` (multi-project aggregation)
- Terminal-formatted output (no TUI yet)

### Phase 3: TUI Dashboard
- Ratatui-based interactive dashboard
- Navigate between projects
- View commit history inline
- Filter by status, sort by activity

### Phase 4: Extensions
- `snapshot` with DAW CLI integration
- `stats` with analytics
- JSON export for external frontends
- Potential web dashboard (Rust → WASM, or just serve JSON to a React app)

## Reaper as Rendering Backend

Reaper is the ever-present backend for every music-forge project. Every project gets a Reaper session by default — it's where mixing, rendering, and CLI-automated operations happen. Reaper is the constant.

Bitwig is an optional instrument layer. When a project involves synthesis, sound design, or modulation-heavy composition, Bitwig serves that role and hands off bounced stems to Reaper. But many projects may never touch Bitwig — a song built from field recordings, samples, or live tracking might live entirely in Reaper. The tool should not assume Bitwig's involvement.

This separation is load-bearing for music-forge. Bitwig has no CLI, so any automated rendering (snapshots, CI-like export pipelines, batch operations) must go through Reaper. When Bitwig is involved, the handoff point is a stem bounce into `assets/stems/`, which naturally produces the audio-files-as-primary-artifact model that makes projects recoverable long-term.

### Project Workflow

The minimal workflow (no Bitwig):

1. Record, arrange, and mix in Reaper
2. `music-forge snapshot` triggers Reaper's CLI to render into `exports/`

The Bitwig-assisted workflow:

1. Compose and sound design in Bitwig
2. Bounce stems into `assets/stems/`
3. Import stems into the Reaper session
4. Mix, master, and render from Reaper
5. `music-forge snapshot` triggers Reaper's CLI to render into `exports/`

### Reaper CLI Integration

Reaper supports command-line rendering via:

```
reaper -renderproject /path/to/project.rpp
```

This renders using whatever render settings are saved in the project file. music-forge should:

- Detect `.rpp` files in the project root
- Use the Reaper CLI for `snapshot` renders
- Name exports with date and short commit hash: `YYYY-MM-DD-<hash>.wav`
- Support a `--dry-run` flag that commits without rendering

### Template Project

The `reaper_template` config option should point to a `.rpp` file that has sensible default render settings pre-configured (output format, sample rate, dither, etc.). On `music-forge init`, this template is copied into the project directory as `<n>.rpp`. The user configures their preferred render settings once in the template and every new project inherits them.

### Directory Structure Update

The `assets/` directory gains a `stems/` subdirectory for the Bitwig→Reaper handoff:

```
<n>/
├── assets/
│   ├── stems/              # Bounced stems from Bitwig (the DAW handoff point)
│   ├── samples/
│   ├── field-recordings/
│   └── references/
```

## Open Questions

- **Stem naming convention:** Should music-forge enforce or suggest a naming pattern for stems (e.g., `drums.wav`, `bass.wav`, `vox-lead.wav`), or leave this entirely to the user? - NO

- **Remote backup:** Git LFS needs a remote for push. Should the tool have an opinion about this (e.g., `music-forge remote setup` that configures a Gitea instance or B2 bucket), or leave it to the user? Gitea self-hosted is the likely recommendation for unlimited LFS storage at low cost. - Yes, but this in not initial dev phase.

- **Reaper render settings override:** Should `snapshot` accept flags to override render format (e.g., `--format mp3` for a quick preview vs. the default wav), or should it always use whatever's in the .rpp? - Not yet

- **Bitwig project inclusion:** When Bitwig is used as the instrument layer, should the `.bwproject` file be committed to git? It's binary and adds bulk, but it's part of the creative record. Likely yes, tracked via LFS, with the understanding that the stems in `assets/stems/` are the durable artifact, not the Bitwig session itself. - YES
