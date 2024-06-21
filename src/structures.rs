use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone, Debug)]
pub enum StructureGroup {
    Base,
    Energy,
    Mine,
    Storage,
    Factory,
}

impl Display for StructureGroup {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct EnergyComponent {
    pub energy_in: u64,
    pub energy_out: u64,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct ResourceComponent {
    pub resource_in: u64,
    pub resource_out: u64,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum ComponentName {
    EnergyComponent,
    ResourceComponent,
}

impl Display for ComponentName {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum ComponentGroup {
    Energy { component: EnergyComponent },
    Resource { component: ResourceComponent },
}

impl Display for ComponentGroup {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            ComponentGroup::Energy { .. } => ComponentName::EnergyComponent,
            ComponentGroup::Resource { .. } => ComponentName::ResourceComponent,
        };

        write!(f, "{:?}", name)
    }
}

pub trait EnergyTrait {
    fn energy_in(&self) -> u64;
    fn energy_out(&self) -> u64;
}

pub trait ResourceTrait {
    fn resource_in(&self) -> u64;
    fn resource_out(&self) -> u64;
}

#[derive(Clone, Debug)]
pub enum Structure {
    PowerPlant { structure: PowerPlant },
    Mine { structure: Mine },
}

impl Display for Structure {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            Structure::PowerPlant { .. } => "Power Plant",
            Structure::Mine { .. } => "Mine",
        };
        write!(f, "{:?}", name)
    }
}

pub trait StructureGroupTrait {
    fn group(&self) -> StructureGroup;
}

impl StructureGroupTrait for Structure {
    fn group(&self) -> StructureGroup {
        match self {
            Structure::PowerPlant { .. } => StructureGroup::Energy,
            Structure::Mine { .. } => StructureGroup::Mine,
        }
    }
}

#[derive(Clone)]
pub struct StructureBlueprint {
    components: HashMap<ComponentName, ComponentGroup>,
}

impl StructureBlueprint {
    pub fn add_component(&mut self, name: ComponentName, component: ComponentGroup) {
        self.components.insert(name, component);
    }

    fn get_component(&self, name: &ComponentName) -> &ComponentGroup {
        let component = self.components.get(name);

        if component.is_some() {
            return component.unwrap();
        } else {
            panic!(
                "{} is missing in this structure",
                ComponentName::EnergyComponent
            )
        }
    }
}

impl EnergyTrait for StructureBlueprint {
    fn energy_in(&self) -> u64 {
        match self.get_component(&ComponentName::EnergyComponent) {
            ComponentGroup::Energy {
                component: EnergyComponent { energy_in, .. },
            } => *energy_in,
            _ => 0,
        }
    }

    fn energy_out(&self) -> u64 {
        match self.get_component(&ComponentName::EnergyComponent) {
            ComponentGroup::Energy {
                component: EnergyComponent { energy_out, .. },
            } => *energy_out,
            _ => 0,
        }
    }
}

impl ResourceTrait for StructureBlueprint {
    fn resource_in(&self) -> u64 {
        match self.get_component(&ComponentName::ResourceComponent) {
            ComponentGroup::Resource {
                component: ResourceComponent { resource_in, .. },
            } => *resource_in,
            _ => 0,
        }
    }

    fn resource_out(&self) -> u64 {
        match self.get_component(&ComponentName::ResourceComponent) {
            ComponentGroup::Resource {
                component: ResourceComponent { resource_out, .. },
            } => *resource_out,
            _ => 0,
        }
    }
}

// PowerPlant
#[derive(Clone)]
pub struct PowerPlant {
    blueprint: StructureBlueprint,
}

impl Debug for PowerPlant {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl PowerPlant {
    pub fn new() -> PowerPlant {
        let energy_component = ComponentGroup::Energy {
            component: EnergyComponent {
                energy_out: 100,
                energy_in: 0,
            },
        };

        let mut components = HashMap::new();
        components.insert(ComponentName::EnergyComponent, energy_component);

        let blueprint = StructureBlueprint { components };

        return PowerPlant { blueprint };
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        return &self.blueprint;
    }
}

// Mine
#[derive(Clone)]
pub struct Mine {
    blueprint: StructureBlueprint,
}

impl Debug for Mine {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Mine {
    pub fn new() -> Mine {
        let energy_component = ComponentGroup::Energy {
            component: EnergyComponent {
                energy_out: 0,
                energy_in: 25,
            },
        };

        let resource_component = ComponentGroup::Resource {
            component: ResourceComponent {
                resource_out: 5,
                resource_in: 0,
            },
        };

        let mut components = HashMap::new();

        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::ResourceComponent, resource_component);

        let blueprint = StructureBlueprint { components };

        return Mine { blueprint };
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        return &self.blueprint;
    }
}
