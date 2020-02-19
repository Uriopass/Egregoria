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

pub fn load_parismap(world: &mut World) {
    let file = File::open("resources/paris_54000.txt").unwrap();
    let mut scanner = Scanner::new(BufReader::new(file));

    let mut map = Map::empty();

    let n = scanner.next::<i32>();
    let m = scanner.next::<i32>();
    let _t = scanner.next::<i32>();
    let _c = scanner.next::<i32>();
    let _s = scanner.next::<i32>();

    let mut ids = vec![];

    const CENTER_A: f64 = 2.3019666000000003;
    const CENTER_B: f64 = 48.8557828;

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
        let a = scanner.next::<usize>();
        let b = scanner.next::<usize>();
        let d = scanner.next::<usize>();
        let _c = scanner.next::<usize>();
        let _l = scanner.next::<usize>();

        map.connect(ids[a], ids[b], 1, d == 1);
    }

    world.insert(map);
}

pub fn load(world: &mut World) {
    load_parismap(world);

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
