// SPDX-FileCopyrightText: Copyright (c) 2021-2026 Igal Alkon
// SPDX-License-Identifier: Zlib

use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut};
use std::ops::{AddAssign, SubAssign};

use crate::component::{ComponentGroup, ComponentName};
use crate::game::{Commodity, Manufactured, MapObject, Position, Resource};
use crate::structures::{
    BatteryTrait, CommodityStorageTrait, EnergyTrait, MineOutputTrait, ResourceStorageTrait,
    Structure, StructureGroup,
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

        EnergyManager {
            output,
            stored,
            discharged,
            deficit,
        }
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

    pub fn collect(&mut self, objects: IterMut<Position, MapObject>) {
        let filtered = objects.filter(|(_, o)| o.is_operational());
        for (_, object) in filtered {
            let structure = object.structure.as_ref().unwrap();

            match structure {
                Structure::Base { structure } => {
                    self.output.add_assign(structure.blueprint().energy_out());
                    self.stored.add_assign(structure.blueprint().stored());
                    if structure.blueprint().energy_out() > 0 {
                        object.active = true;
                    }
                }
                Structure::PowerPlant { structure } => {
                    self.output.add_assign(structure.blueprint().energy_out());
                    object.active = true;
                }
                _ => {}
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
        amount
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
        amount
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
        amount
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
        available
    }

    pub fn charge(&mut self, objects: IterMut<Position, MapObject>) {
        let filtered = objects.filter(|(_, o)| o.is_operational());
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
                _ => {}
            }
        }
    }

    pub fn discharge(&mut self, objects: IterMut<Position, MapObject>) {
        let filtered = objects.filter(|(_, o)| o.is_operational());

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
                _ => {}
            }
        }
    }
}

pub struct ResourceManager {
    resources: HashMap<Resource, u64>,
    resources_deficit: HashMap<Resource, u64>,
    resources_produced: HashMap<Resource, u64>,
    resources_consumed: HashMap<Resource, u64>,
    manufactured: HashMap<Manufactured, u64>,
    manufactured_deficit: HashMap<Manufactured, u64>,
    manufactured_produced: HashMap<Manufactured, u64>,
    commodities: HashMap<Commodity, u64>,
    commodities_deficit: HashMap<Commodity, u64>,
    commodities_produced: HashMap<Commodity, u64>,
}

impl ResourceManager {
    pub fn new(
        resource_types: Vec<Resource>,
        manufactured_types: Vec<Manufactured>,
        commodity_types: Vec<Commodity>,
    ) -> ResourceManager {
        let mut resources: HashMap<Resource, u64> = HashMap::new();
        for resource_type in resource_types {
            resources.insert(resource_type, 0);
        }

        let mut resources_deficit: HashMap<Resource, u64> = HashMap::new();
        for resource_type in resources.keys().copied().collect::<Vec<_>>() {
            resources_deficit.insert(resource_type, 0);
        }

        let mut manufactured: HashMap<Manufactured, u64> = HashMap::new();
        for manufactured_type in manufactured_types {
            manufactured.insert(manufactured_type, 0);
        }

        let mut manufactured_deficit: HashMap<Manufactured, u64> = HashMap::new();
        for manufactured_type in manufactured.keys().copied().collect::<Vec<_>>() {
            manufactured_deficit.insert(manufactured_type, 0);
        }

        let mut commodities: HashMap<Commodity, u64> = HashMap::new();
        for commodity_type in commodity_types {
            commodities.insert(commodity_type, 0);
        }

        let mut commodities_deficit: HashMap<Commodity, u64> = HashMap::new();
        for commodity_type in commodities.keys().copied().collect::<Vec<_>>() {
            commodities_deficit.insert(commodity_type, 0);
        }

        let resources_produced = resources.keys().copied().map(|k| (k, 0)).collect();
        let resources_consumed = resources.keys().copied().map(|k| (k, 0)).collect();
        let manufactured_produced = manufactured.keys().copied().map(|k| (k, 0)).collect();
        let commodities_produced = commodities.keys().copied().map(|k| (k, 0)).collect();

        ResourceManager {
            resources,
            resources_deficit,
            resources_produced,
            resources_consumed,
            manufactured,
            manufactured_deficit,
            manufactured_produced,
            commodities,
            commodities_deficit,
            commodities_produced,
        }
    }

    /// Landing crate so the first Base / Power / Mine can be paid for.
    pub fn seed_landing_crate(&mut self) {
        self.deposit_resource(&Resource::Iron, 40);
        self.deposit_resource(&Resource::Aluminum, 12);
        self.deposit_resource(&Resource::Carbon, 16);
        self.deposit_resource(&Resource::Silica, 24);
        self.deposit_commodity(&Commodity::Concrete, 12);
        self.deposit_manufactured(&Manufactured::Steel, 10);
        self.deposit_manufactured(&Manufactured::Gravel, 20);
    }

    pub fn resource_types(&self) -> Vec<Resource> {
        Vec::from_iter(self.resources.keys().cloned())
    }

    pub fn resources(&self) -> Iter<'_, Resource, u64> {
        self.resources.iter()
    }

    pub fn resources_mut(&mut self) -> IterMut<'_, Resource, u64> {
        self.resources.iter_mut()
    }

    pub fn has_resource(&self, resource_type: &Resource, amount: u64) -> bool {
        let available = self.resources.get(resource_type).unwrap();
        *available >= amount
    }

    pub fn deposit_resource(&mut self, resource_type: &Resource, amount: u64) -> u64 {
        let stored = self.resources.get_mut(&resource_type).unwrap();
        stored.add_assign(amount);
        stored.clone()
    }

    pub fn withdraw_resource(&mut self, resource_type: &Resource, amount: u64) -> u64 {
        let stored = self.resources.get_mut(&resource_type).unwrap();

        if amount > *stored {
            let available = stored.clone();
            stored.sub_assign(available);
            return available;
        }

        stored.sub_assign(amount);
        amount
    }

    fn add_resource_deficit(&mut self, resource_type: &Resource, amount: u64) {
        self.resources_deficit
            .get_mut(&resource_type)
            .unwrap()
            .add_assign(amount)
    }

    pub fn get_resource_deficit(&self, resource_type: &Resource) -> u64 {
        self.resources_deficit.get(resource_type).unwrap().clone()
    }

    pub fn get_resource_produced(&self, resource_type: &Resource) -> u64 {
        self.resources_produced
            .get(resource_type)
            .copied()
            .unwrap_or(0)
    }

    pub fn get_resource_consumed(&self, resource_type: &Resource) -> u64 {
        self.resources_consumed
            .get(resource_type)
            .copied()
            .unwrap_or(0)
    }

    pub fn manufactured_types(&self) -> Vec<Manufactured> {
        Vec::from_iter(self.manufactured.keys().cloned())
    }

    pub fn manufactured(&self) -> Iter<'_, Manufactured, u64> {
        self.manufactured.iter()
    }

    pub fn has_manufactured(&self, manufactured_type: &Manufactured, amount: u64) -> bool {
        let available = self.manufactured.get(manufactured_type).unwrap();
        *available >= amount
    }

    pub fn deposit_manufactured(&mut self, manufactured_type: &Manufactured, amount: u64) -> u64 {
        let stored = self.manufactured.get_mut(&manufactured_type).unwrap();
        stored.add_assign(amount);
        stored.clone()
    }

    pub fn withdraw_manufactured(&mut self, manufactured_type: &Manufactured, amount: u64) -> u64 {
        let stored = self.manufactured.get_mut(&manufactured_type).unwrap();

        if amount > *stored {
            let available = stored.clone();
            stored.sub_assign(available);
            return available;
        }

        stored.sub_assign(amount);
        amount
    }

    fn add_manufactured_deficit(&mut self, manufactured_type: &Manufactured, amount: u64) {
        self.manufactured_deficit
            .get_mut(&manufactured_type)
            .unwrap()
            .add_assign(amount)
    }

    pub fn get_manufactured_produced(&self, manufactured_type: &Manufactured) -> u64 {
        self.manufactured_produced
            .get(manufactured_type)
            .copied()
            .unwrap_or(0)
    }

    pub fn get_manufactured_deficit(&self, manufactured_type: &Manufactured) -> u64 {
        self.manufactured_deficit
            .get(manufactured_type)
            .unwrap()
            .clone()
    }

    pub fn commodity_types(&self) -> Vec<Commodity> {
        Vec::from_iter(self.commodities.keys().cloned())
    }

    pub fn commodities(&self) -> Iter<'_, Commodity, u64> {
        self.commodities.iter()
    }

    pub fn commodities_mut(&mut self) -> IterMut<'_, Commodity, u64> {
        self.commodities.iter_mut()
    }

    pub fn deposit_commodity(&mut self, commodity_type: &Commodity, value: u64) -> u64 {
        let stored = self.commodities.get_mut(&commodity_type).unwrap();
        stored.add_assign(value);
        stored.clone()
    }

    pub fn withdraw_commodity(&mut self, commodity_type: &Commodity, amount: u64) -> u64 {
        let stored = self.commodities.get_mut(&commodity_type).unwrap();

        if amount > *stored {
            let available = stored.clone();
            stored.sub_assign(available);
            return available;
        }

        stored.sub_assign(amount);
        amount
    }

    fn add_commodity_deficit(&mut self, commodity_type: &Commodity, amount: u64) {
        self.commodities_deficit
            .get_mut(&commodity_type)
            .unwrap()
            .add_assign(amount)
    }

    pub fn get_commodity_deficit(&self, resource_type: &Commodity) -> u64 {
        self.commodities_deficit.get(resource_type).unwrap().clone()
    }

    pub fn get_commodity_produced(&self, commodity_type: &Commodity) -> u64 {
        self.commodities_produced
            .get(commodity_type)
            .copied()
            .unwrap_or(0)
    }

    fn add_resource_produced(&mut self, resource_type: &Resource, amount: u64) {
        if let Some(slot) = self.resources_produced.get_mut(resource_type) {
            slot.add_assign(amount);
        }
    }

    fn add_resource_consumed(&mut self, resource_type: &Resource, amount: u64) {
        if let Some(slot) = self.resources_consumed.get_mut(resource_type) {
            slot.add_assign(amount);
        }
    }

    fn add_manufactured_produced(&mut self, manufactured_type: &Manufactured, amount: u64) {
        if let Some(slot) = self.manufactured_produced.get_mut(manufactured_type) {
            slot.add_assign(amount);
        }
    }

    fn add_commodity_produced(&mut self, commodity_type: &Commodity, amount: u64) {
        if let Some(slot) = self.commodities_produced.get_mut(commodity_type) {
            slot.add_assign(amount);
        }
    }

    fn zero_tick(&mut self) {
        for (_, deficit) in self.resources_deficit.iter_mut() {
            *deficit = 0;
        }
        for (_, deficit) in self.manufactured_deficit.iter_mut() {
            *deficit = 0;
        }
        for (_, deficit) in self.commodities_deficit.iter_mut() {
            *deficit = 0;
        }
        for resource in self.resource_types() {
            if let Some(slot) = self.resources_produced.get_mut(&resource) {
                *slot = 0;
            }
            if let Some(slot) = self.resources_consumed.get_mut(&resource) {
                *slot = 0;
            }
        }
        for manufactured in self.manufactured_types() {
            if let Some(slot) = self.manufactured_produced.get_mut(&manufactured) {
                *slot = 0;
            }
        }
        for commodity in self.commodity_types() {
            if let Some(slot) = self.commodities_produced.get_mut(&commodity) {
                *slot = 0;
            }
        }
    }

    fn bins_resource(slots: &[(Position, &mut MapObject)], resource_type: &Resource) -> u64 {
        let mut total = 0;
        for (_, object) in slots.iter() {
            let Some(structure) = object.structure.as_ref() else {
                continue;
            };
            let blueprint = structure.blueprint();
            if blueprint.has_component(&ComponentName::ResourceStorageComponent) {
                total += ResourceStorageTrait::resource(blueprint, resource_type);
            }
        }
        total
    }

    fn colony_has_resource(
        &self,
        resource_type: &Resource,
        amount: u64,
        slots: &[(Position, &mut MapObject)],
    ) -> bool {
        if self.has_resource(resource_type, amount) {
            return true;
        }
        let global = *self.resources.get(resource_type).unwrap_or(&0);
        global + Self::bins_resource(slots, resource_type) >= amount
    }

    pub fn collect(
        &mut self,
        objects: IterMut<Position, MapObject>,
        energy_manager: &mut EnergyManager,
    ) -> Vec<(Position, Resource)> {
        self.zero_tick();

        let mut exhausted = Vec::new();

        let mut slots: Vec<(Position, &mut MapObject)> = objects
            .filter(|(_, object)| object.is_operational())
            .map(|(position, object)| (position.clone(), object))
            .collect();

        for i in 0..slots.len() {
            let mine = match slots[i].1.structure.as_ref() {
                Some(Structure::Mine { structure }) => Some((
                    structure.blueprint().energy_in(),
                    *structure.resource(),
                    *structure.manufactured(),
                    structure.blueprint().resource_out(),
                    structure.blueprint().manufactured_out(),
                )),
                _ => None,
            };
            let Some((energy_required, resource, manufactured, resource_out, manufactured_out)) =
                mine
            else {
                continue;
            };

            let vein_left = slots[i]
                .1
                .deposit
                .map(|deposit| deposit.available)
                .unwrap_or(0);
            if vein_left == 0 {
                self.add_resource_deficit(&resource, resource_out);
                self.add_manufactured_deficit(&manufactured, manufactured_out);
                continue;
            }

            if energy_manager.has_energy(energy_required) {
                energy_manager.withdraw(energy_required);
                let extracted = slots[i]
                    .1
                    .deposit
                    .as_mut()
                    .map(|deposit| deposit.take(resource_out))
                    .unwrap_or(0);
                if extracted > 0 {
                    self.deposit_resource(&resource, extracted);
                    self.add_resource_produced(&resource, extracted);
                    let gravel = if extracted >= resource_out {
                        manufactured_out
                    } else {
                        0
                    };
                    if gravel > 0 {
                        self.deposit_manufactured(&manufactured, gravel);
                        self.add_manufactured_produced(&manufactured, gravel);
                    }
                    slots[i].1.active = true;
                }
                if slots[i]
                    .1
                    .deposit
                    .map(|deposit| deposit.is_exhausted())
                    .unwrap_or(false)
                {
                    exhausted.push((slots[i].0.clone(), resource));
                }
            } else {
                energy_manager.add_deficit(energy_required);
                self.add_resource_deficit(&resource, resource_out);
                self.add_manufactured_deficit(&manufactured, manufactured_out);
            }
        }

        for i in 0..slots.len() {
            let recipe = match slots[i].1.structure.as_ref() {
                Some(Structure::Refinery { structure }) => {
                    let component = structure
                        .blueprint()
                        .get_component(&ComponentName::RefineryOutputComponent);
                    if let ComponentGroup::RefineryOutput { component } = component {
                        let energy_required = component.resource_required_sum();
                        let mut inputs = Vec::new();
                        for (_, required) in component.resources() {
                            for (resource, amount) in required {
                                inputs.push((*resource, *amount));
                            }
                        }
                        let outputs: Vec<(Manufactured, u64)> = structure
                            .resources()
                            .map(|item| (*item, component.manufactured_out[item]))
                            .collect();
                        Some((energy_required, inputs, outputs))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            let Some((energy_required, inputs, outputs)) = recipe else {
                continue;
            };

            let has_energy = energy_manager.has_energy(energy_required);
            let has_resources = inputs
                .iter()
                .all(|(resource, amount)| self.colony_has_resource(resource, *amount, &slots));

            if has_energy && has_resources {
                energy_manager.withdraw(energy_required);
                for (resource, amount) in &inputs {
                    self.pull_resource(resource, *amount, &mut slots);
                    self.add_resource_consumed(resource, *amount);
                }
                for (manufactured, amount) in &outputs {
                    self.deposit_manufactured(manufactured, *amount);
                    self.add_manufactured_produced(manufactured, *amount);
                }
                slots[i].1.active = true;
            } else {
                for (manufactured, amount) in &outputs {
                    self.add_manufactured_deficit(manufactured, *amount);
                }
            }
        }

        for i in 0..slots.len() {
            let recipe = match slots[i].1.structure.as_ref() {
                Some(Structure::Factory { structure }) => {
                    let component = structure
                        .blueprint()
                        .get_component(&ComponentName::FactoryOutputComponent);
                    if let ComponentGroup::FactoryOutput { component } = component {
                        let inputs: Vec<(Resource, u64)> = component
                            .resources()
                            .map(|(resource, amount)| (*resource, *amount))
                            .collect();
                        let manufactured_inputs: Vec<(Manufactured, u64)> = component
                            .manufactured()
                            .map(|(item, amount)| (*item, *amount))
                            .collect();
                        Some((
                            component.energy_required,
                            inputs,
                            manufactured_inputs,
                            *structure.commodity(),
                            component.commodity_out,
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            let Some((energy_required, inputs, manufactured_inputs, commodity, commodity_out)) =
                recipe
            else {
                continue;
            };

            let has_energy = energy_manager.has_energy(energy_required);
            let has_resources = inputs
                .iter()
                .all(|(resource, amount)| self.colony_has_resource(resource, *amount, &slots));
            let has_manufactured = manufactured_inputs
                .iter()
                .all(|(item, amount)| self.has_manufactured(item, *amount));

            if has_energy && has_resources && has_manufactured {
                energy_manager.withdraw(energy_required);
                for (resource, amount) in &inputs {
                    self.pull_resource(resource, *amount, &mut slots);
                    self.add_resource_consumed(resource, *amount);
                }
                for (item, amount) in &manufactured_inputs {
                    self.withdraw_manufactured(item, *amount);
                }
                self.deposit_commodity(&commodity, commodity_out);
                self.add_commodity_produced(&commodity, commodity_out);
                slots[i].1.active = true;
            } else {
                self.add_commodity_deficit(&commodity, commodity_out);
            }
        }

        for i in 0..slots.len() {
            let is_storage = matches!(
                slots[i].1.structure.as_ref(),
                Some(Structure::Storage { .. })
            );
            if !is_storage {
                continue;
            }
            let mut parked = false;
            if let Some(Structure::Storage { structure }) = slots[i].1.structure.as_mut() {
                for (resource, amount) in self.resources_mut() {
                    if *amount > 0 {
                        let amount_stored = structure
                            .blueprint_mut()
                            .resource_add(resource, amount.clone());
                        amount.sub_assign(amount_stored);
                        if amount_stored > 0 {
                            parked = true;
                        }
                    }
                }
                for (commodity, amount) in self.commodities_mut() {
                    if *amount > 0 {
                        let amount_stored = structure
                            .blueprint_mut()
                            .commodity_add(commodity, amount.clone());
                        amount.sub_assign(amount_stored);
                        if amount_stored > 0 {
                            parked = true;
                        }
                    }
                }
            }
            if parked {
                slots[i].1.active = true;
            }
        }
        exhausted
    }

    pub fn warehouse_resources(objects: Iter<'_, Position, MapObject>) -> HashMap<Resource, u64> {
        let mut totals: HashMap<Resource, u64> = HashMap::new();
        for (_, object) in objects {
            let Some(structure) = object.structure.as_ref() else {
                continue;
            };
            let blueprint = structure.blueprint();
            if !blueprint.has_component(&ComponentName::ResourceStorageComponent) {
                continue;
            }
            for resource in blueprint.resources() {
                let stored = ResourceStorageTrait::resource(blueprint, resource);
                totals
                    .entry(*resource)
                    .and_modify(|n: &mut u64| n.add_assign(stored))
                    .or_insert(stored);
            }
        }
        totals
    }

    pub fn warehouse_commodities(
        objects: Iter<'_, Position, MapObject>,
    ) -> HashMap<Commodity, u64> {
        let mut totals: HashMap<Commodity, u64> = HashMap::new();
        for (_, object) in objects {
            let Some(structure) = object.structure.as_ref() else {
                continue;
            };
            let blueprint = structure.blueprint();
            if !blueprint.has_component(&ComponentName::CommodityStorageComponent) {
                continue;
            }
            for commodity in blueprint.commodities() {
                let stored = CommodityStorageTrait::commodity(blueprint, commodity);
                totals
                    .entry(*commodity)
                    .and_modify(|n: &mut u64| n.add_assign(stored))
                    .or_insert(stored);
            }
        }
        totals
    }

    pub fn resource_total(
        &self,
        resource_type: &Resource,
        warehouse: &HashMap<Resource, u64>,
    ) -> u64 {
        self.resources.get(resource_type).copied().unwrap_or(0)
            + warehouse.get(resource_type).copied().unwrap_or(0)
    }

    pub fn commodity_total(
        &self,
        commodity_type: &Commodity,
        warehouse: &HashMap<Commodity, u64>,
    ) -> u64 {
        self.commodities.get(commodity_type).copied().unwrap_or(0)
            + warehouse.get(commodity_type).copied().unwrap_or(0)
    }

    fn pull_resource(
        &mut self,
        resource_type: &Resource,
        amount: u64,
        objects: &mut [(Position, &mut MapObject)],
    ) -> u64 {
        let from_global = self.withdraw_resource(resource_type, amount);
        let mut left = amount - from_global;
        if left == 0 {
            return amount;
        }
        let mut from_bins = 0;
        for (_, object) in objects.iter_mut() {
            let Some(structure) = object.structure.as_mut() else {
                continue;
            };
            let blueprint = structure.blueprint_mut();
            if !blueprint.has_component(&ComponentName::ResourceStorageComponent) {
                continue;
            }
            let pulled = blueprint.resource_take(resource_type, left);
            from_bins += pulled;
            left -= pulled;
            if left == 0 {
                break;
            }
        }
        from_global + from_bins
    }

    fn pull_commodity(
        &mut self,
        commodity_type: &Commodity,
        amount: u64,
        objects: &mut [(Position, &mut MapObject)],
    ) -> u64 {
        let from_global = self.withdraw_commodity(commodity_type, amount);
        let mut left = amount - from_global;
        if left == 0 {
            return amount;
        }
        let mut from_bins = 0;
        for (_, object) in objects.iter_mut() {
            let Some(structure) = object.structure.as_mut() else {
                continue;
            };
            let blueprint = structure.blueprint_mut();
            if !blueprint.has_component(&ComponentName::CommodityStorageComponent) {
                continue;
            }
            let pulled = blueprint.commodity_take(commodity_type, left);
            from_bins += pulled;
            left -= pulled;
            if left == 0 {
                break;
            }
        }
        from_global + from_bins
    }

    pub fn pay(
        &mut self,
        cost: &BuildCost,
        objects: IterMut<'_, Position, MapObject>,
    ) -> Result<(), String> {
        let mut slots: Vec<(Position, &mut MapObject)> = objects
            .map(|(position, object)| (position.clone(), object))
            .collect();

        for (resource, amount) in &cost.resources {
            let have_global = *self.resources.get(resource).unwrap_or(&0);
            let mut have_bins = 0;
            for (_, object) in slots.iter() {
                let Some(structure) = object.structure.as_ref() else {
                    continue;
                };
                let blueprint = structure.blueprint();
                if blueprint.has_component(&ComponentName::ResourceStorageComponent) {
                    have_bins += ResourceStorageTrait::resource(blueprint, resource);
                }
            }
            if have_global + have_bins < *amount {
                return Err(format!(
                    "need {} {} (have {})",
                    amount,
                    resource,
                    have_global + have_bins
                ));
            }
        }
        for (manufactured, amount) in &cost.manufactured {
            if !self.has_manufactured(manufactured, *amount) {
                let have = *self.manufactured.get(manufactured).unwrap_or(&0);
                return Err(format!("need {} {} (have {})", amount, manufactured, have));
            }
        }
        for (commodity, amount) in &cost.commodities {
            let have_global = *self.commodities.get(commodity).unwrap_or(&0);
            let mut have_bins = 0;
            for (_, object) in slots.iter() {
                let Some(structure) = object.structure.as_ref() else {
                    continue;
                };
                let blueprint = structure.blueprint();
                if blueprint.has_component(&ComponentName::CommodityStorageComponent) {
                    have_bins += CommodityStorageTrait::commodity(blueprint, commodity);
                }
            }
            if have_global + have_bins < *amount {
                return Err(format!(
                    "need {} {} (have {})",
                    amount,
                    commodity,
                    have_global + have_bins
                ));
            }
        }

        for (resource, amount) in &cost.resources {
            self.pull_resource(resource, *amount, &mut slots);
        }
        for (manufactured, amount) in &cost.manufactured {
            self.withdraw_manufactured(manufactured, *amount);
        }
        for (commodity, amount) in &cost.commodities {
            self.pull_commodity(commodity, *amount, &mut slots);
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct BuildCost {
    pub resources: Vec<(Resource, u64)>,
    pub manufactured: Vec<(Manufactured, u64)>,
    pub commodities: Vec<(Commodity, u64)>,
}

impl BuildCost {
    pub fn for_group(group: &StructureGroup) -> BuildCost {
        match group {
            StructureGroup::Base => BuildCost {
                resources: vec![(Resource::Iron, 15)],
                commodities: vec![(Commodity::Concrete, 5)],
                manufactured: vec![],
            },
            StructureGroup::Power => BuildCost {
                resources: vec![(Resource::Silica, 5)],
                commodities: vec![],
                manufactured: vec![(Manufactured::Steel, 5)],
            },
            StructureGroup::Mine => BuildCost {
                resources: vec![(Resource::Iron, 8)],
                commodities: vec![],
                manufactured: vec![],
            },
            StructureGroup::Refinery => BuildCost {
                resources: vec![(Resource::Iron, 8)],
                commodities: vec![],
                manufactured: vec![(Manufactured::Steel, 8)],
            },
            StructureGroup::Factory => BuildCost {
                resources: vec![(Resource::Iron, 8)],
                commodities: vec![(Commodity::Concrete, 5)],
                manufactured: vec![(Manufactured::Steel, 5)],
            },
            StructureGroup::Storage => BuildCost {
                resources: vec![(Resource::Iron, 8)],
                commodities: vec![(Commodity::Concrete, 5)],
                manufactured: vec![],
            },
        }
    }

    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        for (item, amount) in &self.resources {
            parts.push(format!("{} {}", amount, item));
        }
        for (item, amount) in &self.manufactured {
            parts.push(format!("{} {}", amount, item));
        }
        for (item, amount) in &self.commodities {
            parts.push(format!("{} {}", amount, item));
        }
        if parts.is_empty() {
            "free".to_string()
        } else {
            parts.join(", ")
        }
    }
}
