use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use worldgen::noise::perlin::PerlinNoise;
use worldgen::noisemap::{NoiseMap, NoiseMapGenerator, Seed, Step};
use worldgen::world::tile::Constraint;
use worldgen::world::tile::ConstraintType;
use worldgen::world::{Size, Tile, World};

#[derive(Clone)]
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

pub struct ResourceStorage {
    map: HashMap<ResourceGroup, u32>,
}

impl ResourceStorage {
    pub fn new(resources: Vec<ResourceGroup>) -> ResourceStorage {
        let mut map: HashMap<ResourceGroup, u32> = HashMap::new();

        for resource_type in resources {
            map.insert(resource_type, 0);
        }

        return ResourceStorage { map };
    }

    pub fn list(&self) -> &HashMap<ResourceGroup, u32> {
        return &self.map;
    }

    pub fn add(&mut self, resource_type: ResourceGroup, value: u32) {
        *self.map.get_mut(&resource_type).unwrap() += value;
    }
}

#[derive(Clone)]
pub struct MapTile {
    pub flora: Flora,
    pub resource: Option<Resource>,
    pub structure: Option<Structure>,
}

impl MapTile {
    pub fn new(flora: Flora, resource: Option<Resource>, structure: Option<Structure>) -> MapTile {
        return MapTile {
            flora,
            resource,
            structure,
        };
    }
}

pub struct MapTileFactory {}

impl MapTileFactory {
    pub fn new(flora: Flora) -> MapTile {
        return MapTile {
            flora,
            resource: Option::None,
            structure: Option::None,
        };
    }
}

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

#[derive(Clone)]
pub enum ComponentGroup {
    Energy { energy_in: f64, energy_out: f64 },
}

trait EnergyTrait {
    fn energy_in(&self) -> f64;
    fn energy_out(&self) -> f64;
}

pub struct EnergyComponent {
    energy_in: f64,
    energy_out: f64,
}

#[derive(Clone, Debug)]
pub enum Structure {
    PowerPlant { blueprint: PowerPlant },
}

impl Display for Structure {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let name = match self {
            Structure::PowerPlant { .. } => "Power Plant",
        };
        write!(f, "{:?}", name)
    }
}

#[derive(Clone)]
pub struct PowerPlant {
    group: StructureGroup,
    components: Vec<ComponentGroup>,
}

impl Debug for PowerPlant {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

impl PowerPlant {
    pub fn new() -> PowerPlant {
        let group = StructureGroup::Energy;
        let energy_component = ComponentGroup::Energy {
            energy_in: 0.0,
            energy_out: 0.0,
        };

        let components = vec![energy_component];

        return PowerPlant { group, components };
    }
}

impl EnergyTrait for PowerPlant {
    fn energy_in(&self) -> f64 {
        unimplemented!()
    }

    fn energy_out(&self) -> f64 {
        unimplemented!()
    }
}

type WorldCache = Vec<Vec<MapTile>>;

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

    pub fn set_structure(&mut self, x: usize, y: usize, structure: Structure) {
        self.cache.as_mut().unwrap()[y][x]
            .structure
            .replace(structure);
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
    position: Position,
}

impl MapController {
    pub fn new(size: Size) -> MapController {
        let position = Position::new(0, 0);
        let (w, h) = (size.w as u16, size.h as u16);

        let map = GameMap::new(w, h);

        return MapController { map, position };
    }

    pub fn map(&self) -> &GameMap {
        return &self.map;
    }

    pub fn map_mut(&mut self) -> &mut GameMap {
        return &mut self.map;
    }

    pub fn position(&self) -> Position {
        return self.position.clone();
    }

    pub fn tile(&self) -> MapTile {
        let x = self.position.x as usize;
        let y = self.position.y as usize;

        return self.map.cache.borrow().as_ref().unwrap()[y][x].clone();
    }

    pub fn tile_at(&self, position: Position) -> MapTile {
        let x = position.x as usize;
        let y = position.y as usize;

        return self.map.cache.borrow().as_ref().unwrap()[y][x].clone();
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
