use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};

use crate::game::ResourceGroup;
use std::ops::AddAssign;

#[derive(Clone, Debug, Eq, PartialEq)]
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

pub trait EnergyTrait {
    fn energy_in(&self) -> u64;
    fn energy_out(&self) -> u64;
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct BatteryComponent {
    pub capacity: u64,
    pub used: u64,
}

pub trait BatteryTrait {
    fn capacity(&self) -> u64;
    fn used(&self) -> u64;
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct ResourceComponent {
    pub resource_in: u64,
    pub resource_out: u64,
}

pub trait ResourceTrait {
    fn resource_in(&self) -> u64;
    fn resource_out(&self) -> u64;
}

#[derive(Clone)]
pub struct StorageComponent {
    pub capacity: HashMap<ResourceGroup, u64>,
    pub resources: HashMap<ResourceGroup, u64>,
}

impl StorageComponent {
    fn new() -> StorageComponent {
        let mut capacity = HashMap::new();
        capacity.insert(ResourceGroup::Energy, 1000);
        capacity.insert(ResourceGroup::Metal, 1000);
        capacity.insert(ResourceGroup::Mineral, 1000);
        capacity.insert(ResourceGroup::Carbon, 1000);
        capacity.insert(ResourceGroup::Gas, 1000);

        let mut resources = HashMap::new();
        resources.insert(ResourceGroup::Energy, 0);
        resources.insert(ResourceGroup::Metal, 0);
        resources.insert(ResourceGroup::Mineral, 0);
        resources.insert(ResourceGroup::Carbon, 0);
        resources.insert(ResourceGroup::Gas, 0);

        return StorageComponent {
            capacity,
            resources,
        };
    }

    fn capacity(&self, group: &ResourceGroup) -> u64 {
        return self.capacity[&group];
    }

    fn capacity_free(&self, group: &ResourceGroup) -> u64 {
        return self.capacity[&group] - self.resources[&group];
    }

    fn resource(&self, group: &ResourceGroup) -> u64 {
        return self.resources[&group];
    }

    fn resource_mut(&mut self, group: &ResourceGroup) -> &mut u64 {
        return self.resources.get_mut(&group).unwrap();
    }

    fn resource_add(&mut self, group: &ResourceGroup, amount: u64) {
        self.resources.get_mut(group).unwrap().add_assign(amount);
    }
}

pub trait StorageTrait {
    fn capacity(&self, group: &ResourceGroup) -> u64;
    fn resource(&self, group: &ResourceGroup) -> u64;
    fn resource_add(&mut self, group: &ResourceGroup, amount: u64) -> u64;
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum ComponentName {
    EnergyComponent,
    ResourceComponent,
    StorageComponent,
    BatteryComponent,
}

impl Display for ComponentName {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
pub enum ComponentGroup {
    Energy { component: EnergyComponent },
    Resource { component: ResourceComponent },
    Storage { component: StorageComponent },
    Battery { component: BatteryComponent },
}

impl Display for ComponentGroup {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            ComponentGroup::Energy { .. } => ComponentName::EnergyComponent,
            ComponentGroup::Resource { .. } => ComponentName::ResourceComponent,
            ComponentGroup::Storage { .. } => ComponentName::StorageComponent,
            ComponentGroup::Battery { .. } => ComponentName::BatteryComponent,
        };

        write!(f, "{}", name)
    }
}

#[derive(Clone, Debug)]
pub enum Structure {
    Base { structure: Base },
    PowerPlant { structure: PowerPlant },
    Mine { structure: Mine },
    Storage { structure: Storage },
}

impl Display for Structure {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            Structure::PowerPlant { .. } => "Power Plant",
            Structure::Mine { .. } => "Mine",
            Structure::Base { .. } => "Base",
            Structure::Storage { .. } => "Storage",
        };
        write!(f, "{}", name)
    }
}

pub trait StructureGroupTrait {
    fn group(&self) -> StructureGroup;
}

impl StructureGroupTrait for Structure {
    fn group(&self) -> StructureGroup {
        match self {
            Structure::Base { .. } => StructureGroup::Base,
            Structure::PowerPlant { .. } => StructureGroup::Energy,
            Structure::Mine { .. } => StructureGroup::Mine,
            Structure::Storage { .. } => StructureGroup::Storage,
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

    pub fn get_component(&self, name: &ComponentName) -> &ComponentGroup {
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

impl StorageTrait for StructureBlueprint {
    fn capacity(&self, group: &ResourceGroup) -> u64 {
        match self.get_component(&ComponentName::StorageComponent) {
            ComponentGroup::Storage {
                component: StorageComponent { capacity, .. },
            } => capacity[&group],
            _ => 0,
        }
    }

    fn resource(&self, group: &ResourceGroup) -> u64 {
        match self.get_component(&ComponentName::StorageComponent) {
            ComponentGroup::Storage {
                component: StorageComponent { resources, .. },
            } => resources[&group],
            _ => 0,
        }
    }

    fn resource_add(&mut self, group: &ResourceGroup, amount: u64) -> u64 {
        let component = self
            .components
            .get_mut(&ComponentName::StorageComponent)
            .unwrap();

        match component {
            ComponentGroup::Storage { ref mut component } => {
                let free_capacity = component.capacity_free(group);

                if free_capacity <= 0 {
                    return 0;
                }

                let left_over: i64 = amount as i64 - free_capacity as i64;

                if left_over <= 0 {
                    component.resource_add(group, amount);
                    return amount;
                }

                component.resource_add(group, free_capacity);
                return free_capacity;
            }
            _ => 0,
        }
    }
}

// Base
#[derive(Clone)]
pub struct Base {
    pub blueprint: StructureBlueprint,
}

impl Debug for Base {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Base {
    pub fn new() -> Base {
        let energy_component = ComponentGroup::Energy {
            component: EnergyComponent {
                energy_out: 25,
                energy_in: 0,
            },
        };

        let battery_component = ComponentGroup::Battery {
            component: BatteryComponent {
                capacity: 1000,
                used: 0,
            },
        };

        let mut components = HashMap::new();

        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::BatteryComponent, battery_component);

        let blueprint = StructureBlueprint { components };

        return Base { blueprint };
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        return &self.blueprint;
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
    resource: ResourceGroup,
}

impl Debug for Mine {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Mine {
    pub fn new(resource: ResourceGroup) -> Mine {
        let energy_component = ComponentGroup::Energy {
            component: EnergyComponent {
                energy_out: 0,
                energy_in: 25,
            },
        };

        let resource_component = ComponentGroup::Resource {
            component: ResourceComponent {
                resource_out: 100,
                resource_in: 0,
            },
        };

        let mut components = HashMap::new();

        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::ResourceComponent, resource_component);

        let blueprint = StructureBlueprint { components };

        return Mine {
            blueprint,
            resource,
        };
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        return &self.blueprint;
    }
    pub fn resource(&self) -> &ResourceGroup {
        return &self.resource;
    }
}

// Storage
#[derive(Clone)]
pub struct Storage {
    pub blueprint: StructureBlueprint,
}

impl Debug for Storage {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Storage {
    pub fn new() -> Storage {
        let storage_component = ComponentGroup::Storage {
            component: StorageComponent::new(),
        };

        let mut components = HashMap::new();

        components.insert(ComponentName::StorageComponent, storage_component);

        let blueprint = StructureBlueprint { components };

        return Storage { blueprint };
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        return &self.blueprint;
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        return &mut self.blueprint;
    }
}
