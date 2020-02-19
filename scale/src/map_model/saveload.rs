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

    let mut min_a = 1000.0;
    let mut min_b = 1000.0;
    let mut max_a = -1000.0;

    for _ in 0..n {
        let long = scanner.next::<f32>();
        let lat = scanner.next::<f32>();
        min_a = f32::min(min_a, lat);
        min_b = f32::min(min_b, long);

        max_a = f32::max(max_a, lat);

        ids.push(map.add_intersection(Vector2::new(lat, long)));
    }

    //Scale nodes
    let scale = 30000.0 / (max_a - min_a);

    let mut max_y = 0.0;
    for (_, inter) in &mut map.intersections {
        inter.pos.x = (inter.pos.x - min_a) * scale;
        inter.pos.y = (inter.pos.y - min_b) * scale / f32::cos(min_b / 180.0 * f32::PI());
        max_y = f32::max(inter.pos.y, max_y);
    }

    //Parse junctions
    for _ in 0..m {
        let a = scanner.next::<usize>();
        let b = scanner.next::<usize>();
        let d = scanner.next::<usize>();
        let _c = scanner.next::<usize>();
        let _l = scanner.next::<usize>();

        map.connect(ids[a], ids[b], 1);
        if d == 2 {
            // two way ?
        }
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
