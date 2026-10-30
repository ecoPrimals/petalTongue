// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gaia taxonomy types — species, primals, and entity classification.
//!
//! petalTongue IS the UI primal: any data → any representation.
//! These types let Gaia create new species and classify entities
//! through the universal compiler.
//!
//! ```text
//! Entity (raw data)
//!     → EntityKind (classification)
//!     → Species (taxonomy position)
//!     → Primal (which of the 16 primals governs rendering)
//!     → petalTongue.render() (any modality output)
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The 16 primals — Gaia's taxonomy roots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Primal {
    ToadStool,
    BiomeOS,
    NestGate,
    BearDog,
    PetalTongue,
    RhizoCrypt,
    SweetGrass,
    LoamSpine,
    SkunkBat,
    SourDough,
    SwarmVine,
    BingoCube,
    CoralReef,
    BarraCuda,
    Squirrel,
    SongBird,
}

impl Primal {
    /// Human-readable role description.
    #[must_use]
    pub const fn role(self) -> &'static str {
        match self {
            Self::ToadStool => "THE TIME — temporal/night/sine",
            Self::BiomeOS => "THE OS — full operating system",
            Self::NestGate => "THE GATE — HUD/membrane/interaction",
            Self::BearDog => "THE GUARD — arena/impact/protection",
            Self::PetalTongue => "THE SENSE — epitope/ears/taste · universal compiler",
            Self::RhizoCrypt => "THE CRYPT — signal/hook/keys",
            Self::SweetGrass => "THE GATHERING — campfire/berry",
            Self::LoamSpine => "THE SPINE — nanowire/dag/hologram",
            Self::SkunkBat => "THE SHADOW — klein/null/shadow",
            Self::SourDough => "THE CULTURE — nucleation/helix/ferment",
            Self::SwarmVine => "THE SWARM — mesh/presence/gossip",
            Self::BingoCube => "THE MATH — periodic table/bingo/sort",
            Self::CoralReef => "THE DEPTH — cosmic/deep/3D",
            Self::BarraCuda => "THE SPEED — search/proton/gokart",
            Self::Squirrel => "THE OBSERVER — chipmunk/mind/commands",
            Self::SongBird => "THE VOICE — scatter/fleet/chitter",
        }
    }

    /// All 16 primals.
    #[must_use]
    pub const fn all() -> [Self; 16] {
        [
            Self::ToadStool, Self::BiomeOS, Self::NestGate, Self::BearDog,
            Self::PetalTongue, Self::RhizoCrypt, Self::SweetGrass, Self::LoamSpine,
            Self::SkunkBat, Self::SourDough, Self::SwarmVine, Self::BingoCube,
            Self::CoralReef, Self::BarraCuda, Self::Squirrel, Self::SongBird,
        ]
    }
}

/// Entity kind — classification within the taxonomy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Person,
    Organization,
    Analysis,
    Coverage,
    Guide,
    Evidence,
    Letter,
    Timeline,
    Document,
    /// Custom kind — Gaia can create new species.
    Species(String),
}

impl EntityKind {
    /// Which primal governs this kind's rendering.
    #[must_use]
    pub fn governing_primal(&self) -> Primal {
        match self {
            Self::Person | Self::Organization => Primal::SwarmVine,
            Self::Analysis => Primal::BingoCube,
            Self::Coverage => Primal::SongBird,
            Self::Guide => Primal::SweetGrass,
            Self::Evidence => Primal::RhizoCrypt,
            Self::Letter => Primal::SongBird,
            Self::Timeline => Primal::ToadStool,
            Self::Document => Primal::PetalTongue,
            Self::Species(_) => Primal::PetalTongue,
        }
    }
}

/// A Gaia entity — the fundamental data unit for petalTongue rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub kind: EntityKind,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub url: String,
    #[serde(default)]
    pub excerpt: String,
    #[serde(default)]
    pub full_text: String,
    #[serde(default)]
    pub words: u64,
    #[serde(default)]
    pub internal_links: Vec<String>,
    #[serde(default)]
    pub page_refs: Vec<PageRef>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A reference from another page to this entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRef {
    pub url: String,
    pub title: String,
}

/// A relation between two entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub relation_type: String,
}

/// The complete entity taxonomy — Gaia's entity API data model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityTaxonomy {
    pub name: String,
    pub compiler: String,
    pub timestamp: String,
    #[serde(default)]
    pub axiom: String,
    pub entity_count: usize,
    pub relation_count: usize,
    #[serde(default)]
    pub kinds: HashMap<String, usize>,
    pub entities: HashMap<String, Entity>,
    #[serde(default)]
    pub relations: Vec<Relation>,
}

/// A species definition — Gaia can create new taxonomy branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeciesDefinition {
    pub name: String,
    pub primal: Primal,
    pub description: String,
    #[serde(default)]
    pub rendering_grammar: Option<String>,
    #[serde(default)]
    pub parent_species: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primal_all_returns_16() {
        assert_eq!(Primal::all().len(), 16);
    }

    #[test]
    fn primal_roles_not_empty() {
        for p in Primal::all() {
            assert!(!p.role().is_empty());
        }
    }

    #[test]
    fn entity_kind_governing_primal() {
        assert_eq!(EntityKind::Person.governing_primal(), Primal::SwarmVine);
        assert_eq!(EntityKind::Analysis.governing_primal(), Primal::BingoCube);
        assert_eq!(EntityKind::Document.governing_primal(), Primal::PetalTongue);
    }

    #[test]
    fn entity_serialization_round_trip() {
        let entity = Entity {
            id: "brian-banks".into(),
            kind: EntityKind::Person,
            title: "Brian Banks".into(),
            description: "Network actor".into(),
            keywords: vec!["detroit".into(), "charter".into()],
            url: "/publicRecord/actors/brian-banks/".into(),
            excerpt: "Board chair".into(),
            full_text: String::new(),
            words: 450,
            internal_links: vec!["/publicRecord/entities/purpose-charter-academy/".into()],
            page_refs: vec![PageRef {
                url: "/publicRecord/analysis/damages-calculator/".into(),
                title: "Damages Calculator".into(),
            }],
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&entity).unwrap();
        let back: Entity = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "brian-banks");
        assert_eq!(back.kind, EntityKind::Person);
    }

    #[test]
    fn custom_species() {
        let kind = EntityKind::Species("judicial-capture".into());
        assert_eq!(kind.governing_primal(), Primal::PetalTongue);
    }

    #[test]
    fn species_definition_serializes() {
        let spec = SpeciesDefinition {
            name: "judicial-capture".into(),
            primal: Primal::BingoCube,
            description: "Anderson localization of judicial oversight".into(),
            rendering_grammar: Some("lattice_graph".into()),
            parent_species: Some("analysis".into()),
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("judicial-capture"));
        assert!(json.contains("bingoCube"));
    }
}
