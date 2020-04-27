use crate::geometry::Vec2;
use crate::map_model::{make_inter_entity, IntersectionID, LanePatternBuilder, Map};
use cgmath::num_traits::FloatConst;
use specs::{LazyUpdate, World, WorldExt};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;

const FILENAME: &str = "world/map.bc";

pub fn save(world: &mut World) {
    let _ = std::fs::create_dir("world");

    let map = world.read_resource::<Map>();

    let file = File::create(FILENAME).unwrap();

    bincode::serialize_into(file, map.deref()).unwrap();
}

fn load_from_file() -> Map {
    let file = File::open(FILENAME);
    if let Err(e) = file {
        println!("error while trying to load map: {}", e);
        return Map::empty();
    }

    let des = bincode::deserialize_from(file.unwrap());
    des.unwrap_or_else(|_| Map::empty())
}

struct Scanner {
    buffer: Vec<String>,
    file: BufReader<File>,
}

impl Scanner {
    pub fn new(file: BufReader<File>) -> Self {
        Self {
            buffer: vec![],
            file,
        }
    }
}

impl Scanner {
    fn next<T: std::str::FromStr>(&mut self) -> T {
        loop {
            if let Some(token) = self.buffer.pop() {
                return token.parse().ok().expect("Failed parse");
            }
            let mut input = String::new();
            self.file.read_line(&mut input).expect("Failed read");
            self.buffer = input.split_whitespace().rev().map(String::from).collect();
        }
    }
}

pub fn load_parismap() -> Map {
    let file = File::open("resources/paris_54000.txt").unwrap();
    let mut scanner = Scanner::new(BufReader::new(file));

    let mut map = Map::empty();

    let n = scanner.next::<i32>();
    let m = scanner.next::<i32>();
    let _ = scanner.next::<i32>();
    let _ = scanner.next::<i32>();
    let _ = scanner.next::<i32>();

    let mut ids = vec![];

    const CENTER_A: f64 = 2.301_966_6;
    const CENTER_B: f64 = 48.855_782_8;

    //Scale nodes
    let scale: f64 = 60000.0;

    for _ in 0..n {
        let mut long = scanner.next::<f64>();
        let mut lat = scanner.next::<f64>();

        long = (long - CENTER_B) * scale / f64::cos(long / 180.0 * f64::PI());
        lat = (lat - CENTER_A) * scale;

        ids.push(map.add_intersection(vec2!(lat as f32, long as f32)));
    }

    //Parse junctions
    for _ in 0..m {
        let src = scanner.next::<usize>();
        let dst = scanner.next::<usize>();
        let n_lanes = scanner.next::<usize>();
        let _ = scanner.next::<usize>();
        let _ = scanner.next::<usize>();

        map.connect(
            ids[src],
            ids[dst],
            &LanePatternBuilder::new().one_way(n_lanes == 1).build(),
        );
    }

    map
}

pub fn add_doublecircle(pos: Vec2, m: &mut Map) {
    let mut first_circle = vec![];
    let mut second_circle = vec![];

    const N_POINTS: usize = 20;
    for i in 0..N_POINTS {
        let angle = (i as f32 / N_POINTS as f32) * 2.0 * std::f32::consts::PI;

        let v: Vec2 = [angle.cos(), angle.sin()].into();
        first_circle.push(m.add_intersection(pos + v * 100.0));
        second_circle.push(m.add_intersection(pos + v * 200.0));
    }

    for x in first_circle.windows(2) {
        m.connect(x[0], x[1], &LanePatternBuilder::new().one_way(true).build());
    }
    m.connect(
        *first_circle.last().unwrap(),
        first_circle[0],
        &LanePatternBuilder::new().one_way(true).build(),
    );

    for x in second_circle.windows(2) {
        m.connect(x[1], x[0], &LanePatternBuilder::new().one_way(true).build());
    }
    m.connect(
        second_circle[0],
        *second_circle.last().unwrap(),
        &LanePatternBuilder::new().one_way(true).build(),
    );

    for (a, b) in first_circle.into_iter().zip(second_circle) {
        m.connect(a, b, &LanePatternBuilder::new().build());
    }
}

pub fn add_grid(pos: Vec2, m: &mut Map) {
    let mut grid: [[Option<IntersectionID>; 10]; 10] = [[None; 10]; 10];
    for (y, l) in grid.iter_mut().enumerate() {
        for (x, v) in l.iter_mut().enumerate() {
            *v = Some(m.add_intersection(pos + vec2!(x as f32 * 70.0, y as f32 * 70.0)));
        }
    }

    for x in 0..9 {
        m.connect(
            grid[9][x].unwrap(),
            grid[9][x + 1].unwrap(),
            &LanePatternBuilder::new().build(),
        );
        m.connect(
            grid[x][9].unwrap(),
            grid[x + 1][9].unwrap(),
            &LanePatternBuilder::new().build(),
        );

        for y in 0..9 {
            m.connect(
                grid[y][x].unwrap(),
                grid[y][x + 1].unwrap(),
                &LanePatternBuilder::new().build(),
            );
            m.connect(
                grid[y][x].unwrap(),
                grid[y + 1][x].unwrap(),
                &LanePatternBuilder::new().build(),
            );
        }
    }
}

pub fn load(world: &mut World) {
    let mut map = load_from_file();

    // map = load_parismap();

    if map.is_empty() {
        add_doublecircle([0.0, 0.0].into(), &mut map);
        add_grid([0.0, 250.0].into(), &mut map);
    }

    world.insert(map);

    let map = world.read_resource::<Map>();

    for (_, inter) in map.intersections() {
        make_inter_entity(
            inter,
            &world.read_resource::<LazyUpdate>(),
            &world.entities(),
            &world.read_resource::<Map>(),
        );
    }
}
