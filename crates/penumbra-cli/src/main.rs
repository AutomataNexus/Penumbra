use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "penumbra",
    about = "Penumbra CLI — project scaffolding, examples, and benchmarks for the 3D rendering SDK"
)]
#[command(version, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Penumbra in a new or existing Rust project
    Init {
        /// Project name (creates a new directory)
        #[arg(default_value = "my-penumbra-app")]
        name: String,

        /// Include terrain/geo crates
        #[arg(long)]
        geo: bool,

        /// Include all crates
        #[arg(long)]
        full: bool,
    },

    /// Add Penumbra crates to an existing Cargo.toml
    Add {
        /// Crate names to add (e.g. scene, pbr, terrain)
        crates: Vec<String>,

        /// Add all crates
        #[arg(long)]
        all: bool,
    },

    /// List all available crates with descriptions
    List,

    /// Show system info (GPU, Vulkan, wgpu adapters)
    Info,

    /// Run a built-in example
    Example {
        /// Example name
        name: String,

        /// Pass --release to cargo
        #[arg(long)]
        release: bool,
    },

    /// Run performance benchmarks
    Bench {
        /// Specific benchmark (instancing, tile_streaming, full_scene, gpu)
        name: Option<String>,
    },

    /// Open documentation in the browser
    Docs,
}

const CRATE_REGISTRY: &[(&str, &str)] = &[
    ("core", "Renderer, RenderFrame, Material, DrawCall, math re-exports"),
    ("backend", "RenderBackend trait — GPU abstraction layer"),
    ("wgpu", "Default wgpu backend (Vulkan/Metal/DX12/WebGPU/WebGL2)"),
    ("scene", "Scene graph, transform hierarchy, frustum culling, LOD"),
    ("pbr", "PBR pipeline, Cook-Torrance BRDF, lights, IBL"),
    ("instance", "GPU instanced rendering, 27K+ entities at 60fps"),
    ("terrain", "Tile streaming, terrain mesh, LRU cache, Terrain-RGB"),
    ("atmosphere", "Bruneton-Neyret scattering, fog, sun/moon"),
    ("post", "Tone mapping, bloom, SSAO, FXAA, color grading"),
    ("shadow", "Cascaded shadow maps, PCF, point light cubemaps"),
    ("text", "SDF font rendering, billboard text, batched labels"),
    ("compute", "Compute shader abstraction, GPU frustum culling"),
    ("geo", "WGS84 geodesy, ECEF/ENU, haversine, tile math"),
    ("immediate", "Per-frame draw API: lines, shapes, billboards"),
    ("camera", "Perspective, orthographic, orbit, fly, globe controllers"),
    ("asset", "glTF 2.0, OBJ, PNG/JPEG, primitive generators"),
    ("winit", "winit window integration, input handling"),
    ("web", "WASM target, browser surface, fetch tile loading"),
];

const EXAMPLES: &[(&str, &str)] = &[
    ("hello_cube", "PBR-lit cube with orbit camera and 3 lights"),
    ("pbr_scene", "Multiple PBR materials, shadows, post-processing"),
    ("instanced_entities", "27K instanced entities with frustum culling"),
    ("globe", "Full globe with satellite tiles, terrain, atmosphere"),
    ("tactical_globe", "NexusPulse Tactical demo: 27K entities + globe + HUD"),
    ("wasm", "Browser WASM example (build with wasm-pack)"),
];

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, geo, full } => init_project(&name, geo, full),
        Commands::Add { crates, all } => add_crates(&crates, all),
        Commands::List => list_crates(),
        Commands::Info => show_info(),
        Commands::Example { name, release } => run_example(&name, release),
        Commands::Bench { name } => run_bench(name.as_deref()),
        Commands::Docs => open_docs(),
    }
}

fn init_project(name: &str, geo: bool, full: bool) {
    println!("Initializing Penumbra project: {}", name);

    // Create project with cargo
    let status = Command::new("cargo")
        .args(["init", name])
        .status();

    match status {
        Ok(s) if s.success() => println!("  Created Rust project: {}", name),
        Ok(_) => {
            eprintln!("  cargo init failed (project may already exist)");
        }
        Err(e) => {
            eprintln!("  Failed to run cargo: {}", e);
            std::process::exit(1);
        }
    }

    let cargo_path = Path::new(name).join("Cargo.toml");
    if !cargo_path.exists() {
        eprintln!("Error: {}/Cargo.toml not found", name);
        std::process::exit(1);
    }

    let mut deps = vec![
        "penumbra-core",
        "penumbra-backend",
        "penumbra-wgpu",
        "penumbra-scene",
        "penumbra-pbr",
        "penumbra-camera",
        "penumbra-asset",
    ];

    if geo || full {
        deps.push("penumbra-geo");
        deps.push("penumbra-terrain");
        deps.push("penumbra-atmosphere");
    }

    if full {
        deps.push("penumbra-instance");
        deps.push("penumbra-shadow");
        deps.push("penumbra-post");
        deps.push("penumbra-text");
        deps.push("penumbra-compute");
        deps.push("penumbra-immediate");
        deps.push("penumbra-winit");
        deps.push("penumbra-web");
    }

    let mut cargo_content = fs::read_to_string(&cargo_path).expect("Failed to read Cargo.toml");
    if !cargo_content.contains("[dependencies]") {
        cargo_content.push_str("\n[dependencies]\n");
    }

    let dep_block: String = deps
        .iter()
        .map(|d| format!("{} = {{ git = \"https://github.com/AutomataNexus/Penumbra.git\" }}", d))
        .collect::<Vec<_>>()
        .join("\n");

    cargo_content.push_str(&format!("\n# Penumbra 3D Rendering SDK\n{}\nglam = \"0.29\"\n", dep_block));
    fs::write(&cargo_path, cargo_content).expect("Failed to write Cargo.toml");
    println!("  Added {} Penumbra crates to Cargo.toml", deps.len());

    // Write a starter main.rs
    let main_path = Path::new(name).join("src").join("main.rs");
    let main_content = r#"use penumbra_core::{Renderer, RendererConfig, Material, Rgba};
use penumbra_wgpu::{WgpuBackend, WgpuConfig};
use penumbra_camera::OrbitController;
use penumbra_scene::{Scene, Transform};
use penumbra_asset::cube_mesh;
use penumbra_pbr::{Light, PbrPipeline, PbrConfig};
use glam::Vec3;

fn main() {
    // Create GPU backend
    let backend = WgpuBackend::headless(1280, 720, WgpuConfig::default())
        .expect("GPU init failed");
    let mut renderer = Renderer::new(backend, RendererConfig::default());

    // Create a mesh and material
    let mesh = renderer.create_mesh(cube_mesh()).unwrap();
    let mat = renderer.add_material(Material {
        albedo: Rgba::new(0.8, 0.2, 0.1, 1.0),
        metallic: 0.6,
        roughness: 0.4,
        ..Material::default()
    });

    // Scene graph
    let mut scene = Scene::new();
    scene.add_mesh(mesh.id, mat);
    scene.update_transforms();

    // Camera
    let orbit = OrbitController::default();
    let camera = orbit.camera();

    // Lighting
    let mut pbr = PbrPipeline::new(PbrConfig::default());
    pbr.add_light(Light::Directional {
        direction: [-0.5, -1.0, -0.3],
        color: [1.0, 0.98, 0.95],
        intensity: 10.0,
        shadows: true,
    });

    // Render
    let mut frame = renderer.begin_frame().unwrap();
    frame.set_camera(camera.view_matrix(), camera.projection_matrix(), 0.1, 1000.0);
    renderer.end_frame(frame).unwrap();

    println!("Penumbra: frame rendered, {} draw calls", renderer.stats().draw_calls);
}
"#;
    fs::write(&main_path, main_content).expect("Failed to write main.rs");
    println!("  Wrote starter main.rs");

    println!("\nDone. Next steps:");
    println!("  cd {}", name);
    println!("  cargo run");
}

fn add_crates(crates: &[String], all: bool) {
    if !Path::new("Cargo.toml").exists() {
        eprintln!("Error: Cargo.toml not found in current directory.");
        std::process::exit(1);
    }

    let targets: Vec<&str> = if all {
        CRATE_REGISTRY.iter().map(|(name, _)| *name).collect()
    } else {
        if crates.is_empty() {
            eprintln!("Error: no crate names provided. Use `penumbra add scene pbr` or `penumbra add --all`.");
            std::process::exit(1);
        }
        crates.iter().map(|s| {
            let name = s.strip_prefix("penumbra-").unwrap_or(s);
            if CRATE_REGISTRY.iter().any(|(n, _)| *n == name) {
                name
            } else {
                eprintln!("Error: unknown crate '{}'. Run `penumbra list`.", s);
                std::process::exit(1);
            }
        }).collect()
    };

    let mut cargo = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let mut added = 0;

    for name in &targets {
        let crate_name = format!("penumbra-{}", name);
        if cargo.contains(&crate_name) {
            println!("  {} — already in Cargo.toml", crate_name);
            continue;
        }
        let dep_line = format!(
            "{} = {{ git = \"https://github.com/AutomataNexus/Penumbra.git\" }}\n",
            crate_name
        );
        cargo.push_str(&dep_line);
        println!("  {} — added", crate_name);
        added += 1;
    }

    fs::write("Cargo.toml", cargo).expect("Failed to write Cargo.toml");
    println!("\nAdded {} crate(s).", added);
}

fn list_crates() {
    println!("Penumbra Crates:\n");
    for (name, desc) in CRATE_REGISTRY {
        println!("  penumbra-{:<14} {}", name, desc);
    }
    println!("\n  Total: {} crates", CRATE_REGISTRY.len());
    println!("\nExamples:\n");
    for (name, desc) in EXAMPLES {
        println!("  {:<22} {}", name, desc);
    }
    println!("\n  Total: {} examples", EXAMPLES.len());
}

fn show_info() {
    println!("Penumbra System Info\n");

    // Rust version
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        println!("  Rust: {}", String::from_utf8_lossy(&output.stdout).trim());
    }

    // wgpu adapter info (via gpu_probe if available)
    println!("  GPU:  Run `cargo run --bin gpu_probe --release` for adapter details");
    println!("  WASM: {}", if cfg!(target_arch = "wasm32") { "yes" } else { "no (native)" });

    // Check if wasm-pack is installed
    let wasm_pack = Command::new("wasm-pack").arg("--version").output();
    match wasm_pack {
        Ok(o) if o.status.success() => {
            println!("  wasm-pack: {}", String::from_utf8_lossy(&o.stdout).trim());
        }
        _ => println!("  wasm-pack: not installed"),
    }
}

fn run_example(name: &str, release: bool) {
    if !EXAMPLES.iter().any(|(n, _)| *n == name) {
        eprintln!("Error: unknown example '{}'. Available:", name);
        for (n, desc) in EXAMPLES {
            eprintln!("  {:<22} {}", n, desc);
        }
        std::process::exit(1);
    }

    if name == "wasm" {
        println!("WASM example — build with:");
        println!("  wasm-pack build examples/wasm --target web");
        println!("  # Then serve examples/wasm/ and open index.html");
        return;
    }

    let mut args = vec!["run", "-p", name];
    if release {
        args.push("--release");
    }

    println!("Running example: {}", name);
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .expect("Failed to run cargo");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_bench(name: Option<&str>) {
    match name {
        Some("gpu") => {
            println!("Running GPU render benchmark...");
            let status = Command::new("cargo")
                .args(["run", "--bin", "gpu_render_bench", "--release"])
                .status()
                .expect("Failed to run cargo");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Some(bench_name) => {
            let valid = ["instancing", "tile_streaming", "full_scene"];
            if !valid.contains(&bench_name) {
                eprintln!("Error: unknown benchmark '{}'. Available: {}, gpu", bench_name, valid.join(", "));
                std::process::exit(1);
            }
            println!("Running benchmark: {}", bench_name);
            let status = Command::new("cargo")
                .args(["bench", "--bench", bench_name])
                .status()
                .expect("Failed to run cargo");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        None => {
            println!("Running all CPU benchmarks...");
            let status = Command::new("cargo")
                .args(["bench"])
                .status()
                .expect("Failed to run cargo");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
        }
    }
}

fn open_docs() {
    let url = "https://github.com/AutomataNexus/Penumbra";
    println!("Opening Penumbra docs: {}", url);

    #[cfg(target_os = "linux")]
    { let _ = Command::new("xdg-open").arg(url).spawn(); }
    #[cfg(target_os = "macos")]
    { let _ = Command::new("open").arg(url).spawn(); }
    #[cfg(target_os = "windows")]
    { let _ = Command::new("cmd").args(["/c", "start", url]).spawn(); }
}
