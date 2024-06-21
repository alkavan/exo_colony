use std::collections::hash_map::{Iter, IterMut};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};

use worldgen::noise::perlin::PerlinNoise;
use worldgen::noisemap::{NoiseMap, NoiseMapGenerator, Seed, Step};
use worldgen::world::tile::Constraint;
use worldgen::world::tile::ConstraintType;
use worldgen::world::{Size, Tile, World};

use crate::structures::{Structure, StructureGroup};
use std::iter::FromIterator;

type WorldCache = Vec<Vec<MapTile>>;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
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

impl Display for Position {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Resource {
    Metal,
    Mineral,
    Gas,
    Carbon,
}

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy)]
pub struct ResourceDeposit {
    resource: Resource,
    amount: u64,
}

impl ResourceDeposit {
    pub fn new(resource: Resource, amount: u64) -> ResourceDeposit {
        return ResourceDeposit { resource, amount };
    }
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

#[derive(Clone)]
pub struct MapTile {
    pub flora: Flora,
    pub resource: Option<Resource>,
}

pub struct MapObject {
    pub structure: Option<Structure>,
    pub deposit: Option<ResourceDeposit>,
}

pub struct ObjectManager {
    objects: HashMap<Position, MapObject>,
}

impl ObjectManager {
    pub fn new() -> ObjectManager {
        let objects = HashMap::new();
        ObjectManager { objects }
    }

    pub fn contains(&self, position: &Position) -> bool {
        self.objects.contains_key(position)
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

pub struct TileFactory {}

impl TileFactory {
    pub fn tile(
        flora: Flora,
        resource: Option<Resource>,
        constraints: Vec<Constraint>,
    ) -> Tile<MapTile> {
        let mut tile = Tile::new(MapTile { flora, resource });

        for constraint in constraints {
            tile = tile.when(constraint);
        }

        return tile;
    }
}

pub struct GameMap {
    width: u16,
    height: u16,
    world: World<MapTile>,
    cache: WorldCache,
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

        let nm = Box::new(nm1 + (nm2 * 4));

        let water_tile = TileFactory::tile(
            Flora::Water,
            Option::None,
            vec![constraint!(nm.clone(), < -0.25)],
        );
        let sand_tile = TileFactory::tile(
            Flora::Sand,
            Option::None,
            vec![constraint!(nm.clone(), < 0.0)],
        );
        let grass_tile = TileFactory::tile(
            Flora::Grass,
            Option::None,
            vec![constraint!(nm.clone(), < 0.45)],
        );
        let dirt_tile = TileFactory::tile(Flora::Dirt, Option::None, vec![]);

        let rock_tile = TileFactory::tile(
            Flora::Rock,
            Option::None,
            vec![
                constraint!(nm.clone(), > 0.8),
                constraint!(nm.clone(), < 0.88),
            ],
        );

        let rock_deposit_tile = TileFactory::tile(
            Flora::Rock,
            Option::from(Resource::Mineral),
            vec![constraint!(nm.clone(), > 0.87)],
        );

        let world = World::new()
            .set(Size::of(width as i64, height as i64))
            // Water
            .add(water_tile)
            // Sand
            .add(sand_tile)
            // Grass
            .add(grass_tile)
            // Mountains
            .add(rock_tile)
            .add(rock_deposit_tile)
            // Hills
            .add(dirt_tile);

        let cache = world.generate(0, 0).unwrap();

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
        return &self.cache;
    }

    pub fn cache_copy(&self) -> WorldCache {
        return WorldCache::from_iter(self.cache.iter().cloned());
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

    pub fn generate_deposits(&mut self) {
        let cache = self.map().cache_copy();

        for (y, row) in cache.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if tile.resource.is_some() {
                    // TODO: make amount random in range
                    let deposit = ResourceDeposit::new(tile.resource.unwrap(), 1000);

                    let object = MapObject {
                        structure: Option::None,
                        deposit: Option::from(deposit),
                    };

                    self.add_object(Position::new(x as i16, y as i16), object);
                }
            }
        }
    }

    pub fn add_structure(&mut self, structure: Structure) -> Option<MapObject> {
        let position = self.position();

        if self.objects.contains(&position) {
            let mut object = self.remove_object(&position)?;

            if object.structure.is_some() {
                panic!(
                    "trying add structure, but {} already exists at {}",
                    &object.structure.unwrap(),
                    &position
                )
            }

            // add deposit to object
            object.structure = Option::from(structure);

            // add structure to existing object
            return self.objects.set(position, object);
        }

        let object = MapObject {
            structure: Option::from(structure),
            deposit: Option::None,
        };

        self.add_object(position, object)
    }

    pub fn destroy_structure(&mut self) {
        let position = self.position();
        let mut object = self.remove_object(&position).unwrap();
        object.structure = Option::None;
        self.add_object(position, object);
    }

    pub fn position(&self) -> Position {
        return self.position.clone();
    }

    pub fn tile(&self) -> &MapTile {
        let x = self.position.x as usize;
        let y = self.position.y as usize;

        return &self.map.cache[y][x];
    }

    pub fn tile_at(&self, position: Position) -> &MapTile {
        let x = position.x as usize;
        let y = position.y as usize;

        return &self.map.cache[y][x];
    }

    pub fn tile_at_mut(&mut self, position: Position) -> &mut MapTile {
        let x = position.x as usize;
        let y = position.y as usize;

        return &mut self.map.cache[y][x];
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
