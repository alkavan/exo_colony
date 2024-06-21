use std::collections::hash_map::{Iter, IterMut};
use std::collections::HashMap;
use std::ops::{AddAssign, SubAssign};

use crate::component::{ComponentGroup, ComponentName};
use crate::game::{MapObject, Position, Resource};
use crate::structures::{
    BatteryTrait, Commodity, EnergyTrait, ResourceOutputTrait, ResourceStorageTrait, Structure,
};

use std::iter::FromIterator;

pub struct EnergyManager {
    output: u64,
    stored: u64,
    discharged: u64,
    deficit: u64,
}

impl EnergyManager {
    pub fn new() -> EnergyManager {
        let output = 0;
        let stored = 0;
        let discharged = 0;
        let deficit = 0;

        return EnergyManager {
            output,
            stored,
            discharged,
            deficit,
        };
    }

    pub fn output(&self) -> u64 {
        self.output
    }

    pub fn stored(&self) -> u64 {
        self.stored
    }

    pub fn discharged(&self) -> u64 {
        self.discharged
    }

    pub fn deficit(&self) -> u64 {
        self.deficit
    }

    pub fn combined(&self) -> u64 {
        self.output + self.stored
    }

    pub fn zero(&mut self) {
        self.output = 0;
        self.stored = 0;
        self.discharged = 0;
        self.deficit = 0;
    }

    pub fn has_energy(&self, amount: u64) -> bool {
        self.output() >= amount || self.combined() >= amount
    }

    pub fn collect(&mut self, objects: Iter<Position, MapObject>) {
        let filtered = objects.filter(|(_, o)| o.structure.is_some());
        for (_, object) in filtered {
            let structure = object.structure.as_ref().unwrap();

            match structure {
                Structure::Base { structure } => {
                    self.output.add_assign(structure.blueprint().energy_out());
                    self.stored.add_assign(structure.blueprint().stored());
                }
                Structure::PowerPlant { structure } => {
                    self.output.add_assign(structure.blueprint().energy_out());
                }
                Structure::Mine { .. } => {}
                Structure::Storage { .. } => {}
                Structure::Factory { .. } => {}
            }
        }
    }

    pub fn withdraw_output(&mut self, amount: u64) -> u64 {
        if self.output == 0 || amount == 0 {
            return 0;
        }

        let available = self.output;
        if available < amount {
            self.output.sub_assign(available);
            return available;
        }

        self.output.sub_assign(amount);
        return amount;
    }

    pub fn withdraw_stored(&mut self, amount: u64) -> u64 {
        if self.stored == 0 || amount == 0 {
            return 0;
        }

        if self.stored < amount {
            let available = self.stored;
            self.stored.sub_assign(available);
            self.discharged.add_assign(available);
            return available;
        }

        self.stored.sub_assign(amount);
        self.discharged.add_assign(amount);
        return amount;
    }

    pub fn withdraw_discharge(&mut self, amount: u64) -> u64 {
        if self.discharged == 0 || amount == 0 {
            return 0;
        }

        let discharged = self.discharged;
        if self.discharged < amount {
            self.discharged.sub_assign(discharged);
            return discharged;
        }

        self.discharged.sub_assign(amount);
        return amount;
    }

    pub fn add_deficit(&mut self, amount: u64) {
        self.deficit.add_assign(amount);
    }

    pub fn withdraw(&mut self, amount: u64) -> u64 {
        let available_output = self.withdraw_output(amount);
        // when we have all requested energy from output
        if available_output >= amount {
            return amount;
        }

        let available_stored = self.withdraw_stored(amount - available_output);
        self.add_deficit(available_stored);
        let available = available_output + available_stored;

        // when we we have requested energy after using stored
        if available >= amount {
            return amount;
        }

        // when we don't have requested energy
        self.add_deficit(available);
        return available;
    }

    pub fn charge(&mut self, objects: IterMut<Position, MapObject>) {
        let filtered = objects.filter(|(_, o)| o.structure.is_some());
        for (_, object) in filtered {
            let structure = object.structure.as_mut().unwrap();

            match structure {
                Structure::Base { structure } => {
                    let battery_capacity_free = BatteryTrait::capacity_free(structure.blueprint());

                    if battery_capacity_free > 0 {
                        if self.output() > 0 {
                            let output_energy = self.withdraw_output(battery_capacity_free);
                            if output_energy > 0 {
                                BatteryTrait::charge(structure.blueprint_mut(), output_energy);
                            }
                        }
                    }
                }
                Structure::PowerPlant { structure } => {
                    self.output.add_assign(structure.blueprint().energy_out());
                }
                Structure::Mine { .. } => {}
                Structure::Storage { .. } => {}
                Structure::Factory { .. } => {}
            }
        }
    }

    pub fn discharge(&mut self, objects: IterMut<Position, MapObject>) {
        let filtered = objects.filter(|(_, o)| o.structure.is_some());

        for (_, object) in filtered {
            let structure = object.structure.as_mut().unwrap();

            match structure {
                Structure::Base { structure } => {
                    let battery_stored = BatteryTrait::stored(structure.blueprint());
                    let total_discharged = self.discharged();
                    // if we have energy in battery and energy deficit (used stored)
                    if battery_stored > 0 && total_discharged > 0 {
                        let discharged =
                            BatteryTrait::discharge(structure.blueprint_mut(), total_discharged);
                        self.withdraw_discharge(discharged);
                    }
                }
                Structure::PowerPlant { .. } => {}
                Structure::Mine { .. } => {}
                Structure::Storage { .. } => {}
                Structure::Factory { .. } => {}
            }
        }
    }
}

pub struct ResourceManager {
    resources: HashMap<Resource, u64>,
    resources_deficit: HashMap<Resource, u64>,
    commodities: HashMap<Commodity, u64>,
    commodities_deficit: HashMap<Commodity, u64>,
}

impl ResourceManager {
    pub fn new(resource_types: Vec<Resource>, commodity_types: Vec<Commodity>) -> ResourceManager {
        let mut resources: HashMap<Resource, u64> = HashMap::new();
        for resource_type in resource_types {
            resources.insert(resource_type, 0);
        }

        let mut resources_deficit: HashMap<Resource, u64> = HashMap::new();
        for resource_type in resources.keys().copied().collect::<Vec<_>>() {
            resources_deficit.insert(resource_type, 0);
        }

        let mut commodities: HashMap<Commodity, u64> = HashMap::new();
        for commodity_type in commodity_types {
            commodities.insert(commodity_type, 0);
        }

        let mut commodities_deficit: HashMap<Commodity, u64> = HashMap::new();
        for commodity_type in commodities.keys().copied().collect::<Vec<_>>() {
            commodities_deficit.insert(commodity_type, 0);
        }

        return ResourceManager {
            resources,
            resources_deficit,
            commodities,
            commodities_deficit,
        };
    }

    pub fn resource_types(&self) -> Vec<Resource> {
        return Vec::from_iter(self.resources.keys().cloned());
        // return vec![];
    }

    pub fn resources(&self) -> Iter<'_, Resource, u64> {
        return self.resources.iter();
    }

    pub fn resources_mut(&mut self) -> IterMut<'_, Resource, u64> {
        return self.resources.iter_mut();
    }

    pub fn has_resource(&self, resource_type: &Resource, amount: u64) -> bool {
        let available = self.resources.get(resource_type).unwrap();
        return *available > amount;
    }

    pub fn deposit_resource(&mut self, resource_type: &Resource, amount: u64) -> u64 {
        let stored = self.resources.get_mut(&resource_type).unwrap();
        stored.add_assign(amount);
        return stored.clone();
    }

    pub fn withdraw_resource(&mut self, resource_type: &Resource, amount: u64) -> u64 {
        let stored = self.resources.get_mut(&resource_type).unwrap();

        if amount > *stored {
            let available = stored.clone();
            stored.sub_assign(available);
            return available;
        }

        stored.sub_assign(amount);
        return amount;
    }

    fn add_resource_deficit(&mut self, resource_type: &Resource, amount: u64) {
        self.resources_deficit
            .get_mut(&resource_type)
            .unwrap()
            .add_assign(amount)
    }

    pub fn get_resource_deficit(&self, resource_type: &Resource) -> u64 {
        return self.resources_deficit.get(resource_type).unwrap().clone();
    }

    pub fn commodity_types(&self) -> Vec<Commodity> {
        return Vec::from_iter(self.commodities.keys().cloned());
        // return vec![];
    }

    pub fn commodities(&self) -> Iter<'_, Commodity, u64> {
        return self.commodities.iter();
    }

    pub fn commodities_mut(&mut self) -> IterMut<'_, Commodity, u64> {
        return self.commodities.iter_mut();
    }

    pub fn deposit_commodity(&mut self, commodity_type: &Commodity, value: u64) -> u64 {
        let stored = self.commodities.get_mut(&commodity_type).unwrap();
        stored.add_assign(value);
        return stored.clone();
    }

    pub fn withdraw_commodity(&mut self, commodity_type: &Commodity, amount: u64) -> u64 {
        let stored = self.commodities.get_mut(&commodity_type).unwrap();

        if amount > *stored {
            let available = stored.clone();
            stored.sub_assign(available);
            return available;
        }

        stored.sub_assign(amount);
        return amount;
    }

    fn add_commodity_deficit(&mut self, commodity_type: &Commodity, amount: u64) {
        self.commodities_deficit
            .get_mut(&commodity_type)
            .unwrap()
            .add_assign(amount)
    }

    pub fn get_commodity_deficit(&self, resource_type: &Commodity) -> u64 {
        return self.commodities_deficit.get(resource_type).unwrap().clone();
    }

    fn zero_deficit(&mut self) {
        for (_, deficit) in self.resources_deficit.iter_mut() {
            *deficit = 0;
        }

        for (_, deficit) in self.commodities_deficit.iter_mut() {
            *deficit = 0;
        }
    }

    pub fn collect(
        &mut self,
        objects: IterMut<Position, MapObject>,
        energy_manager: &mut EnergyManager,
    ) {
        self.zero_deficit();

        let filtered = objects.filter(|(_, o)| o.structure.is_some());

        for (_, object) in filtered {
            // let time_factor: f64 = update_tick.delta() as f64 / 2000.0;
            let structure = object.structure.as_mut().unwrap();

            match structure {
                Structure::PowerPlant { .. } => {}
                Structure::Mine { structure } => {
                    let energy_required = structure.blueprint().energy_in();
                    let resource = structure.resource();

                    if energy_manager.has_energy(energy_required) {
                        // resource mined.
                        energy_manager.withdraw(energy_required);
                        self.deposit_resource(resource, structure.blueprint().resource_out());
                    } else {
                        // resource not mined due to missing energy.
                        energy_manager.add_deficit(energy_required);
                        self.add_resource_deficit(resource, structure.blueprint().resource_out());
                    }
                }
                Structure::Base { .. } => {}
                Structure::Storage { structure } => {
                    for (resource, amount) in self.resources_mut() {
                        if *amount > 0 {
                            let amount_stored = structure
                                .blueprint_mut()
                                .resource_add(resource, amount.clone());

                            amount.sub_assign(amount_stored);
                        }
                    }
                }
                Structure::Factory { structure } => {
                    let component = structure
                        .blueprint()
                        .get_component(&ComponentName::CommodityOutputComponent);

                    if let ComponentGroup::CommodityOutput { component } = component {
                        let has_energy = energy_manager.has_energy(component.energy_required);
                        let has_resources =
                            component
                                .resources()
                                .all(|(required_resource, required_amount)| {
                                    self.has_resource(required_resource, required_amount.clone())
                                });

                        if has_energy && has_resources {
                            energy_manager.withdraw(component.energy_required);

                            for (required_resource, required_amount) in component.resources() {
                                self.withdraw_resource(required_resource, required_amount.clone());
                            }
                            self.deposit_commodity(structure.commodity(), component.commodity_out);
                        } else {
                            // if we don't have required resource to produce commodity we add to deficit
                            self.add_commodity_deficit(
                                structure.commodity(),
                                component.commodity_out,
                            )
                        }
                    }
                }
            }
        }
    }
}
