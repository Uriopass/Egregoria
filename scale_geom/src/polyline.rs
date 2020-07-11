use super::Vec2;
use crate::segment::Segment;
use imgui_inspect::imgui::im_str;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::hint::unreachable_unchecked;
use std::ops::{Index, RangeBounds};
use std::slice::{Iter, IterMut, Windows};

/// An ordered list of at least one point forming a broken line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyLine(Vec<Vec2>);

impl From<Vec<Vec2>> for PolyLine {
    fn from(x: Vec<Vec2>) -> Self {
        Self::new(x)
    }
}

impl PolyLine {
    /// # Safety
    /// Must have at least one element, if the vec is empty then things like first() or last() might behave very badly.
    pub unsafe fn new_unchecked(x: Vec<Vec2>) -> Self {
        Self(x)
    }

    pub fn new(x: Vec<Vec2>) -> Self {
        if x.is_empty() {
            panic!("Vec must have at least one point")
        }
        Self(x)
    }

    pub fn clear_push(&mut self, x: Vec2) {
        self.0.clear();
        self.0.push(x)
    }

    pub fn into_vec(self) -> Vec<Vec2> {
        self.0
    }

    pub fn extend<A, T>(&mut self, s: T)
    where
        T: IntoIterator<Item = A>,
        Vec<Vec2>: Extend<A>,
    {
        self.0.extend(s);
    }

    pub fn pop(&mut self) -> Vec2 {
        let v = match self.0.pop() {
            Some(x) => x,
            None => unsafe { unreachable_unchecked() },
        };
        self.check_empty();
        v
    }

    pub fn push(&mut self, item: Vec2) {
        self.0.push(item)
    }

    pub fn pop_first(&mut self) -> Vec2 {
        let v = self.0.remove(0);
        self.check_empty();
        v
    }

    pub fn reverse(&mut self) {
        self.0.reverse()
    }

    fn check_empty(&self) {
        if self.is_empty() {
            panic!("Cannot have empty polyline")
        }
    }

    pub fn drain(&mut self, r: impl RangeBounds<usize>) {
        self.0.drain(r);
        self.check_empty()
    }

    pub fn project(&self, p: Vec2) -> Vec2 {
        self.project_segment(p).0
    }

    pub fn project_dir(&self, p: Vec2) -> (Vec2, Vec2) {
        let (pos, segm) = self.project_segment(p);
        (
            pos,
            self.segment_vec(segm)
                .and_then(|x| x.try_normalize())
                .unwrap_or(Vec2::UNIT_X),
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
                    src: self.0[0],
                    dst: self.0[1],
                }
                .project(p),
                1,
            ),
            _ => self
                .0
                .windows(2)
                .enumerate()
                .map(|(i, w)| {
                    if let [a, b] = *w {
                        (Segment { src: a, dst: b }.project(p), i + 1)
                    } else {
                        unsafe { unreachable_unchecked() } // windows(2)
                    }
                })
                .min_by_key(|&(proj, _)| OrderedFloat((p - proj).magnitude2()))
                .unwrap(), // Unwrap ok: n_points > 2
        }
    }

    pub fn segment_vec(&self, id: usize) -> Option<Vec2> {
        Some(self.get(id + 1)? - self.get(id)?)
    }

    pub fn first_dir(&self) -> Option<Vec2> {
        if self.0.len() >= 2 {
            (self[1] - self[0]).try_normalize()
        } else {
            None
        }
    }

    pub fn last_dir(&self) -> Option<Vec2> {
        let l = self.0.len();
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

    /// dists should be in ascending order
    pub fn points_dirs_along<T>(&self, dists: T) -> PointsAlongs<T>
    where
        T: Iterator<Item = f32>,
    {
        let mut windows = self.0.windows(2);
        let (dir, dist) = windows
            .next()
            .and_then(|w| (w[1] - w[0]).dir_dist())
            .unwrap_or((Vec2::UNIT_X, 0.0));
        PointsAlongs {
            dists,
            windows,
            lastp: self.first(),
            dir,
            dist,
            partial: 0.0,
        }
    }

    /// Inverse of point_along
    /// proj needs to be on the polyline for the result to be accurate
    pub fn distance_along(&self, proj: Vec2) -> f32 {
        match self.n_points() {
            0 => unsafe { unreachable_unchecked() },
            1 => 0.0,
            2 => self[0].distance(proj),
            _ => {
                let mut partial = 0.0;
                for w in self.0.windows(2) {
                    let d = w[0].distance2(w[1]);
                    let d2 = w[0].distance2(proj);

                    if d2 < d {
                        return partial + d2.sqrt();
                    }

                    partial += d.sqrt();
                }
                partial
            }
        }
    }

    pub fn length(&self) -> f32 {
        self.0.windows(2).map(|x| (x[1] - x[0]).magnitude()).sum()
    }

    pub fn n_points(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, id: usize) -> Option<&Vec2> {
        self.0.get(id)
    }

    pub fn first(&self) -> Vec2 {
        unsafe { *self.0.get_unchecked(0) }
    }

    pub fn last(&self) -> Vec2 {
        unsafe { *self.0.get_unchecked(self.0.len() - 1) }
    }

    pub fn as_slice(&self) -> &[Vec2] {
        self.0.as_slice()
    }

    pub fn iter(&self) -> Iter<Vec2> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Vec2> {
        self.0.iter_mut()
    }

    pub fn windows(&self, id: usize) -> Windows<'_, Vec2> {
        self.0.windows(id)
    }

    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }
}

impl Index<usize> for PolyLine {
    type Output = Vec2;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

pub struct PointsAlongs<'a, T: Iterator<Item = f32>> {
    dists: T,
    windows: Windows<'a, Vec2>,
    lastp: Vec2,
    dir: Vec2,
    dist: f32,
    partial: f32,
}

impl<T: Iterator<Item = f32>> Iterator for PointsAlongs<'_, T> {
    type Item = (Vec2, Vec2);

    fn next(&mut self) -> Option<Self::Item> {
        let d = self.dists.next()?;
        while d > self.partial + self.dist {
            let w = match self.windows.next() {
                Some(x) => x,
                None => break,
            };
            let (dir, dist) = (w[1] - w[0]).dir_dist().unwrap_or((Vec2::UNIT_X, 0.0));
            self.partial += self.dist;
            self.dir = dir; // no structural assignment :(
            self.dist = dist;
            self.lastp = w[0];
        }
        Some((self.lastp + self.dir * (d - self.partial), self.dir))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.dists.size_hint()
    }
}

impl imgui_inspect::InspectRenderDefault<PolyLine> for PolyLine {
    fn render(
        _data: &[&PolyLine],
        _label: &'static str,
        _: &mut imgui_inspect::specs::World,
        _ui: &imgui_inspect::imgui::Ui,
        _args: &imgui_inspect::InspectArgsDefault,
    ) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut PolyLine],
        label: &str,
        w: &mut imgui_inspect::specs::World,
        ui: &imgui_inspect::imgui::Ui,
        args: &imgui_inspect::InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = &mut data[0];
        let mut changed = false;

        if imgui_inspect::imgui::CollapsingHeader::new(&im_str!("{}", label)).build(&ui) {
            ui.indent();
            for (i, x) in v.iter_mut().enumerate() {
                let id = ui.push_id(i as i32);
                changed |= <Vec2 as imgui_inspect::InspectRenderDefault<Vec2>>::render_mut(
                    &mut [x],
                    "",
                    w,
                    ui,
                    args,
                );
                id.pop(ui);
            }
            ui.unindent();
        }

        changed
    }
}
