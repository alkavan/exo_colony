// SPDX-FileCopyrightText: Copyright (c) 2021-2026 Igal Alkon
// SPDX-License-Identifier: Zlib

use std::collections::hash_map::{Iter, IterMut};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};

use worldgen::noise::perlin::PerlinNoise;
use worldgen::noisemap::{NoiseMap, NoiseMapGenerator, Seed, Step};
use worldgen::world::tile::Constraint;
use worldgen::world::tile::ConstraintType;
use worldgen::world::{Size, Tile, World};

use crate::structures::{Structure, StructureGroup};
use rand::prelude::SliceRandom;
use rand::RngExt;
use std::iter::FromIterator;

type WorldCache = Vec<Vec<MapTile>>;

/// Update ticks a newly placed structure spends under construction.
pub const CONSTRUCTION_TICKS: u8 = 8;

pub const WORLD_WIDTH: u16 = 256;
pub const WORLD_HEIGHT: u16 = 256;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

impl Position {
    pub fn new(x: i16, y: i16) -> Position {
        Position { x, y }
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
    Iron,
    Aluminum,
    Carbon,
    Silica,
    Uranium,
    Water,
}

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Manufactured {
    Silicon,
    Food,
    Steel,
    BioPlastic,
    Oxygen,
    Gravel,
    Hydrogen,
    FuelPellet,
}

impl Display for Manufactured {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Commodity {
    Concrete,
    Semiconductor,
    Fuel,
    Glass,
    FuelRod,
}

impl Display for Commodity {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

pub struct ResourceFactory {}

impl ResourceFactory {
    fn random_resource(resources: &mut Vec<Resource>) -> Resource {
        let mut rng = rand::rng();
        resources.shuffle(&mut rng);
        resources.get(0).unwrap().clone()
    }

    fn random_amount(from: u64, to: u64) -> u64 {
        let mut rng = rand::rng();
        rng.random_range(from..to)
    }

    fn random_resource_amount(resource: Resource) -> u64 {
        match resource {
            Resource::Iron => Self::random_amount(10000, 25000),
            Resource::Aluminum => Self::random_amount(10000, 25000),
            Resource::Carbon => Self::random_amount(5000, 15000),
            Resource::Silica => Self::random_amount(5000, 15000),
            Resource::Uranium => Self::random_amount(3000, 6000),
            Resource::Water => Self::random_amount(18000, 22000),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ResourceDeposit {
    pub resource: Resource,
    pub amount: u64,
    pub available: u64,
}

impl ResourceDeposit {
    pub fn new(resource: Resource, amount: u64) -> ResourceDeposit {
        ResourceDeposit {
            resource,
            amount,
            available: amount,
        }
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
    pub is_resource: bool,
}

pub struct MapObject {
    pub structure: Option<Structure>,
    pub deposit: Option<ResourceDeposit>,
    /// Remaining construction ticks. Zero means the structure is operational.
    pub construction_left: u8,
    /// True when the structure produced or supplied energy on the last update.
    pub active: bool,
}

impl MapObject {
    pub fn empty() -> MapObject {
        MapObject {
            structure: None,
            deposit: None,
            construction_left: 0,
            active: false,
        }
    }

    pub fn with_deposit(deposit: ResourceDeposit) -> MapObject {
        MapObject {
            structure: None,
            deposit: Some(deposit),
            construction_left: 0,
            active: false,
        }
    }

    pub fn is_constructing(&self) -> bool {
        self.structure.is_some() && self.construction_left > 0
    }

    pub fn is_operational(&self) -> bool {
        self.structure.is_some() && self.construction_left == 0
    }
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
        self.objects.iter()
    }

    pub fn list_mut(&mut self) -> IterMut<'_, Position, MapObject> {
        self.objects.iter_mut()
    }
}

pub struct TileFactory {}

impl TileFactory {
    pub fn tile(flora: Flora, is_resource: bool, constraints: Vec<Constraint>) -> Tile<MapTile> {
        let mut tile = Tile::new(MapTile { flora, is_resource });

        for constraint in constraints {
            tile = tile.when(constraint);
        }

        tile
    }
}

pub struct GameMap {
    width: u16,
    height: u16,
    seed: String,
    world: World<MapTile>,
    cache: WorldCache,
}

impl GameMap {
    pub fn new(width: u16, height: u16, seed: &str) -> GameMap {
        let noise = PerlinNoise::new();

        let nm1 = NoiseMap::new(noise)
            .set_seed(Seed::of(seed))
            .set_step(Step::of(0.008, 0.008));

        let detail_seed = format!("{}:hi", seed);
        let nm2 = NoiseMap::new(noise)
            .set_seed(Seed::of(detail_seed.as_str()))
            .set_step(Step::of(0.05, 0.05));

        let nm = Box::new(nm1 + (nm2 * 4));

        // Water
        let water_tile =
            TileFactory::tile(Flora::Water, false, vec![constraint!(nm.clone(), < -0.25)]);

        let water_deposit_tile = TileFactory::tile(
            Flora::Water,
            true,
            vec![
                constraint!(nm.clone(), < -0.57),
                constraint!(nm.clone(), > -0.6),
            ],
        );

        // Sand
        let sand_tile = TileFactory::tile(Flora::Sand, false, vec![constraint!(nm.clone(), < 0.0)]);

        let sand_deposit_tile = TileFactory::tile(
            Flora::Sand,
            true,
            vec![
                constraint!(nm.clone(), < -0.05),
                constraint!(nm.clone(), > -0.07),
            ],
        );

        // Grass
        let grass_tile =
            TileFactory::tile(Flora::Grass, false, vec![constraint!(nm.clone(), < 0.45)]);

        let grass_deposit_tile = TileFactory::tile(
            Flora::Grass,
            true,
            vec![
                constraint!(nm.clone(), < 0.3),
                constraint!(nm.clone(), > 0.27),
            ],
        );

        // Mountains
        let rock_tile =
            TileFactory::tile(Flora::Rock, false, vec![constraint!(nm.clone(), > 0.85)]);

        let rock_deposit_tile = TileFactory::tile(
            Flora::Rock,
            true,
            vec![
                constraint!(nm.clone(), < 0.88),
                constraint!(nm.clone(), > 0.86),
            ],
        );

        // Hills
        let dirt_tile = TileFactory::tile(Flora::Dirt, false, vec![]);

        let dirt_deposit_tile = TileFactory::tile(
            Flora::Dirt,
            true,
            vec![
                constraint!(nm.clone(), < 0.6),
                constraint!(nm.clone(), > 0.56),
            ],
        );

        let world = World::new()
            .set(Size::of(width as i64, height as i64))
            .add(water_deposit_tile)
            .add(water_tile)
            .add(sand_deposit_tile)
            .add(sand_tile)
            .add(grass_deposit_tile)
            .add(grass_tile)
            .add(rock_deposit_tile)
            .add(rock_tile)
            .add(dirt_deposit_tile)
            .add(dirt_tile);

        let cache = world.generate(0, 0).unwrap();

        GameMap {
            width,
            height,
            seed: seed.to_string(),
            world,
            cache,
        }
    }

    pub fn seed(&self) -> &str {
        &self.seed
    }

    pub fn world(&self) -> &World<MapTile> {
        &self.world
    }

    pub fn cache(&self) -> &WorldCache {
        &self.cache
    }

    pub fn cache_copy(&self) -> WorldCache {
        WorldCache::from_iter(self.cache.iter().cloned())
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}

pub struct MapController {
    map: GameMap,
    objects: ObjectManager,
    position: Position,
    camera: Position,
    follow: bool,
    home: Option<Position>,
    view_width: u16,
    view_height: u16,
    locations: HashMap<Position, StructureGroup>,
}

impl MapController {
    pub fn new(size: Size, seed: &str) -> MapController {
        let position = Position::new((size.w / 2) as i16, (size.h / 2) as i16);
        let (w, h) = (size.w as u16, size.h as u16);

        let map = GameMap::new(w, h, seed);

        let objects = ObjectManager::new();

        let locations = HashMap::new();

        let mut controller = MapController {
            map,
            objects,
            position: position.clone(),
            camera: Position::new(0, 0),
            follow: true,
            home: None,
            view_width: 1,
            view_height: 1,
            locations,
        };
        controller.center_camera_on_cursor();
        controller
    }

    pub fn seed(&self) -> &str {
        self.map.seed()
    }

    pub fn camera(&self) -> Position {
        self.camera.clone()
    }

    pub fn follow(&self) -> bool {
        self.follow
    }

    pub fn home(&self) -> Option<Position> {
        self.home.clone()
    }

    pub fn view_size(&self) -> (u16, u16) {
        (self.view_width, self.view_height)
    }

    pub fn set_viewport(&mut self, width: u16, height: u16) {
        self.view_width = width.max(1);
        self.view_height = height.max(1);
        if self.follow {
            self.center_camera_on_cursor();
        } else {
            self.clamp_camera();
        }
    }

    pub fn toggle_follow(&mut self) {
        self.follow = !self.follow;
        if self.follow {
            self.center_camera_on_cursor();
        }
    }

    pub fn pin_home(&mut self) {
        self.home = Some(self.position.clone());
    }

    pub fn go_home(&mut self) -> bool {
        if let Some(home) = self.home.clone() {
            self.position = home;
            self.center_camera_on_cursor();
            true
        } else {
            false
        }
    }

    pub fn center_camera_on_cursor(&mut self) {
        let vw = self.view_width.max(1) as i16;
        let vh = self.view_height.max(1) as i16;
        self.camera.x = self.position.x - vw / 2;
        self.camera.y = self.position.y - vh / 2;
        self.clamp_camera();
    }

    fn clamp_camera(&mut self) {
        let max_x = self.map.width().saturating_sub(self.view_width.min(self.map.width())) as i16;
        let max_y = self
            .map
            .height()
            .saturating_sub(self.view_height.min(self.map.height())) as i16;
        if self.camera.x < 0 {
            self.camera.x = 0;
        }
        if self.camera.y < 0 {
            self.camera.y = 0;
        }
        if self.camera.x > max_x {
            self.camera.x = max_x;
        }
        if self.camera.y > max_y {
            self.camera.y = max_y;
        }
    }

    fn after_cursor_move(&mut self) {
        if self.follow {
            self.center_camera_on_cursor();
        }
    }

    pub fn pan_left(&mut self) {
        self.follow = false;
        if self.camera.x > 0 {
            self.camera.x -= 1;
        }
    }

    pub fn pan_right(&mut self) {
        self.follow = false;
        let max_x = self.map.width().saturating_sub(self.view_width.min(self.map.width())) as i16;
        if self.camera.x < max_x {
            self.camera.x += 1;
        }
    }

    pub fn pan_up(&mut self) {
        self.follow = false;
        if self.camera.y > 0 {
            self.camera.y -= 1;
        }
    }

    pub fn pan_down(&mut self) {
        self.follow = false;
        let max_y = self
            .map
            .height()
            .saturating_sub(self.view_height.min(self.map.height())) as i16;
        if self.camera.y < max_y {
            self.camera.y += 1;
        }
    }

    pub fn clear_activity(&mut self) {
        for (_, object) in self.objects.list_mut() {
            object.active = false;
        }
    }

    pub fn tick_construction(&mut self) {
        for (_, object) in self.objects.list_mut() {
            if object.structure.is_some() && object.construction_left > 0 {
                object.construction_left -= 1;
            }
        }
    }

    pub fn map(&self) -> &GameMap {
        &self.map
    }

    pub fn locations(&self) -> &HashMap<Position, StructureGroup> {
        &self.locations
    }

    pub fn objects(&self) -> &ObjectManager {
        &self.objects
    }

    pub fn objects_mut(&mut self) -> &mut ObjectManager {
        &mut self.objects
    }

    pub fn add_object(&mut self, position: Position, object: MapObject) -> Option<MapObject> {
        self.objects.set(position, object)
    }

    pub fn remove_object(&mut self, position: &Position) -> Option<MapObject> {
        self.objects.remove(position)
    }

    pub fn object(&self) -> Option<&MapObject> {
        self.objects.get(&self.position)
    }

    pub fn object_at(&self, position: &Position) -> Option<&MapObject> {
        self.objects.get(position)
    }

    pub fn position(&self) -> Position {
        self.position.clone()
    }

    pub fn tile(&self) -> &MapTile {
        let x = self.position.x as usize;
        let y = self.position.y as usize;

        &self.map.cache[y][x]
    }

    pub fn tile_at(&self, position: &Position) -> &MapTile {
        let x = position.x as usize;
        let y = position.y as usize;

        &self.map.cache[y][x]
    }

    pub fn tile_at_mut(&mut self, position: Position) -> &mut MapTile {
        let x = position.x as usize;
        let y = position.y as usize;

        &mut self.map.cache[y][x]
    }

    pub fn up(&mut self) {
        let y = self.position.y as u16;
        if y > 0 {
            self.position.y((y - 1) as i16);
            self.after_cursor_move();
        }
    }

    pub fn down(&mut self) {
        let y = self.position.y as u16;
        if y < self.map.height() - 1 {
            self.position.y((y + 1) as i16);
            self.after_cursor_move();
        }
    }

    pub fn right(&mut self) {
        let x = self.position.x as u16;
        if x < self.map.width() - 1 {
            self.position.x((x + 1) as i16);
            self.after_cursor_move();
        }
    }

    pub fn left(&mut self) {
        let x = self.position.x as u16;
        if x > 0 {
            self.position.x((x - 1) as i16);
            self.after_cursor_move();
        }
    }

    pub fn add_structure(&mut self, structure: Structure) -> Option<MapObject> {
        let position = self.position();
        let is_base = matches!(structure, Structure::Base { .. });

        if let Some(object) = self.objects.get_mut(&position) {
            if object.structure.is_some() {
                return None;
            }

            object.structure = Option::from(structure);
            object.construction_left = CONSTRUCTION_TICKS;
            object.active = false;
            if is_base && self.home.is_none() {
                self.home = Some(position);
            }
            return None;
        }

        let object = MapObject {
            structure: Option::from(structure),
            deposit: None,
            construction_left: CONSTRUCTION_TICKS,
            active: false,
        };

        if is_base && self.home.is_none() {
            self.home = Some(position.clone());
        }

        self.add_object(position, object)
    }

    pub fn destroy_structure(&mut self) -> bool {
        let position = self.position();
        match self.remove_object(&position) {
            Some(mut object) => {
                if object.structure.is_none() {
                    self.add_object(position, object);
                    return false;
                }
                object.structure = None;
                object.construction_left = 0;
                object.active = false;
                self.add_object(position, object);
                true
            }
            None => false,
        }
    }

    pub fn generate_deposits(&mut self) {
        let cache = self.map().cache_copy();

        for (y, row) in cache.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let mut deposits = match tile.flora {
                    Flora::Water => vec![Resource::Water, Resource::Carbon],
                    Flora::Sand => vec![Resource::Silica],
                    Flora::Dirt => vec![Resource::Iron, Resource::Aluminum],
                    Flora::Grass => vec![Resource::Carbon, Resource::Water],
                    Flora::Rock => vec![Resource::Uranium],
                };

                if tile.is_resource {
                    let resource = ResourceFactory::random_resource(deposits.as_mut());
                    let amount = ResourceFactory::random_resource_amount(resource);
                    // TODO: make amount random in range
                    let deposit = ResourceDeposit::new(resource, amount);

                    let object = MapObject::with_deposit(deposit);

                    self.add_object(Position::new(x as i16, y as i16), object);
                }
            }
        }
    }
}
