// SPDX-FileCopyrightText: Copyright (c) 2021-2026 Igal Alkon
// SPDX-License-Identifier: Zlib

use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::{AddAssign, Sub, SubAssign};

use crate::component::{
    BatteryComponent, CommodityStorageComponent, ComponentGroup, ComponentName, EnergyComponent,
    FactoryOutputComponent, MineOutputComponent, RefineryOutputComponent, ResourceStorageComponent,
};
use crate::game::{Commodity, Flora, Manufactured, MapObject, MapTile, Resource};
use crate::gui::MenuSelector;
use crate::managers::ResourceManager;
use std::slice::Iter;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructureGroup {
    Base,
    Power,
    Mine,
    Refinery,
    Factory,
    Storage,
}

impl Display for StructureGroup {
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

pub trait MineOutputTrait {
    fn resource_out(&self) -> u64;
    fn manufactured_out(&self) -> u64;
}

pub trait ResourceStorageTrait {
    fn capacity(&self, group: &Resource) -> u64;
    fn resource(&self, group: &Resource) -> u64;
    fn resource_add(&mut self, group: &Resource, amount: u64) -> u64;
    fn resource_take(&mut self, group: &Resource, amount: u64) -> u64;
    fn resources(&self) -> Vec<&Resource>;
}

pub trait CommodityStorageTrait {
    fn capacity(&self, group: &Commodity) -> u64;
    fn commodity(&self, group: &Commodity) -> u64;
    fn commodity_add(&mut self, group: &Commodity, amount: u64) -> u64;
    fn commodity_take(&mut self, group: &Commodity, amount: u64) -> u64;
    fn commodities(&self) -> Vec<&Commodity>;
}

#[derive(Debug)]
pub enum Structure {
    Base { structure: Base },
    PowerPlant { structure: PowerPlant },
    Mine { structure: Mine },
    Refinery { structure: Refinery },
    Factory { structure: Factory },
    Storage { structure: Storage },
}

impl Display for Structure {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            Structure::PowerPlant { .. } => "Power Plant",
            Structure::Mine { .. } => "Mine",
            Structure::Base { .. } => "Base",
            Structure::Refinery { .. } => "Refinery",
            Structure::Factory { .. } => "Factory",
            Structure::Storage { .. } => "Storage",
        };
        write!(f, "{}", name)
    }
}

impl Structure {
    pub fn blueprint(&self) -> &StructureBlueprint {
        match self {
            Structure::Base { structure } => structure.blueprint(),
            Structure::PowerPlant { structure } => structure.blueprint(),
            Structure::Mine { structure } => structure.blueprint(),
            Structure::Refinery { structure } => structure.blueprint(),
            Structure::Factory { structure } => structure.blueprint(),
            Structure::Storage { structure } => structure.blueprint(),
        }
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        match self {
            Structure::Base { structure } => structure.blueprint_mut(),
            Structure::PowerPlant { structure } => structure.blueprint_mut(),
            Structure::Mine { structure } => structure.blueprint_mut(),
            Structure::Refinery { structure } => structure.blueprint_mut(),
            Structure::Factory { structure } => structure.blueprint_mut(),
            Structure::Storage { structure } => structure.blueprint_mut(),
        }
    }
}

pub trait StructureGroupTrait {
    fn group(&self) -> StructureGroup;
}

impl StructureGroupTrait for Structure {
    fn group(&self) -> StructureGroup {
        match self {
            Structure::Base { .. } => StructureGroup::Base,
            Structure::PowerPlant { .. } => StructureGroup::Power,
            Structure::Mine { .. } => StructureGroup::Mine,
            Structure::Factory { .. } => StructureGroup::Factory,
            Structure::Refinery { .. } => StructureGroup::Refinery,
            Structure::Storage { .. } => StructureGroup::Storage,
        }
    }
}

pub struct StructureBlueprint {
    components: HashMap<ComponentName, ComponentGroup>,
}

impl StructureBlueprint {
    pub fn get_component(&self, name: &ComponentName) -> &ComponentGroup {
        let component = self.components.get(name);

        if component.is_some() {
            component.unwrap()
        } else {
            panic!("{} is missing in this structure", name)
        }
    }

    pub fn get_component_mut(&mut self, name: &ComponentName) -> &mut ComponentGroup {
        let component = self.components.get_mut(name);

        if component.is_some() {
            component.unwrap()
        } else {
            panic!("{} is missing in this structure", name)
        }
    }

    pub fn has_component(&self, name: &ComponentName) -> bool {
        self.components.get(name).is_some()
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
                amount
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
                amount
            }
            _ => 0,
        }
    }
}

impl MineOutputTrait for StructureBlueprint {
    fn resource_out(&self) -> u64 {
        match self.get_component(&ComponentName::MineOutputComponent) {
            ComponentGroup::MineOutput {
                component: MineOutputComponent { resource_out, .. },
            } => *resource_out,
            _ => 0,
        }
    }

    fn manufactured_out(&self) -> u64 {
        match self.get_component(&ComponentName::MineOutputComponent) {
            ComponentGroup::MineOutput {
                component:
                    MineOutputComponent {
                        manufactured_out, ..
                    },
            } => *manufactured_out,
            _ => 0,
        }
    }
}

impl ResourceStorageTrait for StructureBlueprint {
    fn capacity(&self, group: &Resource) -> u64 {
        match self.get_component(&ComponentName::ResourceStorageComponent) {
            ComponentGroup::ResourceStorage { component } => component.capacity(group),
            _ => 0,
        }
    }

    fn resource(&self, group: &Resource) -> u64 {
        match self.get_component(&ComponentName::ResourceStorageComponent) {
            ComponentGroup::ResourceStorage { component } => component.resource(group),
            _ => 0,
        }
    }

    fn resource_add(&mut self, group: &Resource, amount: u64) -> u64 {
        let component = self.get_component_mut(&ComponentName::ResourceStorageComponent);

        match component {
            ComponentGroup::ResourceStorage { component } => {
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
                free_capacity
            }
            _ => 0,
        }
    }

    fn resource_take(&mut self, group: &Resource, amount: u64) -> u64 {
        let component = self.get_component_mut(&ComponentName::ResourceStorageComponent);
        match component {
            ComponentGroup::ResourceStorage { component } => component.resource_take(group, amount),
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
            ComponentGroup::CommodityStorage { component } => component.capacity(group),
            _ => 0,
        }
    }

    fn commodity(&self, group: &Commodity) -> u64 {
        match self.get_component(&ComponentName::CommodityStorageComponent) {
            ComponentGroup::CommodityStorage { component } => component.commodity(group),
            _ => 0,
        }
    }

    fn commodity_add(&mut self, group: &Commodity, amount: u64) -> u64 {
        let component = self.get_component_mut(&ComponentName::CommodityStorageComponent);

        match component {
            ComponentGroup::CommodityStorage { component } => {
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
                free_capacity
            }
            _ => 0,
        }
    }

    fn commodity_take(&mut self, group: &Commodity, amount: u64) -> u64 {
        let component = self.get_component_mut(&ComponentName::CommodityStorageComponent);
        match component {
            ComponentGroup::CommodityStorage { component } => {
                component.commodity_take(group, amount)
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

        Base { blueprint }
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        &self.blueprint
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        &mut self.blueprint
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
                energy_out: 120,
                energy_in: 0,
            },
        };

        let mut components = HashMap::new();
        components.insert(ComponentName::EnergyComponent, energy_component);

        let blueprint = StructureBlueprint { components };

        PowerPlant { blueprint }
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        &self.blueprint
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        &mut self.blueprint
    }
}

// Mine
pub struct Mine {
    blueprint: StructureBlueprint,
    resource: Resource,
    manufactured: Manufactured,
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
                energy_in: 15,
            },
        };

        let resource_component = ComponentGroup::MineOutput {
            component: MineOutputComponent {
                resource_out: 1,
                manufactured_out: 1,
            },
        };

        let mut components = HashMap::new();

        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::MineOutputComponent, resource_component);

        let manufactured = Manufactured::Gravel;

        let blueprint = StructureBlueprint { components };

        Mine {
            blueprint,
            resource,
            manufactured,
        }
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        &self.blueprint
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        &mut self.blueprint
    }

    pub fn resource(&self) -> &Resource {
        &self.resource
    }

    pub fn manufactured(&self) -> &Manufactured {
        &self.manufactured
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

        Storage { blueprint }
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        &self.blueprint
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        &mut self.blueprint
    }
}

pub struct ResourceRequireFactory {}

impl ResourceRequireFactory {
    fn energy_for_manufactured(resource: &Manufactured) -> u64 {
        match resource {
            Manufactured::Silicon => 30,
            Manufactured::Steel => 25,
            Manufactured::Gravel => 10,
            Manufactured::Hydrogen => 20,
            Manufactured::FuelPellet => 50,
        }
    }

    fn resources_for_manufactured(resource: &Manufactured) -> HashMap<Resource, u64> {
        let mut requires = HashMap::new();

        match resource {
            Manufactured::Silicon => {
                requires.insert(Resource::Silica, 4);
            }
            Manufactured::Steel => {
                requires.insert(Resource::Iron, 3);
            }
            Manufactured::Gravel => {}
            Manufactured::Hydrogen => {
                requires.insert(Resource::Water, 6);
            }
            Manufactured::FuelPellet => {
                requires.insert(Resource::Uranium, 2);
            }
        }

        requires
    }
}

pub struct CommodityRequireFactory {}

impl CommodityRequireFactory {
    fn energy_for_commodity(commodity: &Commodity) -> u64 {
        match commodity {
            Commodity::Concrete => 25,
            Commodity::Fuel => 20,
            Commodity::Semiconductor => 40,
            Commodity::Glass => 35,
            Commodity::FuelRod => 60,
        }
    }

    fn resources_for_commodity(commodity: &Commodity) -> HashMap<Resource, u64> {
        let mut requires = HashMap::new();

        match commodity {
            Commodity::Concrete => {
                requires.insert(Resource::Silica, 5);
            }
            Commodity::Fuel => {}
            Commodity::Semiconductor => {
                requires.insert(Resource::Aluminum, 3);
                requires.insert(Resource::Carbon, 4);
            }
            Commodity::Glass => {
                requires.insert(Resource::Silica, 6);
            }
            Commodity::FuelRod => {}
        }

        requires
    }

    fn manufactured_for_commodity(commodity: &Commodity) -> HashMap<Manufactured, u64> {
        let mut requires = HashMap::new();

        match commodity {
            Commodity::Concrete => {
                requires.insert(Manufactured::Gravel, 5);
            }
            Commodity::Fuel => {
                requires.insert(Manufactured::Hydrogen, 2);
            }
            Commodity::Semiconductor => {
                requires.insert(Manufactured::Silicon, 2);
            }
            Commodity::Glass => {
                requires.insert(Manufactured::Silicon, 1);
            }
            Commodity::FuelRod => {
                requires.insert(Manufactured::FuelPellet, 1);
                requires.insert(Manufactured::Steel, 1);
            }
        }

        requires
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

        let energy_required = CommodityRequireFactory::energy_for_commodity(&commodity);
        let resource_required = CommodityRequireFactory::resources_for_commodity(&commodity);
        let manufactured_required =
            CommodityRequireFactory::manufactured_for_commodity(&commodity);

        let commodity_component = ComponentGroup::FactoryOutput {
            component: FactoryOutputComponent {
                commodity_out: 1,
                energy_required,
                resource_required,
                manufactured_required,
            },
        };

        let mut components = HashMap::new();
        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::FactoryOutputComponent, commodity_component);

        let blueprint = StructureBlueprint { components };

        Factory {
            blueprint,
            commodity,
        }
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        &self.blueprint
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        &mut self.blueprint
    }

    pub fn commodity(&self) -> &Commodity {
        &self.commodity
    }
}

// Refinery
pub struct Refinery {
    blueprint: StructureBlueprint,
    resources: Vec<Manufactured>,
}

impl Debug for Refinery {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl Refinery {
    pub fn new(resources: Vec<Manufactured>) -> Refinery {
        let energy_component = ComponentGroup::Energy {
            component: EnergyComponent {
                energy_out: 0,
                energy_in: 50,
            },
        };

        let mut manufactured_out = HashMap::new();
        let mut energy_required = HashMap::new();
        let mut resource_required = HashMap::new();

        for resource in resources.iter().clone() {
            manufactured_out.insert(resource.clone(), 1);
            energy_required.insert(
                resource.clone(),
                ResourceRequireFactory::energy_for_manufactured(&resource),
            );
            resource_required.insert(
                resource.clone(),
                ResourceRequireFactory::resources_for_manufactured(&resource),
            );
        }

        let refinery_component = ComponentGroup::RefineryOutput {
            component: RefineryOutputComponent {
                manufactured_out,
                energy_required,
                resource_required,
            },
        };

        let mut components = HashMap::new();
        components.insert(ComponentName::EnergyComponent, energy_component);
        components.insert(ComponentName::RefineryOutputComponent, refinery_component);

        let blueprint = StructureBlueprint { components };

        Refinery {
            blueprint,
            resources,
        }
    }

    pub fn blueprint(&self) -> &StructureBlueprint {
        &self.blueprint
    }

    pub fn blueprint_mut(&mut self) -> &mut StructureBlueprint {
        &mut self.blueprint
    }

    pub fn resources(&self) -> Iter<'_, Manufactured> {
        self.resources.iter()
    }
}

pub struct StructureFactory {}

impl StructureFactory {
    pub fn new(
        group: &StructureGroup,
        object: Option<&MapObject>,
        resource_manager: &ResourceManager,
        refinery_select: &dyn MenuSelector<Vec<Manufactured>>,
        factory_select: &dyn MenuSelector<Commodity>,
    ) -> Option<Structure> {
        match group {
            StructureGroup::Base => {
                let structure = Structure::Base {
                    structure: Base::new(),
                };
                Option::from(structure)
            }
            StructureGroup::Power => {
                let structure = Structure::PowerPlant {
                    structure: PowerPlant::new(),
                };
                Option::from(structure)
            }
            StructureGroup::Mine => {
                let deposit = object.and_then(|o| o.deposit);
                let Some(deposit) = deposit else {
                    return None;
                };
                if deposit.is_exhausted() {
                    return None;
                }

                let map_resource = deposit.resource.clone();

                let structure = Structure::Mine {
                    structure: Mine::new(map_resource),
                };
                Option::from(structure)
            }
            StructureGroup::Refinery => {
                let structure = Structure::Refinery {
                    structure: Refinery::new(refinery_select.selected()),
                };
                Option::from(structure)
            }
            StructureGroup::Factory => {
                let structure = Structure::Factory {
                    structure: { Factory::new(factory_select.selected()) },
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
        }
    }

    pub fn allowed(group: &StructureGroup, tile: &MapTile) -> bool {
        match group {
            StructureGroup::Base => {
                !tile.is_resource && (tile.flora == Flora::Sand || tile.flora == Flora::Grass)
            }
            StructureGroup::Power => {
                !tile.is_resource && (tile.flora == Flora::Sand || tile.flora == Flora::Water)
            }
            StructureGroup::Mine => tile.is_resource,
            StructureGroup::Storage => {
                !tile.is_resource && (tile.flora == Flora::Sand || tile.flora == Flora::Grass)
            }
            StructureGroup::Factory => {
                !tile.is_resource && (tile.flora == Flora::Grass || tile.flora == Flora::Dirt)
            }
            StructureGroup::Refinery => {
                !tile.is_resource
                    && (tile.flora == Flora::Sand
                        || tile.flora == Flora::Dirt
                        || tile.flora == Flora::Grass)
            }
        }
    }
}
