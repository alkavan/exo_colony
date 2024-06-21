use crate::structures::{
    BatteryTrait, CommodityGroup, CommodityOutputTrait, EnergyTrait, ResourceOutputTrait,
    ResourceRequire, ResourceStorageTrait, Structure, StructureGroup,
};
use std::collections::hash_map::{Iter, IterMut};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::{AddAssign, SubAssign};
use worldgen::noise::perlin::PerlinNoise;
use worldgen::noisemap::{NoiseMap, NoiseMapGenerator, Seed, Step};
use worldgen::world::tile::Constraint;
use worldgen::world::tile::ConstraintType;
use worldgen::world::{Size, Tile, World};

type WorldCache = Vec<Vec<MapTile>>;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub fn new(x: i16, y: i16) -> Position {
        return Position { x, y };
    }

    pub fn x(&mut self, x: i16) {
        self.x = x
    }

    pub fn y(&mut self, y: i16) {
        self.y = y
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceGroup {
    Energy,
    Metal,
    Mineral,
    Gas,
    Carbon,
}

impl Display for ResourceGroup {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
pub struct Resource {
    group: ResourceGroup,
    amount: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Flora {
    Water,
    Sand,
    Dirt,
    Grass,
    Rock,
}

impl Display for Flora {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

pub struct ResourceManager {
    resources: HashMap<ResourceGroup, u64>,
    commodities: HashMap<CommodityGroup, u64>,
}

impl ResourceManager {
    pub fn new(
        resource_types: Vec<ResourceGroup>,
        commodity_types: Vec<CommodityGroup>,
    ) -> ResourceManager {
        let mut resources: HashMap<ResourceGroup, u64> = HashMap::new();
        for resource_type in resource_types {
            resources.insert(resource_type, 0);
        }

        let mut commodities: HashMap<CommodityGroup, u64> = HashMap::new();
        for commodity_type in commodity_types {
            commodities.insert(commodity_type, 0);
        }

        return ResourceManager {
            resources,
            commodities,
        };
    }

    pub fn list_resources(&self) -> Iter<'_, ResourceGroup, u64> {
        return self.resources.iter();
    }

    pub fn list_resources_mut(&mut self) -> IterMut<'_, ResourceGroup, u64> {
        return self.resources.iter_mut();
    }

    pub fn has_resource(&self, resource_type: &ResourceGroup, amount: u64) -> bool {
        let available = self.resources.get(resource_type).unwrap();
        return *available > amount;
    }

    pub fn deposit_resource(&mut self, resource_type: &ResourceGroup, amount: u64) -> u64 {
        let stored = self.resources.get_mut(&resource_type).unwrap();
        *stored += amount;
        return *stored;
    }

    pub fn withdraw_resource(&mut self, resource_type: &ResourceGroup, amount: u64) -> u64 {
        let stored = self.resources.get_mut(&resource_type).unwrap();
        if amount > *stored {
            let available = *stored;
            *stored = 0;
            return available;
        }
        *stored -= amount;
        return amount;
    }

    pub fn list_commodities(&self) -> Iter<'_, CommodityGroup, u64> {
        return self.commodities.iter();
    }

    pub fn list_commodities_mut(&mut self) -> IterMut<'_, CommodityGroup, u64> {
        return self.commodities.iter_mut();
    }

    pub fn deposit_commodity(&mut self, resource_type: &CommodityGroup, value: u64) -> bool {
        if value == 0 {
            return false;
        }
        let amount = self.commodities.get_mut(&resource_type).unwrap();
        *amount += value;
        return true;
    }

    pub fn withdraw_commodity(&mut self, resource_type: &CommodityGroup, value: u64) -> bool {
        let amount = self.commodities.get_mut(&resource_type).unwrap();
        if *amount < value {
            return false;
        }
        *amount -= value;
        return true;
    }

    pub fn collect(
        &mut self,
        objects: IterMut<Position, MapObject>,
        energy_manager: &mut EnergyManager,
    ) {
        for (_, object) in objects {
            // let time_factor: f64 = update_tick.delta() as f64 / 2000.0;
            let structure = object.structure.as_mut().unwrap();

            match structure {
                Structure::PowerPlant { structure } => {}
                Structure::Mine { structure } => {
                    let energy_required = structure.blueprint().energy_in();
                    let energy_available = energy_manager.withdraw(energy_required);

                    if energy_available >= energy_required {
                        self.deposit_resource(
                            structure.resource(),
                            structure.blueprint().resource_out(),
                        );
                    } else {
                        let deficit = energy_required - energy_available;
                        energy_manager.deposit_deficit(deficit);
                    }
                }
                Structure::Base { structure } => {}
                Structure::Storage { structure } => {
                    for (resource, amount) in self.list_resources_mut() {
                        if *amount > 0 {
                            let amount_stored = structure
                                .blueprint_mut()
                                .resource_add(resource, amount.clone());

                            amount.sub_assign(amount_stored);
                        }
                    }
                }
                Structure::Factory { structure } => {
                    let requires = structure.blueprint().requires();

                    if requires.is_some() {
                        let has_resources =
                            requires
                                .unwrap()
                                .iter()
                                .all(|(required_resource, required_amount)| {
                                    if *required_resource == ResourceGroup::Energy {
                                        return energy_manager.has_energy(required_amount.clone());
                                    }

                                    self.has_resource(required_resource, required_amount.clone())
                                });

                        if has_resources {
                            for (required_resource, required_amount) in requires.unwrap().iter() {
                                if *required_resource == ResourceGroup::Energy {
                                    energy_manager.withdraw(required_amount.clone());
                                    continue;
                                }

                                self.withdraw_resource(required_resource, required_amount.clone());
                            }

                            let commodity_out = structure.blueprint().commodity_out();
                            self.deposit_commodity(structure.commodity(), commodity_out);
                        }
                    }
                }
            }
        }
    }
}

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
        for (_, object) in objects {
            let structure = object.structure.as_ref().unwrap();

            match structure {
                Structure::Base { structure } => {
                    self.output.add_assign(structure.blueprint().energy_out());
                    self.stored.add_assign(structure.blueprint().stored());
                }
                Structure::PowerPlant { structure } => {
                    self.output.add_assign(structure.blueprint().energy_out());
                }
                Structure::Mine { structure } => {}
                Structure::Storage { structure } => {}
                Structure::Factory { structure } => {}
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

    pub fn deposit_deficit(&mut self, amount: u64) {
        self.deficit.add_assign(amount);
    }

    pub fn withdraw(&mut self, amount: u64) -> u64 {
        let available_output = self.withdraw_output(amount);
        if available_output >= amount {
            return amount;
        }

        let available_stored = self.withdraw_stored(amount - available_output);
        let available = available_output + available_stored;

        if available >= amount {
            return amount;
        }

        return available;
    }

    pub fn charge(&mut self, objects: IterMut<Position, MapObject>) {
        for (_, object) in objects {
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
                Structure::Mine { structure } => {}
                Structure::Storage { structure } => {}
                Structure::Factory { structure } => {}
            }
        }
    }

    pub fn discharge(&mut self, objects: IterMut<Position, MapObject>) {
        for (_, object) in objects {
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
                Structure::PowerPlant { structure } => {}
                Structure::Mine { structure } => {}
                Structure::Storage { structure } => {}
                Structure::Factory { structure } => {}
            }
        }
    }
}

#[derive(Clone)]
pub struct MapTile {
    pub flora: Flora,
}

pub struct MapObject {
    pub resource: Option<Resource>,
    pub structure: Option<Structure>,
}

pub struct ObjectManager {
    objects: HashMap<Position, MapObject>,
}

impl ObjectManager {
    pub fn new() -> ObjectManager {
        let objects = HashMap::new();
        ObjectManager { objects }
    }
    pub fn get(&self, position: &Position) -> Option<&MapObject> {
        self.objects.get(position)
    }

    pub fn get_mut(&mut self, position: &Position) -> Option<&mut MapObject> {
        self.objects.get_mut(position)
    }

    pub fn set(&mut self, position: Position, object: MapObject) -> Option<MapObject> {
        self.objects.insert(position, object)
    }

    pub fn remove(&mut self, position: &Position) -> Option<MapObject> {
        self.objects.remove(&position)
    }

    pub fn list(&self) -> Iter<'_, Position, MapObject> {
        return self.objects.iter();
    }

    pub fn list_mut(&mut self) -> IterMut<'_, Position, MapObject> {
        return self.objects.iter_mut();
    }
}

pub struct MapTileFactory {}

impl MapTileFactory {
    pub fn new(flora: Flora) -> MapTile {
        return MapTile { flora };
    }
}

pub struct GameMap {
    width: u16,
    height: u16,
    world: World<MapTile>,
    cache: Option<WorldCache>,
}

impl GameMap {
    pub fn new(width: u16, height: u16) -> GameMap {
        let noise = PerlinNoise::new();

        let nm1 = NoiseMap::new(noise)
            .set_seed(Seed::of("FooMoo!"))
            .set_step(Step::of(0.005, 0.005));

        let nm2 = NoiseMap::new(noise)
            .set_seed(Seed::of("!GooToo"))
            .set_step(Step::of(0.05, 0.05));

        let nm = Box::new(nm1 + nm2 * 4);

        let water_tile = MapTileFactory::new(Flora::Water);
        let sand_tile = MapTileFactory::new(Flora::Sand);
        let grass_tile = MapTileFactory::new(Flora::Grass);
        let dirt_tile = MapTileFactory::new(Flora::Dirt);
        let rock_tile = MapTileFactory::new(Flora::Rock);

        let world = World::new()
            .set(Size::of(width as i64, height as i64))
            // Water
            .add(Tile::new(water_tile).when(constraint!(nm.clone(), < -0.25)))
            // Sand
            .add(Tile::new(sand_tile).when(constraint!(nm.clone(), < 0.0)))
            // Grass
            .add(Tile::new(grass_tile).when(constraint!(nm.clone(), < 0.45)))
            // Mountains
            .add(Tile::new(rock_tile).when(constraint!(nm.clone(), > 0.8)))
            // Hills
            .add(Tile::new(dirt_tile));

        let cache = world.generate(0, 0);

        return GameMap {
            width,
            height,
            world,
            cache,
        };
    }

    pub fn world(&self) -> &World<MapTile> {
        return &self.world;
    }

    pub fn cache(&self) -> &WorldCache {
        return self.cache.as_ref().unwrap();
    }

    pub fn width(&self) -> u16 {
        return self.width;
    }

    pub fn height(&self) -> u16 {
        return self.height;
    }
}

pub struct MapController {
    map: GameMap,
    objects: ObjectManager,
    position: Position,
    locations: HashMap<Position, StructureGroup>,
}

impl MapController {
    pub fn new(size: Size) -> MapController {
        let position = Position::new(0, 0);
        let (w, h) = (size.w as u16, size.h as u16);

        let map = GameMap::new(w, h);

        let objects = ObjectManager::new();

        let locations = HashMap::new();

        return MapController {
            map,
            objects,
            position,
            locations,
        };
    }

    pub fn map(&self) -> &GameMap {
        return &self.map;
    }

    pub fn locations(&self) -> &HashMap<Position, StructureGroup> {
        return &self.locations;
    }

    pub fn objects(&self) -> &ObjectManager {
        return &self.objects;
    }

    pub fn objects_mut(&mut self) -> &mut ObjectManager {
        return &mut self.objects;
    }

    pub fn add_object(&mut self, position: Position, object: MapObject) -> Option<MapObject> {
        self.objects.set(position, object)
    }

    pub fn remove_object(&mut self, position: &Position) -> Option<MapObject> {
        self.objects.remove(position)
    }

    pub fn object(&self) -> Option<&MapObject> {
        return self.objects.get(&self.position);
    }

    pub fn object_at(&self, position: &Position) -> Option<&MapObject> {
        return self.objects.get(position);
    }

    pub fn add_structure(&mut self, structure: Structure) -> Option<MapObject> {
        let position = self.position().clone();

        let object = MapObject {
            structure: Option::from(structure),
            resource: Option::None,
        };

        self.add_object(position, object)
    }

    pub fn destroy_structure(&mut self) {
        let position = &self.position();
        self.remove_object(position);
    }

    pub fn position(&self) -> Position {
        return self.position.clone();
    }

    pub fn tile(&self) -> &MapTile {
        let x = self.position.x as usize;
        let y = self.position.y as usize;

        return &self.map.cache.as_ref().unwrap()[y][x];
    }

    pub fn tile_at(&self, position: Position) -> &MapTile {
        let x = position.x as usize;
        let y = position.y as usize;

        return &self.map.cache.as_ref().unwrap()[y][x];
    }

    pub fn tile_at_mut(&mut self, position: Position) -> &mut MapTile {
        let x = position.x as usize;
        let y = position.y as usize;

        return &mut self.map.cache.as_mut().unwrap()[y][x];
    }

    pub fn up(&mut self) {
        let y = self.position.y as u16;
        if y > 0 {
            self.position.y((y - 1) as i16);
        }
    }

    pub fn down(&mut self) {
        let y = self.position.y as u16;
        if y < self.map.height() - 1 {
            self.position.y((y + 1) as i16);
        }
    }

    pub fn right(&mut self) {
        let x = self.position.x as u16;
        if x < self.map.width() - 1 {
            self.position.x((x + 1) as i16);
        }
    }

    pub fn left(&mut self) {
        let x = self.position.x as u16;
        if x > 0 {
            self.position.x((x - 1) as i16);
        }
    }
}
