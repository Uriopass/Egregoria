use crate::map_model::{make_inter_entity, Map};
use cgmath::num_traits::FloatConst;
use cgmath::Vector2;
use specs::{LazyUpdate, World, WorldExt};
use std::fs::File;
use std::io::{BufRead, BufReader};

//const GRAPH_FILENAME: &str = "world/graph";

pub fn save(_world: &mut World) {
    //world.read_resource::<NavMesh>().save(GRAPH_FILENAME);
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

        ids.push(map.add_intersection(Vector2::new(lat as f32, long as f32)));
    }

    //Parse junctions
    for _ in 0..m {
        let src = scanner.next::<usize>();
        let dst = scanner.next::<usize>();
        let n_lanes = scanner.next::<usize>();
        let _ = scanner.next::<usize>();
        let _ = scanner.next::<usize>();

        map.connect(ids[src], ids[dst], 1, n_lanes == 1);
    }

    map
}

pub fn load_doublecircle() -> Map {
    let mut m = Map::empty();

    let mut first_circle = vec![];
    let mut second_circle = vec![];

    for i in 0..20 {
        let angle = (i as f32 / 30.0) * 2.0 * std::f32::consts::PI;

        let v: Vector2<f32> = [angle.cos(), angle.sin()].into();
        first_circle.push(m.add_intersection(v * 100.0));
        second_circle.push(m.add_intersection(v * 200.0));
    }

    for x in first_circle.windows(2) {
        m.connect(x[0], x[1], 1, true);
    }
    m.connect(*first_circle.last().unwrap(), first_circle[0], 1, true);

    for x in second_circle.windows(2) {
        m.connect(x[0], x[1], 1, true);
    }
    m.connect(*second_circle.last().unwrap(), second_circle[0], 1, true);

    for (a, b) in first_circle.into_iter().zip(second_circle) {
        m.connect(a, b, 1, false);
    }

    m
}

pub fn load(world: &mut World) {
    let map = load_doublecircle();

    world.insert(map);
    let map = world.read_resource::<Map>();

    for (_, inter) in &map.intersections {
        make_inter_entity(
            inter,
            inter.pos,
            &world.read_resource::<LazyUpdate>(),
            &world.entities(),
        );
    }
}
