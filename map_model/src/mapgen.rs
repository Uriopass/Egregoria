use crate::{IntersectionID, LanePatternBuilder, Map};
use flat_spatial::SparseGrid;
use geom::{vec2, Vec2};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

pub fn load_parismap(map: &mut Map) {
    let t = std::time::Instant::now();
    let file = unwrap_or!(File::open("resources/paris_54000.txt").ok(), {
        error!("Couldn't open parismap file");
        return;
    });

    let mut scanner = Scanner::new(BufReader::new(file));

    let n = scanner.next::<i32>();
    let m = scanner.next::<i32>();
    let _ = scanner.next::<i32>();
    let _ = scanner.next::<i32>();
    let _ = scanner.next::<i32>();

    let mut ids = vec![];

    const CENTER_A: f64 = 2.301_966_6;
    const CENTER_B: f64 = 48.855_782_8;

    //Scale nodes
    let scale: f64 = 90000.0;

    let mut g = SparseGrid::new(50);

    for _ in 0..n {
        let mut long = scanner.next::<f64>();
        let mut lat = scanner.next::<f64>();

        long = (long - CENTER_B) * scale / f64::cos(long / 180.0 * std::f64::consts::PI);
        lat = (lat - CENTER_A) * scale;

        let p = vec2(lat as f32, long as f32);

        let n = g.query_around(p, 50.0).next();
        if let Some((h, _)) = n {
            let (_, close_id) = g.get(h).unwrap();
            ids.push(*close_id);
            let newpos = (map.intersections[*close_id].pos + p) * 0.5;
            map.update_intersection(*close_id, |i| i.pos = newpos);
            g.set_position(h, newpos);
            g.maintain();
            continue;
        }
        let id = map.add_intersection(p);
        ids.push(id);
        g.insert(p, id);
    }

    let mut already = HashSet::new();
    //Parse junctions
    for _ in 0..m {
        let src = scanner.next::<usize>();
        let dst = scanner.next::<usize>();
        let n_lanes = scanner.next::<usize>();
        let _ = scanner.next::<usize>();
        let _ = scanner.next::<usize>();

        let src = ids[src];
        let dst = ids[dst];
        if src == dst {
            continue;
        }

        if already.contains(&(src, dst)) {
            continue;
        }
        already.insert((src, dst));
        if already.contains(&(dst, src)) {
            map.remove_road(map.find_road(dst, src).unwrap());
            map.connect_straight(
                src,
                dst,
                &LanePatternBuilder::new()
                    .one_way(false)
                    .parking(true)
                    .build(),
            );
            continue;
        }
        map.connect_straight(
            src,
            dst,
            &LanePatternBuilder::new()
                .one_way(n_lanes == 1)
                .parking(true)
                .build(),
        );
        if n_lanes != 1 {
            already.insert((dst, src));
        }
    }

    info!(
        "loading parismap took {}ms",
        t.elapsed().as_secs_f32() * 1000.0
    );

    print_stats(map);
}

pub fn add_doublecircle(pos: Vec2, m: &mut Map) {
    let mut first_circle = vec![];
    let mut second_circle = vec![];

    const N_POINTS: usize = 20;
    for i in 0..N_POINTS {
        let angle = (i as f32 / N_POINTS as f32) * 2.0 * std::f32::consts::PI;

        let v: Vec2 = [angle.cos(), angle.sin()].into();
        first_circle.push(m.add_intersection(pos + v * 200.0));
        second_circle.push(m.add_intersection(pos + v * 300.0));
    }

    for x in first_circle.windows(2) {
        m.connect_straight(
            x[0],
            x[1],
            &LanePatternBuilder::new()
                .one_way(true)
                .parking(false)
                .build(),
        );
    }
    m.connect_straight(
        *first_circle.last().unwrap(), // Unwrap ok: n_points > 0
        first_circle[0],
        &LanePatternBuilder::new().one_way(true).build(),
    );

    for x in second_circle.windows(2) {
        m.connect_straight(
            x[1],
            x[0],
            &LanePatternBuilder::new()
                .one_way(true)
                .parking(false)
                .build(),
        );
    }
    m.connect_straight(
        second_circle[0],
        *second_circle.last().unwrap(), // Unwrap ok: n_points > 0
        &LanePatternBuilder::new().one_way(true).build(),
    );

    for (a, b) in first_circle.into_iter().zip(second_circle) {
        m.connect_straight(a, b, &LanePatternBuilder::new().build());
    }
}

pub fn add_grid(pos: Vec2, m: &mut Map, size: usize) {
    if size == 0 {
        return;
    }
    let mut grid: Vec<Vec<IntersectionID>> = vec![vec![]; size];
    for (y, l) in grid.iter_mut().enumerate() {
        for x in 0..size {
            l.push(m.add_intersection(pos + vec2(x as f32 * 100.0, y as f32 * 100.0)));
        }
    }

    let pat = LanePatternBuilder::new().build();
    for x in 0..size - 1 {
        m.connect_straight(grid[9][x], grid[9][x + 1], &pat);
        m.connect_straight(grid[x][9], grid[x + 1][9], &pat);

        for y in 0..size - 1 {
            m.connect_straight(grid[y][x], grid[y][x + 1], &pat);
            m.connect_straight(grid[y][x], grid[y + 1][x], &pat);
        }
    }
}

fn print_stats(map: &Map) {
    info!("{} intersections", map.intersections.len());
    info!("{} roads", map.roads.len());
    info!("{} lanes", map.lanes.len());
    info!(
        "{} turns",
        map.intersections
            .iter()
            .map(|(_, x)| x.turns().len())
            .sum::<usize>()
    );
}

pub fn load_testfield(map: &mut Map) {
    add_doublecircle([0.0, 0.0].into(), map);
    add_grid([0.0, 350.0].into(), map, 10);
}
