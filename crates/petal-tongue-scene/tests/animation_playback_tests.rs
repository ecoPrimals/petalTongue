// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Animation playback, scene graph interaction, and serialization tests.
//!
//! Split from `animation_tests.rs` which covers easing, state, and construction.

use petal_tongue_scene::animation::AnimationTarget;
use petal_tongue_scene::animation::{Animation, AnimationPlayer, Easing, Sequence};
use petal_tongue_scene::scene_graph::{SceneGraph, SceneNode};

// ── AnimationPlayer lifecycle ──────────────────────────────────────────────

#[test]
fn animation_player_plays_and_completes() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("n1").with_opacity(0.0));

    let mut player = AnimationPlayer::new();
    assert!(!player.is_playing());

    player.play(Animation::fade_in("n1", 1.0));
    assert!(player.is_playing());
    assert_eq!(player.active_count(), 1);

    let remaining = player.tick(0.5, &mut scene);
    assert_eq!(remaining, 1);
    let opacity = scene.get("n1").expect("node n1 should exist").opacity;
    assert!(
        opacity > 0.0 && opacity < 1.0,
        "half-way opacity should be between 0 and 1"
    );

    let remaining = player.tick(1.0, &mut scene);
    assert_eq!(remaining, 0);
    assert!(!player.is_playing());
}

#[test]
fn animation_player_tick_translates_node() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("n1"));

    let mut player = AnimationPlayer::new();
    player.play(Animation::move_to("n1", [0.0, 0.0], [100.0, 200.0], 1.0));
    player.tick(1.0, &mut scene);
    let node = scene.get("n1").expect("node n1 should exist");
    let (x, y) = node.transform.apply(0.0, 0.0);
    assert!((x - 100.0).abs() < 1.0);
    assert!((y - 200.0).abs() < 1.0);
}

#[test]
fn animation_player_parallel_animations() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("n1").with_opacity(0.0));
    scene.add_to_root(SceneNode::new("n2").with_opacity(0.0));

    let seq = Sequence::Parallel(vec![
        Animation::fade_in("n1", 1.0),
        Animation::fade_in("n2", 1.0),
    ]);
    let mut player = AnimationPlayer::new();
    player.play_sequence(seq);
    assert_eq!(player.active_count(), 2);
    player.tick(1.0, &mut scene);
    assert_eq!(player.active_count(), 0);
    assert!(
        (scene.get("n1").expect("n1 exists").opacity - 1.0).abs() < 0.01,
        "n1 should be fully opaque"
    );
    assert!(
        (scene.get("n2").expect("n2 exists").opacity - 1.0).abs() < 0.01,
        "n2 should be fully opaque"
    );
}

#[test]
fn animation_player_play_sequence_sequential() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("n1"));
    scene.add_to_root(SceneNode::new("n2"));

    let seq = Sequence::Sequential(vec![
        Animation::fade_in("n1", 0.5),
        Animation::fade_in("n2", 0.5),
    ]);
    let mut player = AnimationPlayer::new();
    player.play_sequence(seq);
    assert_eq!(player.active_count(), 2);
}

#[test]
fn animation_player_sequence_group_in_tick_path() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("n1").with_opacity(0.0));
    scene.add_to_root(SceneNode::new("n2").with_opacity(0.0));

    let a1 = Animation::fade_in("n1", 0.5);
    let a2 = Animation::fade_in("n2", 0.5);
    let inner = Sequence::Sequential(vec![a1, a2]);
    let group = Sequence::Group(vec![inner.clone(), inner]);

    let mut player = AnimationPlayer::new();
    player.play_sequence(group);
    assert_eq!(player.active_count(), 4);

    let remaining = player.tick(2.0, &mut scene);
    assert_eq!(remaining, 0);
    assert!(!player.is_playing());
    assert!(
        (scene.get("n1").expect("n1 exists").opacity - 1.0).abs() < 0.01,
        "n1 should be fully opaque"
    );
    assert!(
        (scene.get("n2").expect("n2 exists").opacity - 1.0).abs() < 0.01,
        "n2 should be fully opaque"
    );
}

#[test]
fn animation_player_empty_tick() {
    let mut scene = SceneGraph::new();
    let mut player = AnimationPlayer::new();
    let remaining = player.tick(1.0, &mut scene);
    assert_eq!(remaining, 0);
}

#[test]
fn animation_player_default_equals_new() {
    let defaulted = AnimationPlayer::default();
    let created = AnimationPlayer::new();
    assert_eq!(defaulted.active_count(), created.active_count());
    assert!(!defaulted.is_playing());
}

#[test]
fn animation_player_clone() {
    let mut player = AnimationPlayer::new();
    player.play(Animation::fade_in("n", 1.0));
    let cloned = player.clone();
    assert_eq!(cloned.active_count(), 1);
}

// ── Scene graph interaction ────────────────────────────────────────────────

#[test]
fn animation_scale_with_negative_from() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("n1"));

    let anim = Animation {
        target: AnimationTarget::Scale {
            node_id: "n1".to_string(),
            from: -1.0,
            to: 1.0,
        },
        duration_secs: 1.0,
        easing: Easing::Linear,
        delay_secs: 0.0,
    };

    let mut player = AnimationPlayer::new();
    player.play(anim);
    player.tick(1.0, &mut scene);

    let node = scene.get("n1").expect("node n1 should exist");
    let (x, _) = node.transform.apply(1.0, 0.0);
    assert!(
        (x - 1.0).abs() < 0.01,
        "Scale should interpolate from -1 to 1"
    );
}

#[test]
fn animation_rotation_interpolation() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("n1"));

    let anim = Animation {
        target: AnimationTarget::Rotation {
            node_id: "n1".to_string(),
            from: 0.0,
            to: std::f64::consts::PI,
        },
        duration_secs: 1.0,
        easing: Easing::Linear,
        delay_secs: 0.0,
    };

    let mut player = AnimationPlayer::new();
    player.play(anim);
    player.tick(1.0, &mut scene);
    let node = scene.get("n1").expect("node n1 should exist");
    let (x, _y) = node.transform.apply(1.0, 0.0);
    assert!((x - (-1.0)).abs() < 0.1, "PI rotation should flip x");
}

#[test]
fn animation_stroke_draw_progress() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("n1").with_opacity(0.0));

    let anim = Animation::create("n1", 1.0);
    let mut player = AnimationPlayer::new();
    player.play(anim);
    player.tick(0.5, &mut scene);
    let opacity = scene.get("n1").expect("n1 exists").opacity;
    assert!(
        opacity > 0.0 && opacity < 1.0,
        "mid-animation opacity should be between 0 and 1"
    );
    player.tick(1.0, &mut scene);
    assert!(
        (scene.get("n1").expect("n1 exists").opacity - 1.0).abs() < 0.01,
        "stroke draw should complete to full opacity"
    );
}

#[test]
fn animation_target_missing_node_no_panic() {
    let mut scene = SceneGraph::new();
    scene.add_to_root(SceneNode::new("n1"));

    let anim = Animation::fade_in("nonexistent", 1.0);
    let mut player = AnimationPlayer::new();
    player.play(anim);
    let remaining = player.tick(1.0, &mut scene);
    assert_eq!(remaining, 0);
}

// ── Serialization roundtrips ───────────────────────────────────────────────

const EPS: f64 = 1e-10;

#[test]
fn animation_serialization_roundtrip() {
    let anim = Animation::fade_in("node1", 2.5)
        .with_easing(Easing::Spring)
        .with_delay(0.5);
    let json = serde_json::to_string(&anim).expect("serialization should succeed");
    let decoded: Animation = serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(anim.target, decoded.target);
    assert!((anim.duration_secs - decoded.duration_secs).abs() < EPS);
    assert_eq!(anim.easing, decoded.easing);
    assert!((anim.delay_secs - decoded.delay_secs).abs() < EPS);
}

#[test]
fn animation_target_serialization_all_variants() {
    let targets = [
        AnimationTarget::Opacity {
            node_id: "n".to_string(),
            from: 0.0,
            to: 1.0,
        },
        AnimationTarget::Translate {
            node_id: "n".to_string(),
            from: [0.0, 0.0],
            to: [10.0, 20.0],
        },
        AnimationTarget::Scale {
            node_id: "n".to_string(),
            from: 1.0,
            to: 2.0,
        },
        AnimationTarget::Rotation {
            node_id: "n".to_string(),
            from: 0.0,
            to: std::f64::consts::PI,
        },
        AnimationTarget::StrokeDraw {
            node_id: "n".to_string(),
        },
    ];
    for target in targets {
        let json = serde_json::to_string(&target).expect("serialization should succeed");
        let decoded: AnimationTarget =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(target, decoded, "roundtrip for {target:?}");
    }
}

#[test]
fn easing_serialization_roundtrip() {
    for easing in [
        Easing::Linear,
        Easing::EaseIn,
        Easing::EaseOut,
        Easing::EaseInOut,
        Easing::Spring,
        Easing::Bounce,
    ] {
        let json = serde_json::to_string(&easing).expect("serialization should succeed");
        let decoded: Easing = serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(easing, decoded);
    }
}

#[test]
fn sequence_serialization_roundtrip() {
    let seq = Sequence::Sequential(vec![
        Animation::fade_in("n1", 1.0),
        Animation::fade_out("n2", 2.0),
    ]);
    let json = serde_json::to_string(&seq).expect("serialization should succeed");
    let decoded: Sequence = serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(seq, decoded);
}

#[test]
fn sequence_parallel_serialization_roundtrip() {
    let seq = Sequence::Parallel(vec![Animation::create("n", 1.0)]);
    let json = serde_json::to_string(&seq).expect("serialization should succeed");
    let decoded: Sequence = serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(seq, decoded);
}

#[test]
fn sequence_group_serialization_roundtrip() {
    let inner = Sequence::Sequential(vec![Animation::fade_in("n", 0.5)]);
    let seq = Sequence::Group(vec![inner]);
    let json = serde_json::to_string(&seq).expect("serialization should succeed");
    let decoded: Sequence = serde_json::from_str(&json).expect("deserialization should succeed");
    assert_eq!(seq, decoded);
}

// ── Trait implementations ──────────────────────────────────────────────────

#[test]
fn animation_target_debug_formatting() {
    let target = AnimationTarget::Opacity {
        node_id: "n".to_string(),
        from: 0.0,
        to: 1.0,
    };
    let s = format!("{target:?}");
    assert!(s.contains("Opacity"));
    assert!(s.contains('n'));
}

#[test]
fn animation_target_partial_eq() {
    let a = AnimationTarget::StrokeDraw {
        node_id: "x".to_string(),
    };
    let b = AnimationTarget::StrokeDraw {
        node_id: "x".to_string(),
    };
    let c = AnimationTarget::StrokeDraw {
        node_id: "y".to_string(),
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}
