use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use std::hash::Hash;
use std::ops::AddAssign;

use crate::game::ResourceGroup;
use itertools::Itertools;
use std::iter::FromIterator;

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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CommodityGroup {
    MetalPlate,
    MetalPipe,
    Gravel,
    Fuel,
}

impl Display for CommodityGroup {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

pub struct EnergyComponent {
    pub energy_in: u64,
    pub energy_out: u64,
}

pub trait EnergyTrait {
    fn energy_in(&self) -> u64;
    fn energy_out(&self) -> u64;
}

#[derive(Hash, Eq, PartialEq)]
pub struct BatteryComponent {
    pub capacity: u64,
    pub stored: u64,
}

pub trait BatteryTrait {
    fn capacity(&self) -> u64;
    fn capacity_free(&self) -> u64;
    fn stored(&self) -> u64;
    fn charge(&mut self, amount: u64);
}

pub struct ResourceOutputComponent {
    pub resource_out: u64,
}

pub trait ResourceOutputTrait {
    fn resource_out(&self) -> u64;
}

pub struct ResourceStorageComponent {
    pub capacity: HashMap<ResourceGroup, u64>,
    pub resources: HashMap<ResourceGroup, u64>,
}

impl ResourceStorageComponent {
    fn new() -> ResourceStorageComponent {
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

        return ResourceStorageComponent {
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

    pub fn resources(&self) -> Vec<&ResourceGroup> {
        return Vec::from_iter(self.resources.keys());
    }
}

pub trait ResourceStorageTrait {
    fn capacity(&self, group: &ResourceGroup) -> u64;
    fn resource(&self, group: &ResourceGroup) -> u64;
    fn resource_add(&mut self, group: &ResourceGroup, amount: u64) -> u64;
    fn resources(&self) -> Vec<&ResourceGroup>;
}

pub struct CommodityOutputComponent {
    pub commodity_out: u64,
}

pub trait CommodityOutputTrait {
    fn commodity_out(&self) -> u64;
}

pub struct CommodityStorageComponent {
    pub capacity: HashMap<CommodityGroup, u64>,
    pub commodities: HashMap<CommodityGroup, u64>,
}

impl CommodityStorageComponent {
    fn new() -> CommodityStorageComponent {
        let mut capacity = HashMap::new();
        capacity.insert(CommodityGroup::Fuel, 1000);
        capacity.insert(CommodityGroup::Gravel, 1000);
        capacity.insert(CommodityGroup::MetalPlate, 1000);
        capacity.insert(CommodityGroup::MetalPipe, 1000);

        let mut commodities = HashMap::new();
        commodities.insert(CommodityGroup::Fuel, 0);
        commodities.insert(CommodityGroup::Gravel, 0);
        commodities.insert(CommodityGroup::MetalPlate, 0);
        commodities.insert(CommodityGroup::MetalPipe, 0);

        return CommodityStorageComponent {
            capacity,
            commodities,
        };
    }

    fn capacity(&self, group: &CommodityGroup) -> u64 {
        return self.capacity[&group];
    }

    fn capacity_free(&self, group: &CommodityGroup) -> u64 {
        return self.capacity[&group] - self.commodities[&group];
    }

    fn commodity(&self, group: &CommodityGroup) -> u64 {
        return self.commodities[&group];
    }

    fn commodity_mut(&mut self, group: &CommodityGroup) -> &mut u64 {
        return self.commodities.get_mut(&group).unwrap();
    }

    fn commodity_add(&mut self, group: &CommodityGroup, amount: u64) {
        self.commodities.get_mut(group).unwrap().add_assign(amount);
    }

    pub fn commodities(&self) -> Vec<&CommodityGroup> {
        return Vec::from_iter(self.commodities.keys());
    }
}

pub trait CommodityStorageTrait {
    fn capacity(&self, group: &CommodityGroup) -> u64;
    fn commodity(&self, group: &CommodityGroup) -> u64;
    fn commodity_add(&mut self, group: &CommodityGroup, amount: u64) -> u64;
    fn commodities(&self) -> Vec<&CommodityGroup>;
}

pub struct ResourceRequireFactory {
    pub requires: HashMap<ResourceGroup, u64>,
}

pub trait ResourceRequire {
    fn requires(&self) -> Option<&HashMap<ResourceGroup, u64>>;
}

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum ComponentName {
    EnergyComponent,
    ResourceOutputComponent,
    ResourceStorageComponent,
    CommodityStorageComponent,
    BatteryComponent,
    CommodityOutputComponent,
    ResourceRequireComponent,
}

impl Display for ComponentName {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

pub enum ComponentGroup {
    Energy {
        component: EnergyComponent,
    },
    ResourceOutput {
        component: ResourceOutputComponent,
    },
    ResourceRequire {
        component: ResourceRequireFactory,
    },
    ResourceStorage {
        component: ResourceStorageComponent,
    },
    CommodityStorage {
        component: CommodityStorageComponent,
    },
    Battery {
        component: BatteryComponent,
    },
    CommodityOutput {
        component: CommodityOutputComponent,
    },
}

impl Display for ComponentGroup {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            ComponentGroup::Energy { .. } => ComponentName::EnergyComponent,
            ComponentGroup::ResourceOutput { .. } => ComponentName::ResourceOutputComponent,
            ComponentGroup::ResourceRequire { .. } => ComponentName::ResourceRequireComponent,
            ComponentGroup::ResourceStorage { .. } => ComponentName::ResourceStorageComponent,
            ComponentGroup::CommodityOutput { .. } => ComponentName::CommodityOutputComponent,
            ComponentGroup::CommodityStorage { .. } => ComponentName::CommodityStorageComponent,
            ComponentGroup::Battery { .. } => ComponentName::BatteryComponent,
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug)]
pub enum Structure {
    Base { structure: Base },
    PowerPlant { structure: PowerPlant },
    Mine { structure: Mine },
    Storage { structure: Storage },
    Factory { structure: Factory },
}

impl Display for Structure {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            Structure::PowerPlant { .. } => "Power Plant",
            Structure::Mine { .. } => "Mine",
            Structure::Base { .. } => "Base",
            Structure::Storage { .. } => "Storage",
            Structure::Factory { .. } => "Factory",
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
            Structure::Factory { .. } => StructureGroup::Factory,
        }
    }
}

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
            panic!("{} is missing in this structure", name)
        }
    }

    pub fn get_component_mut(&mut self, name: &ComponentName) -> &mut ComponentGroup {
        let component = self.components.get_mut(name);

        if component.is_some() {
            return component.unwrap();
        } else {
            panic!("{} is missing in this structure", name)
        }
    }

    pub fn has_component(&self, name: &ComponentName) -> bool {
        return self.components.get(name).is_some();
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

impl BatteryTrait for StructureBlueprint {
    fn capacity(&self) -> u64 {
        match self.get_component(&ComponentName::BatteryComponent) {
            ComponentGroup::Battery {
                component: BatteryComponent { capacity, .. },
            } => *capacity,
            _ => 0,
        }
    }

    fn capacity_free(&self) -> u64 {
        match self.get_component(&ComponentName::BatteryComponent) {
            ComponentGroup::Battery {
                component: BatteryComponent { capacity, stored },
            } => *capacity - *stored,
            _ => 0,
        }
    }

    fn stored(&self) -> u64 {
        match self.get_component(&ComponentName::BatteryComponent) {
            ComponentGroup::Battery {
                component: BatteryComponent { stored, .. },
            } => *stored,
            _ => 0,
        }
    }

    fn charge(&mut self, amount: u64) {
        let component = self.get_component_mut(&ComponentName::BatteryComponent);

        match component {
            ComponentGroup::Battery {
                component: BatteryComponent { capacity, stored },
            } => {
                let free = *capacity - *stored;

                if free > 0 {
                    if amount <= free {
                        *stored += amount;
                    } else {
                        *stored += free;
                    }
                };
            }
            _ => {}
        };
    }
}

impl ResourceOutputTrait for StructureBlueprint {
    fn resource_out(&self) -> u64 {
        match self.get_component(&ComponentName::ResourceOutputComponent) {
            ComponentGroup::ResourceOutput {
                component: ResourceOutputComponent { resource_out },
            } => *resource_out,
            _ => 0,
        }
    }
}

impl ResourceStorageTrait for StructureBlueprint {
    fn capacity(&self, group: &ResourceGroup) -> u64 {
        match self.get_component(&ComponentName::ResourceStorageComponent) {
            ComponentGroup::ResourceStorage {
                component: ResourceStorageComponent { capacity, .. },
            } => capacity[&group],
            _ => 0,
        }
    }

    fn resource(&self, group: &ResourceGroup) -> u64 {
        match self.get_component(&ComponentName::ResourceStorageComponent) {
            ComponentGroup::ResourceStorage {
                component: ResourceStorageComponent { resources, .. },
            } => resources[&group],
            _ => 0,
        }
    }

    fn resource_add(&mut self, group: &ResourceGroup, amount: u64) -> u64 {
        let component = self.get_component_mut(&ComponentName::ResourceStorageComponent);

        match component {
            ComponentGroup::ResourceStorage { ref mut component } => {
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

    fn resources(&self) -> Vec<&ResourceGroup> {
        match self.get_component(&ComponentName::ResourceStorageComponent) {
            ComponentGroup::ResourceStorage { component } => component.resources(),
            _ => Vec::new(),
        }
    }
}

impl CommodityStorageTrait for StructureBlueprint {
    fn capacity(&self, group: &CommodityGroup) -> u64 {
        match self.get_component(&ComponentName::CommodityStorageComponent) {
            ComponentGroup::CommodityStorage {
                component: CommodityStorageComponent { capacity, .. },
            } => capacity[&group],
            _ => 0,
        }
    }

    fn commodity(&self, group: &CommodityGroup) -> u64 {
        match self.get_component(&ComponentName::CommodityStorageComponent) {
            ComponentGroup::CommodityStorage {
                component: CommodityStorageComponent { commodities, .. },
            } => commodities[&group],
            _ => 0,
        }
    }

    fn commodity_add(&mut self, group: &CommodityGroup, amount: u64) -> u64 {
        let component = self.get_component_mut(&ComponentName::CommodityStorageComponent);

        match component {
            ComponentGroup::CommodityStorage { ref mut component } => {
                let free_capacity = component.capacity_free(group);

                if free_capacity <= 0 {
                    return 0;
                }

                let left_over: i64 = amount as i64 - free_capacity as i64;

                if left_over <= 0 {
                    component.commodity_add(group, amount);
                    return amount;
                }

                component.commodity_add(group, free_capacity);
                return free_capacity;
            }
            _ => 0,
        }
    }

    fn commodities(&self) -> Vec<&CommodityGroup> {
        match self.get_component(&ComponentName::CommodityStorageComponent) {
            ComponentGroup::CommodityStorage { component } => component.commodities(),
            _ => Vec::new(),
        }
    }
}

impl CommodityOutputTrait for StructureBlueprint {
    fn commodity_out(&self) -> u64 {
        match self.get_component(&ComponentName::CommodityOutputComponent) {
            ComponentGroup::CommodityOutput {
                component: CommodityOutputComponent { commodity_out, .. },
            } => *commodity_out,
            _ => 0,
        }
    }
}

impl ResourceRequire for StructureBlueprint {
    fn requires(&self) -> Option<&HashMap<ResourceGroup, u64>> {
        match self.get_component(&ComponentName::ResourceRequireComponent) {
            ComponentGroup::ResourceRequire {
                component: ResourceRequireFactory { requires },
            } => Option::from(requires),
            _ => Option::None,
        }
    }
}

// Base
pub struct Base {
    blueprint: StructureBlueprint,
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
                capacity: 5000,
                stored: 0,
            },
        };

        let storage_component = ComponentGroup::ResourceStorage {
            component: ResourceStorageComponent::new(),
        };

        let mut components = HashMap::new();

        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::BatteryComponent, battery_component);
        components.insert(ComponentName::ResourceStorageComponent, storage_component);

        let blueprint = StructureBlueprint { components };

        return Base { blueprint };
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        return &self.blueprint;
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        return &mut self.blueprint;
    }
}

// PowerPlant
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

        let resource_component = ComponentGroup::ResourceOutput {
            component: ResourceOutputComponent { resource_out: 100 },
        };

        let mut components = HashMap::new();

        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::ResourceOutputComponent, resource_component);

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
pub struct Storage {
    blueprint: StructureBlueprint,
}

impl Debug for Storage {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Storage {
    pub fn new() -> Storage {
        let resource_storage_component = ComponentGroup::ResourceStorage {
            component: ResourceStorageComponent::new(),
        };

        let commodity_storage_component = ComponentGroup::CommodityStorage {
            component: CommodityStorageComponent::new(),
        };

        let mut components = HashMap::new();

        components.insert(
            ComponentName::ResourceStorageComponent,
            resource_storage_component,
        );

        components.insert(
            ComponentName::CommodityStorageComponent,
            commodity_storage_component,
        );

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

impl ResourceRequireFactory {
    fn for_commodity(commodity: &CommodityGroup) -> HashMap<ResourceGroup, u64> {
        let mut requires = HashMap::new();

        match commodity {
            CommodityGroup::MetalPlate => {
                requires.insert(ResourceGroup::Metal, 15);
            }
            CommodityGroup::MetalPipe => {
                requires.insert(ResourceGroup::Metal, 5);
            }
            CommodityGroup::Gravel => {
                requires.insert(ResourceGroup::Mineral, 20);
            }
            CommodityGroup::Fuel => {
                requires.insert(ResourceGroup::Mineral, 5);
                requires.insert(ResourceGroup::Carbon, 35);
            }
        }

        return requires;
    }
}

// Factory
pub struct Factory {
    blueprint: StructureBlueprint,
    commodity: CommodityGroup,
}

impl Debug for Factory {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Factory {
    pub fn new(commodity: CommodityGroup) -> Factory {
        let energy_component = ComponentGroup::Energy {
            component: EnergyComponent {
                energy_out: 0,
                energy_in: 15,
            },
        };

        let requires = ResourceRequireFactory::for_commodity(&commodity);

        let resource_require_component = ComponentGroup::ResourceRequire {
            component: ResourceRequireFactory { requires },
        };

        let commodity_component = ComponentGroup::CommodityOutput {
            component: CommodityOutputComponent { commodity_out: 25 },
        };

        let mut components = HashMap::new();

        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::CommodityOutputComponent, commodity_component);
        components.insert(
            ComponentName::ResourceRequireComponent,
            resource_require_component,
        );

        let blueprint = StructureBlueprint { components };

        return Factory {
            blueprint,
            commodity,
        };
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        return &self.blueprint;
    }
    pub fn commodity(&self) -> &CommodityGroup {
        return &self.commodity;
    }
}
