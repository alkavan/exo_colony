// SPDX-FileCopyrightText: Copyright (c) 2021-2026 Igal Alkon
// SPDX-License-Identifier: Zlib

//! Player-facing supply chain.
//!
//! Recipes and build inputs live here so the panel, construction costs, and
//! structure blueprints share one table. Milestones are not placeable yet:
//! Spaceport is the first goal and unlocks the Intergalactic Market.

use std::fmt::{Display, Formatter, Result};

use crate::game::{Commodity, Manufactured, Resource};
use crate::structures::StructureGroup;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Good {
    Resource(Resource),
    Manufactured(Manufactured),
    Commodity(Commodity),
}

impl Display for Good {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Good::Resource(item) => write!(f, "{}", item),
            Good::Manufactured(item) => write!(f, "{}", item),
            Good::Commodity(item) => write!(f, "{}", item),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ingredient {
    pub good: Good,
    pub amount: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerKind {
    Mine,
    Refinery,
    Factory,
}

#[derive(Clone, Copy, Debug)]
pub struct Recipe {
    pub id: &'static str,
    pub producer: ProducerKind,
    pub output: Good,
    pub amount: u64,
    pub energy: u64,
    pub inputs: &'static [Ingredient],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MilestoneId {
    Spaceport,
    IntergalacticMarket,
    GalacticSuperhighway,
}

#[derive(Clone, Copy, Debug)]
pub struct Milestone {
    pub id: MilestoneId,
    pub name: &'static str,
    /// What completing this milestone opens. Empty if it is a terminal unlock.
    pub unlocks: &'static str,
    pub after: Option<MilestoneId>,
    pub placeable: bool,
}

const MINE_GRAVEL: Ingredient = Ingredient {
    good: Good::Manufactured(Manufactured::Gravel),
    amount: 1,
};

const RECIPES: &[Recipe] = &[
    Recipe {
        id: "silicon",
        producer: ProducerKind::Refinery,
        output: Good::Manufactured(Manufactured::Silicon),
        amount: 1,
        energy: 30,
        inputs: &[Ingredient {
            good: Good::Resource(Resource::Silica),
            amount: 4,
        }],
    },
    Recipe {
        id: "steel",
        producer: ProducerKind::Refinery,
        output: Good::Manufactured(Manufactured::Steel),
        amount: 1,
        energy: 25,
        inputs: &[Ingredient {
            good: Good::Resource(Resource::Iron),
            amount: 3,
        }],
    },
    Recipe {
        id: "hydrogen",
        producer: ProducerKind::Refinery,
        output: Good::Manufactured(Manufactured::Hydrogen),
        amount: 1,
        energy: 20,
        inputs: &[Ingredient {
            good: Good::Resource(Resource::Water),
            amount: 6,
        }],
    },
    Recipe {
        id: "fuel-pellet",
        producer: ProducerKind::Refinery,
        output: Good::Manufactured(Manufactured::FuelPellet),
        amount: 1,
        energy: 50,
        inputs: &[Ingredient {
            good: Good::Resource(Resource::Uranium),
            amount: 2,
        }],
    },
    Recipe {
        id: "concrete",
        producer: ProducerKind::Factory,
        output: Good::Commodity(Commodity::Concrete),
        amount: 1,
        energy: 25,
        inputs: &[
            Ingredient {
                good: Good::Resource(Resource::Silica),
                amount: 5,
            },
            Ingredient {
                good: Good::Manufactured(Manufactured::Gravel),
                amount: 5,
            },
        ],
    },
    Recipe {
        id: "fuel",
        producer: ProducerKind::Factory,
        output: Good::Commodity(Commodity::Fuel),
        amount: 1,
        energy: 20,
        inputs: &[Ingredient {
            good: Good::Manufactured(Manufactured::Hydrogen),
            amount: 2,
        }],
    },
    Recipe {
        id: "semiconductor",
        producer: ProducerKind::Factory,
        output: Good::Commodity(Commodity::Semiconductor),
        amount: 1,
        energy: 40,
        inputs: &[
            Ingredient {
                good: Good::Resource(Resource::Aluminum),
                amount: 3,
            },
            Ingredient {
                good: Good::Resource(Resource::Carbon),
                amount: 4,
            },
            Ingredient {
                good: Good::Manufactured(Manufactured::Silicon),
                amount: 2,
            },
        ],
    },
    Recipe {
        id: "glass",
        producer: ProducerKind::Factory,
        output: Good::Commodity(Commodity::Glass),
        amount: 1,
        energy: 35,
        inputs: &[
            Ingredient {
                good: Good::Resource(Resource::Silica),
                amount: 6,
            },
            Ingredient {
                good: Good::Manufactured(Manufactured::Silicon),
                amount: 1,
            },
        ],
    },
    Recipe {
        id: "fuel-rod",
        producer: ProducerKind::Factory,
        output: Good::Commodity(Commodity::FuelRod),
        amount: 1,
        energy: 60,
        inputs: &[
            Ingredient {
                good: Good::Manufactured(Manufactured::FuelPellet),
                amount: 1,
            },
            Ingredient {
                good: Good::Manufactured(Manufactured::Steel),
                amount: 1,
            },
        ],
    },
];

const MILESTONES: &[Milestone] = &[
    Milestone {
        id: MilestoneId::Spaceport,
        name: "Spaceport",
        unlocks: "Intergalactic Market",
        after: None,
        placeable: false,
    },
    Milestone {
        id: MilestoneId::IntergalacticMarket,
        name: "Intergalactic Market",
        unlocks: "Galactic Economy",
        after: Some(MilestoneId::Spaceport),
        placeable: false,
    },
    Milestone {
        id: MilestoneId::GalacticSuperhighway,
        name: "Galactic Superhighway",
        unlocks: "travel between galaxies (infinite mode)",
        after: Some(MilestoneId::IntergalacticMarket),
        placeable: false,
    },
];

struct BuildSpec {
    role: &'static str,
    resources: &'static [(Resource, u64)],
    manufactured: &'static [(Manufactured, u64)],
    commodities: &'static [(Commodity, u64)],
}

fn build_spec(group: &StructureGroup) -> BuildSpec {
    match group {
        StructureGroup::Base => BuildSpec {
            role: "foothold: 50 energy, battery, starter bins",
            resources: &[(Resource::Iron, 15)],
            manufactured: &[],
            commodities: &[(Commodity::Concrete, 5)],
        },
        StructureGroup::Power => BuildSpec {
            role: "120 energy",
            resources: &[(Resource::Silica, 5)],
            manufactured: &[(Manufactured::Steel, 5)],
            commodities: &[],
        },
        StructureGroup::Mine => BuildSpec {
            role: "extracts the vein under the cursor (+1 gravel)",
            resources: &[(Resource::Iron, 8)],
            manufactured: &[],
            commodities: &[],
        },
        StructureGroup::Refinery => BuildSpec {
            role: "raw ore into manufactured goods",
            resources: &[(Resource::Iron, 8)],
            manufactured: &[(Manufactured::Steel, 8)],
            commodities: &[],
        },
        StructureGroup::Factory => BuildSpec {
            role: "manufactured goods into commodities",
            resources: &[(Resource::Iron, 8)],
            manufactured: &[(Manufactured::Steel, 5)],
            commodities: &[(Commodity::Concrete, 5)],
        },
        StructureGroup::Storage => BuildSpec {
            role: "parks raw and commodities out of the loose pile",
            resources: &[(Resource::Iron, 8)],
            manufactured: &[],
            commodities: &[(Commodity::Concrete, 5)],
        },
    }
}

pub struct Catalog;

impl Catalog {
    pub fn recipes() -> &'static [Recipe] {
        RECIPES
    }

    pub fn milestones() -> &'static [Milestone] {
        MILESTONES
    }

    /// First goal. Later milestones stay locked until this is placeable and built.
    pub fn current_goal() -> &'static Milestone {
        &MILESTONES[0]
    }

    pub fn build_parts(
        group: &StructureGroup,
    ) -> (
        &'static [(Resource, u64)],
        &'static [(Manufactured, u64)],
        &'static [(Commodity, u64)],
    ) {
        let spec = build_spec(group);
        (spec.resources, spec.manufactured, spec.commodities)
    }

    pub fn refine(output: &Manufactured) -> Option<&'static Recipe> {
        RECIPES.iter().find(|recipe| {
            recipe.producer == ProducerKind::Refinery
                && recipe.output == Good::Manufactured(*output)
        })
    }

    pub fn manufacture(output: &Commodity) -> Option<&'static Recipe> {
        RECIPES.iter().find(|recipe| {
            recipe.producer == ProducerKind::Factory && recipe.output == Good::Commodity(*output)
        })
    }

    pub fn consumers_of(good: Good) -> Vec<&'static Recipe> {
        RECIPES
            .iter()
            .filter(|recipe| recipe.inputs.iter().any(|input| input.good == good))
            .collect()
    }

    pub fn panel_lines(
        group: &StructureGroup,
        mine: Option<Resource>,
        refinery: &[Manufactured],
        factory: Option<Commodity>,
    ) -> Vec<String> {
        let spec = build_spec(group);
        let mut lines = vec![
            format!("{}", group),
            spec.role.to_string(),
            format!("Build: {}", describe_build(group)),
        ];

        match group {
            StructureGroup::Mine => {
                if let Some(resource) = mine {
                    lines.push(format!("Vein: +1 {} +1 Gravel", resource));
                    push_feeds(&mut lines, Good::Resource(resource));
                    push_feeds(&mut lines, MINE_GRAVEL.good);
                } else {
                    lines.push("Place on a colored deposit. Mines that vein.".to_string());
                }
            }
            StructureGroup::Refinery => {
                if refinery.is_empty() {
                    lines.push("Select a manufactured good.".to_string());
                }
                for output in refinery {
                    if let Some(recipe) = Catalog::refine(output) {
                        lines.push(format_recipe(recipe));
                        push_feeds(&mut lines, recipe.output);
                    }
                }
            }
            StructureGroup::Factory => {
                if let Some(output) = factory {
                    if let Some(recipe) = Catalog::manufacture(&output) {
                        lines.push(format_recipe(recipe));
                        push_feeds(&mut lines, recipe.output);
                    }
                }
            }
            _ => {}
        }

        let goal = Catalog::current_goal();
        lines.push(format!(
            "Goal: {} unlocks {}",
            goal.name, goal.unlocks
        ));
        if let Some(market) = MILESTONES
            .iter()
            .find(|item| item.id == MilestoneId::IntergalacticMarket)
        {
            lines.push(format!("{} enables {}", market.name, market.unlocks));
        }
        if let Some(highway) = MILESTONES
            .iter()
            .find(|item| item.id == MilestoneId::GalacticSuperhighway)
        {
            lines.push(format!("{}: {}", highway.name, highway.unlocks));
        }
        lines
    }
}

fn describe_build(group: &StructureGroup) -> String {
    let (resources, manufactured, commodities) = Catalog::build_parts(group);
    let mut parts = Vec::new();
    for (item, amount) in resources {
        parts.push(format!("{} {}", amount, item));
    }
    for (item, amount) in manufactured {
        parts.push(format!("{} {}", amount, item));
    }
    for (item, amount) in commodities {
        parts.push(format!("{} {}", amount, item));
    }
    if parts.is_empty() {
        "free".to_string()
    } else {
        parts.join(", ")
    }
}

fn format_recipe(recipe: &Recipe) -> String {
    let needs = if recipe.inputs.is_empty() {
        "nothing".to_string()
    } else {
        recipe
            .inputs
            .iter()
            .map(|input| format!("{} {}", input.amount, input.good))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "+{} {} needs {} ({} energy)",
        recipe.amount, recipe.output, needs, recipe.energy
    )
}

fn push_feeds(lines: &mut Vec<String>, good: Good) {
    let consumers = Catalog::consumers_of(good);
    if consumers.is_empty() {
        return;
    }
    let names = consumers
        .iter()
        .map(|recipe| recipe.output.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    lines.push(format!("{} feeds {}", good, names));
}

pub fn resource_inputs(recipe: &Recipe) -> Vec<(Resource, u64)> {
    recipe
        .inputs
        .iter()
        .filter_map(|input| match input.good {
            Good::Resource(resource) => Some((resource, input.amount)),
            _ => None,
        })
        .collect()
}

pub fn manufactured_inputs(recipe: &Recipe) -> Vec<(Manufactured, u64)> {
    recipe
        .inputs
        .iter()
        .filter_map(|input| match input.good {
            Good::Manufactured(item) => Some((item, input.amount)),
            _ => None,
        })
        .collect()
}
