use crate::game::Commodity;
use crate::game::Manufactured;
use crate::game::Resource;

use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use std::iter::FromIterator;
use std::ops::AddAssign;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub enum ComponentName {
    EnergyComponent,
    MineOutputComponent,
    ResourceStorageComponent,
    CommodityStorageComponent,
    BatteryComponent,
    RefineryOutputComponent,
    FactoryOutputComponent,
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

pub struct MineOutputComponent {
    pub resource_out: u64,
    pub manufactured_out: u64,
}

pub struct RefineryOutputComponent {
    pub manufactured_out: HashMap<Manufactured, u64>,
    pub energy_required: HashMap<Manufactured, u64>,
    pub resource_required: HashMap<Manufactured, HashMap<Resource, u64>>,
}

pub struct FactoryOutputComponent {
    pub commodity_out: u64,
    pub energy_required: u64,
    pub resource_required: HashMap<Resource, u64>,
}

pub struct ResourceStorageComponent {
    pub capacity: HashMap<Resource, u64>,
    pub resources: HashMap<Resource, u64>,
}

pub struct CommodityStorageComponent {
    pub capacity: HashMap<Commodity, u64>,
    pub commodities: HashMap<Commodity, u64>,
}

impl ResourceStorageComponent {
    pub fn new(items: Vec<Resource>) -> ResourceStorageComponent {
        let mut resources = HashMap::new();
        let mut capacity = HashMap::new();

        for resource in items {
            resources.insert(resource, 0);
            capacity.insert(resource, 1000);
        }

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

impl FactoryOutputComponent {
    pub fn resources(&self) -> Iter<'_, Resource, u64> {
        self.resource_required.iter()
    }
}

impl RefineryOutputComponent {
    pub fn resources(&self) -> Iter<'_, Manufactured, HashMap<Resource, u64>> {
        self.resource_required.iter()
    }

    pub fn resource_required_sum(&self) -> u64 {
        self.energy_required.values().sum()
    }
}

impl CommodityStorageComponent {
    pub fn new(items: Vec<Commodity>) -> CommodityStorageComponent {
        let mut commodities = HashMap::new();
        let mut capacity = HashMap::new();

        for commodity in items {
            commodities.insert(commodity, 0);
            capacity.insert(commodity, 1000);
        }

        return CommodityStorageComponent {
            capacity,
            commodities,
        };
    }

    pub fn capacity(&self, group: &Commodity) -> u64 {
        return self.capacity[&group];
    }

    pub fn capacity_free(&self, group: &Commodity) -> u64 {
        return self.capacity[&group] - self.commodities[&group];
    }

    pub fn commodity(&self, group: &Commodity) -> u64 {
        return self.commodities[&group];
    }

    pub fn commodity_mut(&mut self, group: &Commodity) -> &mut u64 {
        return self.commodities.get_mut(&group).unwrap();
    }

    pub fn commodity_add(&mut self, group: &Commodity, amount: u64) {
        self.commodities.get_mut(group).unwrap().add_assign(amount);
    }

    pub fn commodities(&self) -> Vec<&Commodity> {
        return Vec::from_iter(self.commodities.keys());
    }
}

pub enum ComponentGroup {
    Energy {
        component: EnergyComponent,
    },
    MineOutput {
        component: MineOutputComponent,
    },
    RefineryOutput {
        component: RefineryOutputComponent,
    },
    FactoryOutput {
        component: FactoryOutputComponent,
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
}

impl Display for ComponentGroup {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            ComponentGroup::Energy { .. } => ComponentName::EnergyComponent,
            ComponentGroup::MineOutput { .. } => ComponentName::MineOutputComponent,
            ComponentGroup::FactoryOutput { .. } => ComponentName::FactoryOutputComponent,
            ComponentGroup::RefineryOutput { .. } => ComponentName::RefineryOutputComponent,
            ComponentGroup::ResourceStorage { .. } => ComponentName::ResourceStorageComponent,
            ComponentGroup::CommodityStorage { .. } => ComponentName::CommodityStorageComponent,
            ComponentGroup::Battery { .. } => ComponentName::BatteryComponent,
        };

        write!(f, "{}", name)
    }
}
