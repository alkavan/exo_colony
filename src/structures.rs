use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use std::hash::Hash;
use std::ops::{AddAssign, Sub, SubAssign};

use crate::component::{
    BatteryComponent, CommodityOutputComponent, CommodityStorageComponent, ComponentGroup,
    ComponentName, EnergyComponent, ResourceOutputComponent, ResourceStorageComponent,
};
use crate::game::{Flora, MapObject, MapTile, Resource};
use crate::gui::MenuSelector;
use crate::managers::ResourceManager;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Commodity {
    Concrete,
    Electronics,
    Fuel,
    Glass,
}

impl Display for Commodity {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

pub trait EnergyTrait {
    fn energy_in(&self) -> u64;
    fn energy_out(&self) -> u64;
}

pub trait BatteryTrait {
    fn capacity(&self) -> u64;
    fn capacity_free(&self) -> u64;
    fn stored(&self) -> u64;
    fn charge(&mut self, amount: u64) -> u64;
    fn discharge(&mut self, amount: u64) -> u64;
}

pub trait ResourceOutputTrait {
    fn resource_out(&self) -> u64;
}

pub trait ResourceStorageTrait {
    fn capacity(&self, group: &Resource) -> u64;
    fn resource(&self, group: &Resource) -> u64;
    fn resource_add(&mut self, group: &Resource, amount: u64) -> u64;
    fn resources(&self) -> Vec<&Resource>;
}

pub trait CommodityStorageTrait {
    fn capacity(&self, group: &Commodity) -> u64;
    fn commodity(&self, group: &Commodity) -> u64;
    fn commodity_add(&mut self, group: &Commodity, amount: u64) -> u64;
    fn commodities(&self) -> Vec<&Commodity>;
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

    fn charge(&mut self, amount: u64) -> u64 {
        let component = self.get_component_mut(&ComponentName::BatteryComponent);

        match component {
            ComponentGroup::Battery {
                component: BatteryComponent { capacity, stored },
            } => {
                let free = capacity.sub(*stored);

                if free == 0 {
                    return 0;
                }

                if free <= amount {
                    stored.add_assign(free);
                    return free;
                }

                stored.add_assign(amount);
                return amount;
            }
            _ => 0,
        }
    }

    fn discharge(&mut self, amount: u64) -> u64 {
        let component = self.get_component_mut(&ComponentName::BatteryComponent);

        match component {
            ComponentGroup::Battery {
                component: BatteryComponent { stored, .. },
            } => {
                if *stored < amount {
                    let stored_available = *stored;
                    stored.sub_assign(stored_available);
                    return stored_available;
                }

                stored.sub_assign(amount);
                return amount;
            }
            _ => 0,
        }
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
    fn capacity(&self, group: &Resource) -> u64 {
        match self.get_component(&ComponentName::ResourceStorageComponent) {
            ComponentGroup::ResourceStorage {
                component: ResourceStorageComponent { capacity, .. },
            } => capacity[&group],
            _ => 0,
        }
    }

    fn resource(&self, group: &Resource) -> u64 {
        match self.get_component(&ComponentName::ResourceStorageComponent) {
            ComponentGroup::ResourceStorage {
                component: ResourceStorageComponent { resources, .. },
            } => resources[&group],
            _ => 0,
        }
    }

    fn resource_add(&mut self, group: &Resource, amount: u64) -> u64 {
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

    fn resources(&self) -> Vec<&Resource> {
        match self.get_component(&ComponentName::ResourceStorageComponent) {
            ComponentGroup::ResourceStorage { component } => component.resources(),
            _ => Vec::new(),
        }
    }
}

impl CommodityStorageTrait for StructureBlueprint {
    fn capacity(&self, group: &Commodity) -> u64 {
        match self.get_component(&ComponentName::CommodityStorageComponent) {
            ComponentGroup::CommodityStorage {
                component: CommodityStorageComponent { capacity, .. },
            } => capacity[&group],
            _ => 0,
        }
    }

    fn commodity(&self, group: &Commodity) -> u64 {
        match self.get_component(&ComponentName::CommodityStorageComponent) {
            ComponentGroup::CommodityStorage {
                component: CommodityStorageComponent { commodities, .. },
            } => commodities[&group],
            _ => 0,
        }
    }

    fn commodity_add(&mut self, group: &Commodity, amount: u64) -> u64 {
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

    fn commodities(&self) -> Vec<&Commodity> {
        match self.get_component(&ComponentName::CommodityStorageComponent) {
            ComponentGroup::CommodityStorage { component } => component.commodities(),
            _ => Vec::new(),
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
                energy_out: 50,
                energy_in: 0,
            },
        };

        let battery_component = ComponentGroup::Battery {
            component: BatteryComponent {
                capacity: 1000,
                stored: 0,
            },
        };

        let storage_resources = vec![
            Resource::Iron,
            Resource::Aluminum,
            Resource::Carbon,
            Resource::Silica,
            Resource::Uranium,
            Resource::Water,
        ];

        let storage_component = ComponentGroup::ResourceStorage {
            component: ResourceStorageComponent::new(storage_resources),
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

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        return &mut self.blueprint;
    }
}

// Mine
pub struct Mine {
    blueprint: StructureBlueprint,
    resource: Resource,
}

impl Debug for Mine {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Mine {
    pub fn new(resource: Resource) -> Mine {
        let energy_component = ComponentGroup::Energy {
            component: EnergyComponent {
                energy_out: 0,
                energy_in: 25,
            },
        };

        let resource_component = ComponentGroup::ResourceOutput {
            component: ResourceOutputComponent { resource_out: 1 },
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

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        return &mut self.blueprint;
    }

    pub fn resource(&self) -> &Resource {
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
    pub fn new(resources: Vec<Resource>, commodities: Vec<Commodity>) -> Storage {
        let resource_storage_component = ComponentGroup::ResourceStorage {
            component: ResourceStorageComponent::new(resources),
        };

        let commodity_storage_component = ComponentGroup::CommodityStorage {
            component: CommodityStorageComponent::new(commodities),
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

pub struct ResourceRequireFactory {}

impl ResourceRequireFactory {
    fn energy_for_commodity(commodity: &Commodity) -> u64 {
        match commodity {
            Commodity::Concrete => 45,
            Commodity::Fuel => 20,
            Commodity::Electronics => 40,
            Commodity::Glass => 120,
        }
    }

    fn resources_for_commodity(commodity: &Commodity) -> HashMap<Resource, u64> {
        let mut requires = HashMap::new();

        match commodity {
            Commodity::Concrete => {
                requires.insert(Resource::Silica, 15);
            }
            Commodity::Fuel => {
                requires.insert(Resource::Water, 35);
            }
            Commodity::Electronics => {
                requires.insert(Resource::Aluminum, 5);
                requires.insert(Resource::Carbon, 10);
                requires.insert(Resource::Silica, 25);
            }
            Commodity::Glass => {
                requires.insert(Resource::Silica, 50);
            }
        }

        return requires;
    }
}

// Factory
pub struct Factory {
    blueprint: StructureBlueprint,
    commodity: Commodity,
}

impl Debug for Factory {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Factory {
    pub fn new(commodity: Commodity) -> Factory {
        let energy_component = ComponentGroup::Energy {
            component: EnergyComponent {
                energy_out: 0,
                energy_in: 20,
            },
        };

        let energy_required = ResourceRequireFactory::energy_for_commodity(&commodity);
        let resource_required = ResourceRequireFactory::resources_for_commodity(&commodity);

        let commodity_component = ComponentGroup::CommodityOutput {
            component: CommodityOutputComponent {
                commodity_out: 1,
                energy_required,
                resource_required,
            },
        };

        let mut components = HashMap::new();
        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::CommodityOutputComponent, commodity_component);

        let blueprint = StructureBlueprint { components };

        return Factory {
            blueprint,
            commodity,
        };
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        return &self.blueprint;
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        return &mut self.blueprint;
    }

    pub fn commodity(&self) -> &Commodity {
        return &self.commodity;
    }
}

pub struct StructureFactory {}

impl StructureFactory {
    pub fn new(
        group: &StructureGroup,
        object: Option<&MapObject>,
        resource_manager: &ResourceManager,
        commodity_select: &dyn MenuSelector<Commodity>,
    ) -> Option<Structure> {
        match group {
            StructureGroup::Base => {
                let structure = Structure::Base {
                    structure: Base::new(),
                };
                Option::from(structure)
            }
            StructureGroup::Energy => {
                let structure = Structure::PowerPlant {
                    structure: PowerPlant::new(),
                };
                Option::from(structure)
            }
            StructureGroup::Mine => {
                if object.is_none() {
                    panic!("cannot build mine, map object missing!")
                }

                let map_resource = object.unwrap().deposit.unwrap().resource.clone();

                let structure = Structure::Mine {
                    structure: Mine::new(map_resource),
                };
                Option::from(structure)
            }
            StructureGroup::Storage => {
                let structure = Structure::Storage {
                    structure: Storage::new(
                        resource_manager.resource_types(),
                        resource_manager.commodity_types(),
                    ),
                };
                Option::from(structure)
            }
            StructureGroup::Factory => {
                let structure = Structure::Factory {
                    structure: { Factory::new(commodity_select.selected()) },
                };
                Option::from(structure)
            }
        }
    }

    pub fn allowed(group: &StructureGroup, tile: &MapTile) -> bool {
        match group {
            StructureGroup::Base => {
                !tile.is_resource && (tile.flora == Flora::Sand || tile.flora == Flora::Grass)
            }
            StructureGroup::Energy => {
                !tile.is_resource && (tile.flora == Flora::Sand || tile.flora == Flora::Water)
            }
            StructureGroup::Mine => tile.is_resource,
            StructureGroup::Storage => {
                !tile.is_resource && (tile.flora == Flora::Sand || tile.flora == Flora::Grass)
            }
            StructureGroup::Factory => {
                !tile.is_resource && (tile.flora == Flora::Grass || tile.flora == Flora::Dirt)
            }
        }
    }
}
