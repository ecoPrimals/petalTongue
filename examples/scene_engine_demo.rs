// SPDX-License-Identifier: AGPL-3.0-only
//! Scene engine demonstration -- exercises grammar, scene graph, Tufte,
//! math objects, modality compilers, animation, and physics bridge.
//!
//! Run with: cargo run --example scene_engine_demo -- [subcommand]
//! Subcommands: grammar, scene-graph, tufte, math-objects, animation,
//!              svg, audio, accessibility, physics, all

use petal_tongue_scene::{
    animation::{Animation, AnimationState, Easing, Sequence},
    compiler::GrammarCompiler,
    grammar::{GeometryType, GrammarExpr},
    math::{Axes, FunctionPlot, MathObject, NumberLine, ParametricCurve, VectorField},
    modality::{AudioCompiler, DescriptionCompiler, ModalityCompiler, SvgCompiler},
    physics::{CollisionShape, PhysicsBody, PhysicsWorld},
    primitive::{Color, Primitive, StrokeStyle},
    scene_graph::{SceneGraph, SceneNode},
    transform::Transform2D,
    tufte::{ChartjunkDetection, DataInkRatio, TufteReport},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map_or("all", String::as_str);

    match cmd {
        "grammar" => demo_grammar(),
        "scene-graph" => demo_scene_graph(),
        "tufte" => demo_tufte(),
        "math-objects" => demo_math_objects(),
        "animation" => demo_animation(),
        "svg" => demo_svg(),
        "audio" => demo_audio(),
        "accessibility" => demo_accessibility(),
        "physics" => demo_physics(),
        "all" => {
            demo_grammar();
            demo_scene_graph();
            demo_tufte();
            demo_math_objects();
            demo_animation();
            demo_svg();
            demo_audio();
            demo_accessibility();
            demo_physics();
            println!("=== All demos passed ===");
        }
        other => {
            eprintln!("Unknown subcommand: {other}");
            eprintln!(
                "Usage: scene_engine_demo [grammar|scene-graph|tufte|math-objects|animation|svg|audio|accessibility|physics|all]"
            );
            std::process::exit(1);
        }
    }
}

fn json_array(val: &serde_json::Value) -> Vec<serde_json::Value> {
    val.as_array().cloned().unwrap_or_default()
}

fn demo_grammar() {
    println!("=== Grammar of Graphics Compilation ===");

    let expr = GrammarExpr::new("patient_vitals", GeometryType::Line)
        .with_x("time_hours")
        .with_y("heart_rate")
        .with_title("Patient Heart Rate Over 24 Hours")
        .with_domain("health");

    println!("  GrammarExpr: {} -> {:?}", expr.data_source, expr.geometry);
    println!("  Variables: {}", expr.data_dimensions());
    println!("  Domain: {:?}", expr.domain);
    println!("  3D coords: {}", expr.uses_3d_coord());

    let data = json_array(&serde_json::json!([
        {"time_hours": 0, "heart_rate": 72},
        {"time_hours": 4, "heart_rate": 68},
        {"time_hours": 8, "heart_rate": 75},
        {"time_hours": 12, "heart_rate": 82},
        {"time_hours": 16, "heart_rate": 78},
        {"time_hours": 20, "heart_rate": 70},
        {"time_hours": 24, "heart_rate": 71}
    ]));

    let compiler = GrammarCompiler;
    let scene = compiler.compile(&expr, &data);
    println!("  Scene nodes: {}", scene.node_count());
    println!("  Total primitives: {}", scene.total_primitives());
    println!("  RESULT: grammar compilation OK");
    println!();
}

fn demo_scene_graph() {
    println!("=== Scene Graph Operations ===");

    let mut graph = SceneGraph::new();
    println!("  Root node: {:?}", graph.root_id());

    let axis_node = SceneNode::new("axis-group")
        .with_label("axis-group")
        .with_transform(Transform2D::translate(50.0, 50.0));
    graph.add_node(axis_node, "root");

    let point_node = SceneNode::new("data-points")
        .with_label("data-points")
        .with_primitive(Primitive::Point {
            x: 100.0,
            y: 200.0,
            radius: 5.0,
            fill: Some(Color::rgba(0.2, 0.6, 1.0, 1.0)),
            stroke: None,
            data_id: Some("point-1".into()),
        });
    graph.add_to_root(point_node);

    let x_axis_node = SceneNode::new("x-axis")
        .with_label("x-axis")
        .with_primitive(Primitive::Line {
            points: vec![[0.0, 0.0], [400.0, 0.0]],
            stroke: StrokeStyle::default(),
            closed: false,
            data_id: None,
        });
    graph.add_node(x_axis_node, "axis-group");

    println!("  Nodes: {}", graph.node_count());
    println!("  Primitives: {}", graph.total_primitives());

    let flat = graph.flatten();
    println!("  Flattened primitives: {}", flat.len());

    let found = graph.find_by_data_id("point-1");
    println!(
        "  find_by_data_id(\"point-1\"): {}",
        if found.is_empty() {
            "not found"
        } else {
            "found"
        }
    );

    graph.remove("data-points");
    println!("  After remove: {} nodes", graph.node_count());
    println!("  RESULT: scene graph operations OK");
    println!();
}

fn demo_tufte() {
    println!("=== Tufte Constraint Validation ===");

    let data_ink = DataInkRatio;
    let chartjunk = ChartjunkDetection;
    let constraints: Vec<&dyn petal_tongue_scene::tufte::TufteConstraint> =
        vec![&data_ink, &chartjunk];

    let mut primitives = Vec::new();
    for i in 0..10 {
        let x = f64::from(i) * 40.0;
        primitives.push(Primitive::Point {
            x,
            y: (x * 0.1).sin() * 100.0 + 150.0,
            radius: 3.0,
            fill: Some(Color::rgba(0.1, 0.4, 0.8, 1.0)),
            stroke: None,
            data_id: Some(format!("d-{i}")),
        });
    }

    let expr = GrammarExpr::new("test_data", GeometryType::Point)
        .with_x("x")
        .with_y("y");

    let report = TufteReport::evaluate_all(&constraints, &primitives, &expr, None);
    println!("  Constraints evaluated: {}", report.results.len());
    for (name, r) in &report.results {
        println!(
            "    {} : {:.2} {}",
            name,
            r.score,
            if r.passed { "PASS" } else { "FAIL" }
        );
    }
    println!("  Overall score: {:.2}", report.overall_score);
    println!("  RESULT: tufte validation OK");
    println!();
}

fn demo_math_objects() {
    println!("=== Math Objects (Manim-style) ===");

    let number_line = NumberLine {
        start: -5.0,
        end: 5.0,
        step: 1.0,
        origin_x: 50.0,
        origin_y: 250.0,
        length: 500.0,
        color: Color::WHITE,
        show_labels: true,
        label_font_size: 12.0,
    };
    let nl_prims = number_line.to_primitives();
    println!("  NumberLine [-5, 5]: {} primitives", nl_prims.len());

    let axes = Axes {
        x_range: (-3.0, 3.0, 1.0),
        y_range: (-2.0, 2.0, 1.0),
        origin: (300.0, 300.0),
        width: 400.0,
        height: 300.0,
        color: Color::WHITE,
        show_labels: true,
        label_font_size: 12.0,
    };
    let axes_prims = axes.to_primitives();
    println!("  Axes [-3,3]x[-2,2]: {} primitives", axes_prims.len());

    let screen = axes.data_to_screen(1.0, 1.0);
    let data = axes.screen_to_data(screen.0, screen.1);
    println!(
        "  data_to_screen(1,1) = ({:.0}, {:.0}), round-trip = ({:.1}, {:.1})",
        screen.0, screen.1, data.0, data.1
    );

    let stroke = StrokeStyle {
        color: Color::rgba(0.2, 0.8, 0.4, 1.0),
        width: 2.0,
        ..StrokeStyle::default()
    };
    let func_plot = FunctionPlot::sample(&axes, f64::sin, stroke);
    let func_prims = func_plot.to_primitives();
    println!("  FunctionPlot sin(x): {} primitives", func_prims.len());

    let circle_stroke = StrokeStyle {
        color: Color::rgba(1.0, 0.5, 0.0, 1.0),
        width: 2.0,
        ..StrokeStyle::default()
    };
    let param_curve = ParametricCurve::sample(
        |t: f64| t.cos() * 100.0 + 300.0,
        |t: f64| t.sin() * 100.0 + 300.0,
        (0.0, std::f64::consts::TAU),
        64,
        circle_stroke,
    );
    let param_prims = param_curve.to_primitives();
    println!(
        "  ParametricCurve (circle): {} primitives",
        param_prims.len()
    );

    let vf = VectorField::from_fn(
        |x: f64, y: f64| (-y * 0.01, x * 0.01),
        (100.0, 500.0),
        (100.0, 500.0),
        8,
        30.0,
        Color::rgba(0.6, 0.3, 0.9, 0.7),
    );
    let vf_prims = vf.to_primitives();
    println!("  VectorField (rotation): {} primitives", vf_prims.len());

    println!("  RESULT: math objects OK");
    println!();
}

fn demo_animation() {
    println!("=== Animation System ===");

    let fade_in = Animation::fade_in("node-1", 0.5);
    let move_to = Animation::move_to("node-1", [0.0, 0.0], [200.0, 300.0], 1.0);
    let create = Animation::create("node-2", 0.8);

    println!(
        "  FadeIn: {:.1}s, easing={:?}",
        fade_in.duration_secs, fade_in.easing
    );
    println!("  MoveTo: {:.1}s to [200,300]", move_to.duration_secs);
    println!(
        "  Create: {:.1}s, easing={:?}",
        create.duration_secs, create.easing
    );

    let seq = Sequence::Sequential(vec![fade_in.clone(), move_to.clone()]);
    let par = Sequence::Parallel(vec![fade_in.clone(), create]);
    println!("  Sequential(FadeIn+MoveTo): {:.1}s", seq.total_duration());
    println!("  Parallel(FadeIn+Create): {:.1}s", par.total_duration());

    let easings = [
        Easing::Linear,
        Easing::EaseIn,
        Easing::EaseOut,
        Easing::EaseInOut,
        Easing::Spring,
        Easing::Bounce,
    ];
    print!("  Easing at t=0.5:");
    for e in &easings {
        print!(" {:?}={:.3}", e, e.apply(0.5));
    }
    println!();

    let mut state = AnimationState::new(fade_in);
    state.advance(0.25);
    println!(
        "  AnimationState: progress={:.2}, done={}",
        state.progress(),
        state.is_done()
    );

    println!("  RESULT: animation system OK");
    println!();
}

fn demo_svg() {
    println!("=== SVG Modality Compiler ===");

    let expr = GrammarExpr::new("temperature", GeometryType::Point)
        .with_x("day")
        .with_y("temp_c")
        .with_title("Weekly Temperature");

    let data = json_array(&serde_json::json!([
        {"day": 1, "temp_c": 18},
        {"day": 2, "temp_c": 22},
        {"day": 3, "temp_c": 20},
        {"day": 4, "temp_c": 25},
        {"day": 5, "temp_c": 23},
        {"day": 6, "temp_c": 19},
        {"day": 7, "temp_c": 21}
    ]));

    let compiler = GrammarCompiler;
    let scene = compiler.compile(&expr, &data);

    let svg_compiler = SvgCompiler;
    let output = svg_compiler.compile(&scene);
    let svg_str = match &output {
        petal_tongue_scene::modality::ModalityOutput::Svg(b) => {
            std::str::from_utf8(b.as_ref()).unwrap_or_default()
        }
        _ => panic!("Expected SVG output"),
    };

    println!("  SVG output: {} bytes", svg_str.len());
    println!("  Starts with <svg: {}", svg_str.starts_with("<svg"));
    println!("  Ends with </svg>: {}", svg_str.ends_with("</svg>"));
    println!("  Compiler name: {}", svg_compiler.name());

    let output_path =
        std::env::var("DEMO_OUTPUT_DIR").unwrap_or_else(|_| "/tmp/petaltongue-showcase".into());
    std::fs::create_dir_all(&output_path).ok();
    let svg_path = format!("{output_path}/weekly_temperature.svg");
    std::fs::write(&svg_path, svg_str).ok();
    println!("  Written to: {svg_path}");
    println!("  RESULT: SVG modality OK");
    println!();
}

fn demo_audio() {
    println!("=== Audio Modality Compiler ===");

    let mut graph = SceneGraph::new();
    for i in 0..5 {
        let x = f64::from(i) * 100.0 + 50.0;
        let y = f64::from(i) * 50.0 + 100.0;
        graph.add_to_root(
            SceneNode::new(format!("audio-{i}")).with_primitive(Primitive::Point {
                x,
                y,
                radius: 5.0,
                fill: Some(Color::rgba(0.3, 0.7, 0.5, 1.0)),
                stroke: None,
                data_id: Some(format!("note-{i}")),
            }),
        );
    }

    let audio_compiler = AudioCompiler;
    let output = audio_compiler.compile(&graph);
    let petal_tongue_scene::modality::ModalityOutput::AudioParams(params) = &output else {
        panic!("Expected AudioParams output")
    };

    println!("  Audio parameters: {} sonification points", params.len());
    for p in params {
        println!(
            "    freq={:.0}Hz pan={:.2} amp={:.2} dur={:.2}s",
            p.frequency, p.pan, p.amplitude, p.duration_secs
        );
    }
    println!("  Compiler name: {}", audio_compiler.name());
    println!("  RESULT: audio modality OK");
    println!();
}

fn demo_accessibility() {
    println!("=== Accessibility / Description Compiler ===");

    let expr = GrammarExpr::new("sensor_data", GeometryType::Bar)
        .with_x("sensor")
        .with_y("reading")
        .with_title("Sensor Readings");

    let data = json_array(&serde_json::json!([
        {"sensor": 0, "reading": 45},
        {"sensor": 1, "reading": 78},
        {"sensor": 2, "reading": 32}
    ]));

    let compiler = GrammarCompiler;
    let scene = compiler.compile(&expr, &data);

    let desc_compiler = DescriptionCompiler;
    let output = desc_compiler.compile(&scene);
    let text = match &output {
        petal_tongue_scene::modality::ModalityOutput::Description(b) => {
            std::str::from_utf8(b.as_ref()).unwrap_or_default()
        }
        _ => panic!("Expected Description output"),
    };

    println!("  Description: {} chars", text.len());
    for line in text.lines().take(8) {
        println!("    {line}");
    }
    println!("  Compiler name: {}", desc_compiler.name());
    println!("  RESULT: accessibility OK");
    println!();
}

fn demo_physics() {
    println!("=== Physics Bridge (barraCuda IPC) ===");

    let mut world = PhysicsWorld::new();
    world.add_body(PhysicsBody {
        id: "planet-a".into(),
        mass: 1e6,
        position: [0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
        collision_shape: CollisionShape::Sphere { radius: 50.0 },
    });
    world.add_body(PhysicsBody {
        id: "planet-b".into(),
        mass: 1.0,
        position: [200.0, 0.0, 0.0],
        velocity: [0.0, 50.0, 0.0],
        collision_shape: CollisionShape::Sphere { radius: 5.0 },
    });

    println!("  Bodies: {}", world.bodies.len());
    println!("  Gravity: {:?}", world.gravity);
    println!("  Time step: {:.4}s", world.time_step);

    let ipc_json = world.to_ipc_request();
    let ipc_str = serde_json::to_string_pretty(&ipc_json).unwrap_or_default();
    println!("  IPC request (math.physics.nbody):");
    for line in ipc_str.lines().take(10) {
        println!("    {line}");
    }
    if ipc_str.lines().count() > 10 {
        println!("    ...");
    }

    world.step_euler();
    let b = &world.bodies[1];
    println!(
        "  After CPU Euler step: planet-b pos=({:.1}, {:.1}, {:.1})",
        b.position[0], b.position[1], b.position[2]
    );

    let mock_response = serde_json::json!({
        "bodies": [
            {"id": "planet-a", "position": [0.0, 0.0, 0.0], "velocity": [0.0, 0.0, 0.0]},
            {"id": "planet-b", "position": [199.5, 0.8, 0.0], "velocity": [-0.5, 50.0, 0.0]}
        ]
    });
    world.apply_ipc_response(&mock_response);
    let b = &world.bodies[1];
    println!(
        "  After IPC response: planet-b pos=({:.1}, {:.1}, {:.1})",
        b.position[0], b.position[1], b.position[2]
    );

    println!("  RESULT: physics bridge OK");
    println!();
}
