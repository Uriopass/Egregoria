#![allow(clippy::indexing_slicing)]

use crate::map::{IntersectionID, LanePatternBuilder, Map, RoadSegmentKind};
use common::FastMap;
use flat_spatial::Grid;
use geom::{vec2, vec3, Vec2};
use std::io::{BufRead, Cursor};

struct Scanner<T> {
    buffer: Vec<String>,
    reader: T,
}

impl<T> Scanner<T> {
    pub fn new(reader: T) -> Self {
        Self {
            buffer: vec![],
            reader,
        }
    }
}

impl<R: BufRead> Scanner<R> {
    fn next<T: std::str::FromStr>(&mut self) -> T {
        loop {
            if let Some(token) = self.buffer.pop() {
                return token.parse().ok().expect("Failed parse");
            }
            let mut input = String::new();
            self.reader.read_line(&mut input).expect("Failed read");
            self.buffer = input.split_whitespace().rev().map(String::from).collect();
        }
    }
}

static PARISMAP_STR: &str = include_str!("../../../../assets/paris_54000.txt");

pub fn load_parismap(map: &mut Map) {
    let time = std::time::Instant::now();

    let mut scanner = Scanner::new(Cursor::new(PARISMAP_STR));

    let n_inters = scanner.next::<i32>();
    let n_roads = scanner.next::<i32>();
    let _ = scanner.next::<i32>();
    let _ = scanner.next::<i32>();
    let _ = scanner.next::<i32>();

    let mut ids = vec![];

    const CENTER_A: f64 = 2.301_966_6;
    const CENTER_B: f64 = 48.855_782_8;

    //Scale nodes
    let scale: f64 = 90000.0;

    let mut g = Grid::new(50);

    for _ in 0..n_inters {
        let mut long = scanner.next::<f64>();
        let mut lat = scanner.next::<f64>();

        long = (long - CENTER_B) * scale / f64::cos(long / 180.0 * std::f64::consts::PI);
        lat = (lat - CENTER_A) * scale;

        let pos = vec2(lat as f32, long as f32) + vec2(12000.0, 8000.0);

        let n = g.query_around(pos, 50.0).next();
        if let Some((h, _)) = n {
            let (_, close_id) = g.get(h).unwrap();
            ids.push(*close_id);
            let newpos = (map.intersections[*close_id].pos.xy() + pos) * 0.5;
            map.intersections[*close_id].pos = newpos.z(0.3);
            g.set_position(h, newpos);
            g.maintain();
            continue;
        }
        let id = map.add_intersection(pos.z(0.3));
        ids.push(id);
        g.insert(pos, id);
    }

    let mut edges = FastMap::default();

    //Parse junctions
    for _ in 0..n_roads {
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

        let mi = src.min(dst);
        let ma = src.max(dst);

        let v: &mut (bool, bool) = edges.entry((mi, ma)).or_default();
        v.0 |= mi == src || (n_lanes != 1);
        v.1 |= mi == dst || (n_lanes != 1);
    }

    let mut edges_keys: Vec<_> = edges.keys().copied().collect();
    edges_keys.sort_unstable();

    for (src, dst) in edges_keys {
        let (fw, bw) = edges[&(src, dst)];
        if !fw && !bw {
            continue;
        }
        let one_way = fw && bw;
        let (src, dst) = if fw { (src, dst) } else { (dst, src) };
        map.connect(
            src,
            dst,
            &LanePatternBuilder::new()
                .one_way(one_way)
                .parking(true)
                .build(),
            RoadSegmentKind::Straight,
        )
        .unwrap();
    }

    info!(
        "loading parismap took {}ms",
        time.elapsed().as_secs_f32() * 1000.0
    );

    map.check_invariants();

    print_stats(map);
}

pub fn add_doublecircle(pos: Vec2, m: &mut Map) {
    let mut first_circle = vec![];
    let mut second_circle = vec![];

    const N_POINTS: usize = 20;
    for i in 0..N_POINTS {
        let angle = (i as f32 / N_POINTS as f32) * 2.0 * std::f32::consts::PI;

        let v: Vec2 = [angle.cos(), angle.sin()].into();
        first_circle.push(m.add_intersection((pos + v * 200.0).z(0.3)));
        second_circle.push(m.add_intersection((pos + v * 300.0).z(0.3)));
    }

    for x in first_circle.windows(2) {
        m.connect(
            x[0],
            x[1],
            &LanePatternBuilder::new()
                .one_way(true)
                .parking(false)
                .build(),
            RoadSegmentKind::Straight,
        );
    }
    m.connect(
        *first_circle.last().unwrap(), // Unwrap ok: n_points > 0
        first_circle[0],
        &LanePatternBuilder::new().one_way(true).build(),
        RoadSegmentKind::Straight,
    );

    for x in second_circle.windows(2) {
        m.connect(
            x[1],
            x[0],
            &LanePatternBuilder::new()
                .one_way(true)
                .parking(false)
                .build(),
            RoadSegmentKind::Straight,
        );
    }
    m.connect(
        second_circle[0],
        *second_circle.last().unwrap(), // Unwrap ok: n_points > 0
        &LanePatternBuilder::new().one_way(true).build(),
        RoadSegmentKind::Straight,
    );

    for (a, b) in first_circle.into_iter().zip(second_circle) {
        m.connect(
            a,
            b,
            &LanePatternBuilder::new().build(),
            RoadSegmentKind::Straight,
        );
    }
}

pub fn add_grid(m: &mut Map, pos: Vec2, size: u32, spacing: f32) {
    if size == 0 {
        return;
    }
    let pos = pos - Vec2::splat(size as f32 * spacing * 0.5);
    let size = size as usize;
    let mut grid: Vec<Vec<IntersectionID>> = vec![vec![]; size];
    for (y, l) in grid.iter_mut().enumerate() {
        for x in 0..size {
            l.push(
                m.add_intersection(pos.z0() + vec3(x as f32 * spacing, y as f32 * spacing, 0.3)),
            );
        }
    }

    let pat = LanePatternBuilder::new().build();
    let l = size - 1;
    for x in 0..l {
        m.connect(grid[l][x], grid[l][x + 1], &pat, RoadSegmentKind::Straight);
        m.connect(grid[x][l], grid[x + 1][l], &pat, RoadSegmentKind::Straight);

        for y in 0..l {
            m.connect(grid[y][x], grid[y][x + 1], &pat, RoadSegmentKind::Straight);
            m.connect(grid[y][x], grid[y + 1][x], &pat, RoadSegmentKind::Straight);
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

pub fn load_testfield(map: &mut Map, pos: Vec2, size: u32, spacing: f32) {
    //add_doublecircle([0.0, 0.0].into(), map);
    add_grid(map, pos, size, spacing);
    map.check_invariants();
    print_stats(map);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn testfield_valid() {
        let mut m = Map::empty();
        load_testfield(&mut m, Vec2::ZERO, 10, 100.0);
        m.check_invariants();
    }

    #[test]
    fn parismap_valid() {
        let mut m = Map::empty();
        load_parismap(&mut m);
        m.check_invariants();
    }
}
