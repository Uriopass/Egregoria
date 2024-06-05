use super::Vec2;
use crate::aabb::AABB;
use crate::segment::Segment;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::hint::unreachable_unchecked;
use std::ops::{Index, Range};
use std::slice::{Iter, IterMut, SliceIndex, Windows};

/// An ordered list of at least one point forming a broken line
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolyLine {
    points: Vec<Vec2>,
    l: f32,
}

impl From<Vec<Vec2>> for PolyLine {
    fn from(x: Vec<Vec2>) -> Self {
        Self::new(x)
    }
}

impl PolyLine {
    #[inline]
    pub fn new(x: Vec<Vec2>) -> Self {
        if x.is_empty() {
            panic!("Vec must have at least one point")
        }
        Self {
            l: length(&x),
            points: x,
        }
    }

    #[inline]
    pub fn clear_push(&mut self, x: Vec2) {
        self.points.clear();
        self.points.push(x);
        self.l = 0.0;
    }

    pub fn extend<A, T>(&mut self, s: T)
    where
        T: IntoIterator<Item = A>,
        Vec<Vec2>: Extend<A>,
    {
        let old_l = self.points.len();
        self.points.extend(s);
        self.l += length(&self.points[old_l - 1..]);
    }

    #[inline]
    pub fn pop(&mut self) -> Vec2 {
        let v = match self.points.pop() {
            Some(x) => x,
            None => unsafe { unreachable_unchecked() },
        };
        self.check_empty();
        self.l -= (v - self.last()).mag();
        v
    }

    #[inline]
    pub fn push(&mut self, item: Vec2) {
        self.l += (self.last() - item).mag();
        self.points.push(item);
    }

    pub fn pop_first(&mut self) -> Vec2 {
        let v = self.points.remove(0);
        self.check_empty();
        self.l -= (self.first() - v).mag();
        v
    }

    pub fn reverse(&mut self) {
        self.points.reverse()
    }

    fn check_empty(&self) {
        if self.is_empty() {
            panic!("Cannot have empty polyline")
        }
    }

    pub fn into_vec(self) -> Vec<Vec2> {
        self.points
    }

    /// Distance squared from the projection to p
    pub fn project_dist2(&self, p: Vec2) -> f32 {
        let proj = self.project(p);
        proj.distance2(p)
    }

    /// Distance squared from the projection to p
    pub fn project_dist(&self, p: Vec2) -> f32 {
        let proj = self.project(p);
        proj.distance(p)
    }

    pub fn segments(&self) -> impl Iterator<Item = Segment> + '_ {
        self.array_windows::<2>()
            .map(|&[src, dst]| Segment { src, dst })
    }

    /// Closest point to p on the polyline
    pub fn project(&self, p: Vec2) -> Vec2 {
        self.project_segment(p).0
    }

    pub fn project_segment_dir(&self, p: Vec2) -> (Vec2, usize, Vec2) {
        let (pos, segm) = self.project_segment(p);
        (
            pos,
            segm,
            self.segment_vec(segm - 1)
                .and_then(|x| x.try_normalize())
                .unwrap_or(Vec2::X),
        )
    }

    /// Returns the id of the point right after the projection along with the projection
    /// None if polyline is empty
    pub fn project_segment(&self, p: Vec2) -> (Vec2, usize) {
        match self.n_points() {
            0 => unsafe { unreachable_unchecked() },
            1 => (self.first(), 0),
            2 => (
                Segment {
                    src: self.points[0],
                    dst: self.points[1],
                }
                .project(p),
                1,
            ),
            _ => self
                .array_windows::<2>()
                .enumerate()
                .map(|(i, &[a, b])| (Segment { src: a, dst: b }.project(p), i + 1))
                .min_by_key(|&(proj, _)| OrderedFloat((p - proj).mag2()))
                .unwrap(), // Unwrap ok: n_points > 2
        }
    }

    pub fn segment_vec(&self, id: usize) -> Option<Vec2> {
        Some(self.get(id + 1)? - self.get(id)?)
    }

    pub fn first_dir(&self) -> Option<Vec2> {
        if self.points.len() >= 2 {
            (self[1] - self[0]).try_normalize()
        } else {
            None
        }
    }

    pub fn last_dir(&self) -> Option<Vec2> {
        let l = self.points.len();
        if l >= 2 {
            (self[l - 1] - self[l - 2]).try_normalize()
        } else {
            None
        }
    }

    pub fn point_along(&self, l: f32) -> Vec2 {
        self.point_dir_along(l).0
    }

    pub fn point_dir_along(&self, l: f32) -> (Vec2, Vec2) {
        self.points_dirs_along(std::iter::once(l)).next().unwrap() // Unwrap ok: std::iter::once
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.l
    }

    /// dists should be in ascending order
    pub fn points_dirs_along<'a>(
        &'a self,
        dists: impl Iterator<Item = f32> + 'a,
    ) -> impl Iterator<Item = (Vec2, Vec2)> + 'a {
        self.points_dirs_manual().into_iter(dists)
    }

    pub fn points_dirs_manual(&self) -> PointsAlongs<'_> {
        let mut windows = self.points.windows(2);
        let (dir, dist) = windows
            .next()
            .and_then(|w| (w[1] - w[0]).dir_dist())
            .unwrap_or((Vec2::X, 0.0));
        PointsAlongs {
            windows,
            lastp: self.first(),
            dir,
            dist,
            partial: 0.0,
        }
    }

    /// Inverse of `point_along`
    /// proj needs to be on the polyline for the result to be accurate
    pub fn distance_along(&self, proj: Vec2) -> f32 {
        match self.n_points() {
            0 => unsafe { unreachable_unchecked() },
            1 => 0.0,
            2 => self[0].distance(proj),
            _ => {
                let mut partial = 0.0;
                for &[a, b] in self.array_windows::<2>() {
                    let d = a.distance2(b);
                    let d2 = a.distance2(proj);

                    if d2 < d {
                        return partial + d2.sqrt();
                    }

                    partial += d.sqrt();
                }
                partial
            }
        }
    }

    // dst is distance from start to cut
    pub fn cut_start(&self, mut dst: f32) -> PolyLine {
        match *self.points {
            [] => unsafe { unreachable_unchecked() },
            [x] => PolyLine::new(vec![x]),
            [f, l] => {
                let v = l - f;
                let m = v.mag();
                dst = dst.min(m);

                PolyLine::new(vec![f + v * (dst / m), l])
            }
            _ => {
                let mut partial = 0.0;
                let mut v = None;
                if dst < f32::EPSILON {
                    v = Some(PolyLine::new(vec![self.first()]));
                }
                for &[a, b] in self.array_windows::<2>() {
                    match v {
                        None => {
                            let d = a.distance(b);

                            if partial + d > dst {
                                let dir = (b - a).normalize();
                                v = Some(PolyLine::new(vec![a + dir * (dst - partial)]));
                            }

                            partial += d;
                        }
                        Some(ref mut v) => {
                            v.push(a);
                        }
                    }
                }
                let mut end_poly = v.unwrap_or_else(|| {
                    PolyLine::new(vec![self.last() - self.last_dir().unwrap() * 0.001])
                });
                end_poly.push(self.last());
                end_poly
            }
        }
    }

    // start is distance from start to cut
    // end is distance from end to cut
    pub fn cut(&self, dst_from_start: f32, dst_from_end: f32) -> PolyLine {
        match self.n_points() {
            0 => unsafe { unreachable_unchecked() },
            1 => self.clone(),
            2 => {
                let n = self.first_dir().unwrap();
                PolyLine::new(vec![
                    self.first() + n * dst_from_start,
                    self.last() - n * dst_from_end,
                ])
            }
            _ => {
                let mut s_cut = self.cut_start(dst_from_start);
                s_cut.reverse();
                s_cut = s_cut.cut_start(dst_from_end);
                s_cut.reverse();
                s_cut
            }
        }
    }

    #[inline]
    pub fn bbox(&self) -> AABB {
        let (min, max) = match super::minmax(self.points.iter().copied()) {
            Some(x) => x,
            None => unsafe { unreachable_unchecked() },
        };

        AABB::new_ll_ur(min, max)
    }

    #[inline]
    pub fn n_points(&self) -> usize {
        self.points.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[Vec2]>,
    {
        self.points.get(index)
    }

    #[inline]
    pub fn first(&self) -> Vec2 {
        unsafe { *self.points.get_unchecked(0) }
    }

    #[inline]
    pub fn last(&self) -> Vec2 {
        unsafe { *self.points.get_unchecked(self.points.len() - 1) }
    }

    pub fn as_slice(&self) -> &[Vec2] {
        self.points.as_slice()
    }
    pub fn as_mut_slice(&mut self) -> &mut [Vec2] {
        self.points.as_mut_slice()
    }

    pub fn iter(&self) -> Iter<'_, Vec2> {
        self.points.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Vec2> {
        self.points.iter_mut()
    }

    pub fn array_windows<const N: usize>(&self) -> impl Iterator<Item = &[Vec2; N]> + '_ {
        self.points.windows(N).map(|x| x.try_into().unwrap())
    }

    pub fn reserve(&mut self, additional: usize) {
        self.points.reserve(additional);
    }
}

impl Index<Range<usize>> for PolyLine {
    type Output = [Vec2];

    #[inline]
    fn index(&self, r: Range<usize>) -> &[Vec2] {
        &self.points[r]
    }
}

impl Index<usize> for PolyLine {
    type Output = Vec2;

    #[inline]
    fn index(&self, index: usize) -> &Vec2 {
        &self.points[index]
    }
}

pub struct PointsAlongs<'a> {
    windows: Windows<'a, Vec2>,
    lastp: Vec2,
    dir: Vec2,
    dist: f32,
    partial: f32,
}

impl<'a> PointsAlongs<'a> {
    #[inline]
    pub fn next(&mut self, d: f32) -> Option<(Vec2, Vec2)> {
        while d > self.partial + self.dist {
            let w = self.windows.next()?;
            let (dir, dist) = (w[1] - w[0]).dir_dist().unwrap_or((Vec2::X, 0.0));
            self.partial += self.dist;
            self.dir = dir; // no structural assignment :(
            self.dist = dist;
            self.lastp = w[0];
        }
        Some((self.lastp + self.dir * (d - self.partial), self.dir))
    }

    pub fn into_iter<IT: 'a + Iterator<Item = f32>>(
        mut self,
        mut it: IT,
    ) -> impl Iterator<Item = (Vec2, Vec2)> + 'a {
        std::iter::from_fn(move || self.next(it.next()?))
    }
}

fn length(v: &[Vec2]) -> f32 {
    v.windows(2).map(|x| (x[1] - x[0]).mag()).sum()
}
