use crate::game::Resource;
use crate::structures::CommodityGroup;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use std::iter::FromIterator;
use std::ops::AddAssign;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum ComponentName {
    EnergyComponent,
    ResourceOutputComponent,
    ResourceStorageComponent,
    CommodityStorageComponent,
    BatteryComponent,
    CommodityOutputComponent,
}

impl Display for ComponentName {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct EnergyComponent {
    pub energy_in: u64,
    pub energy_out: u64,
}

#[derive(Eq, PartialEq, Hash)]
pub struct BatteryComponent {
    pub capacity: u64,
    pub stored: u64,
}

pub struct ResourceOutputComponent {
    pub resource_out: u64,
}

pub struct ResourceStorageComponent {
    pub capacity: HashMap<Resource, u64>,
    pub resources: HashMap<Resource, u64>,
}

pub struct CommodityOutputComponent {
    pub commodity_out: u64,
    pub energy_required: u64,
    pub resource_required: HashMap<Resource, u64>,
}

pub struct CommodityStorageComponent {
    pub capacity: HashMap<CommodityGroup, u64>,
    pub commodities: HashMap<CommodityGroup, u64>,
}

impl ResourceStorageComponent {
    pub fn new() -> ResourceStorageComponent {
        let mut capacity = HashMap::new();
        capacity.insert(Resource::Metal, 1000);
        capacity.insert(Resource::Mineral, 1000);
        capacity.insert(Resource::Carbon, 1000);
        capacity.insert(Resource::Gas, 1000);

        let mut resources = HashMap::new();
        resources.insert(Resource::Metal, 0);
        resources.insert(Resource::Mineral, 0);
        resources.insert(Resource::Carbon, 0);
        resources.insert(Resource::Gas, 0);

        return ResourceStorageComponent {
            capacity,
            resources,
        };
    }

    pub fn capacity(&self, group: &Resource) -> u64 {
        return self.capacity[&group];
    }

    pub fn capacity_free(&self, group: &Resource) -> u64 {
        return self.capacity[&group] - self.resources[&group];
    }

    pub fn resource(&self, group: &Resource) -> u64 {
        return self.resources[&group];
    }

    pub fn resource_mut(&mut self, group: &Resource) -> &mut u64 {
        return self.resources.get_mut(&group).unwrap();
    }

    pub fn resource_add(&mut self, group: &Resource, amount: u64) {
        self.resources.get_mut(group).unwrap().add_assign(amount);
    }

    pub fn resources(&self) -> Vec<&Resource> {
        return Vec::from_iter(self.resources.keys());
    }
}

impl CommodityOutputComponent {
    pub fn resources(&self) -> Iter<'_, Resource, u64> {
        self.resource_required.iter()
    }
}

impl CommodityStorageComponent {
    pub fn new() -> CommodityStorageComponent {
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

    pub fn capacity(&self, group: &CommodityGroup) -> u64 {
        return self.capacity[&group];
    }

    pub fn capacity_free(&self, group: &CommodityGroup) -> u64 {
        return self.capacity[&group] - self.commodities[&group];
    }

    pub fn commodity(&self, group: &CommodityGroup) -> u64 {
        return self.commodities[&group];
    }

    pub fn commodity_mut(&mut self, group: &CommodityGroup) -> &mut u64 {
        return self.commodities.get_mut(&group).unwrap();
    }

    pub fn commodity_add(&mut self, group: &CommodityGroup, amount: u64) {
        self.commodities.get_mut(group).unwrap().add_assign(amount);
    }

    pub fn commodities(&self) -> Vec<&CommodityGroup> {
        return Vec::from_iter(self.commodities.keys());
    }
}

pub enum ComponentGroup {
    Energy {
        component: EnergyComponent,
    },
    ResourceOutput {
        component: ResourceOutputComponent,
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
            ComponentGroup::ResourceStorage { .. } => ComponentName::ResourceStorageComponent,
            ComponentGroup::CommodityOutput { .. } => ComponentName::CommodityOutputComponent,
            ComponentGroup::CommodityStorage { .. } => ComponentName::CommodityStorageComponent,
            ComponentGroup::Battery { .. } => ComponentName::BatteryComponent,
        };

        write!(f, "{}", name)
    }
}
