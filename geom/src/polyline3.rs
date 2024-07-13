use crate::{vec3, PolyLine, Radians, Segment, Segment3, Vec2, Vec3, AABB3};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::hint::unreachable_unchecked;
use std::ops::{Index, Range};
use std::slice::{Iter, IterMut, SliceIndex, Windows};

/// An ordered list of at least one point forming a broken line
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolyLine3 {
    points: Vec<Vec3>,
    l: f32,
}

impl From<Vec<Vec3>> for PolyLine3 {
    fn from(x: Vec<Vec3>) -> Self {
        Self::new(x)
    }
}

impl PolyLine3 {
    #[inline]
    pub fn new(x: Vec<Vec3>) -> Self {
        if x.is_empty() {
            panic!("Vec must have at least one point")
        }
        Self {
            l: length(&x),
            points: x,
        }
    }

    /// # Safety
    /// Must not be used with advanced functions if passed vector is empty, as it would lead to UB
    #[inline]
    pub unsafe fn new_unchecked(x: Vec<Vec3>) -> Self {
        Self {
            l: length(&x),
            points: x,
        }
    }

    pub fn flatten(&self) -> PolyLine {
        PolyLine::new(self.points.iter().copied().map(Vec3::xy).collect())
    }

    #[inline]
    pub fn clear_push(&mut self, x: Vec3) {
        self.points.clear();
        self.points.push(x);
        self.l = 0.0;
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn clear_extend<A, T>(&mut self, s: T)
    where
        T: IntoIterator<Item = A>,
        Vec<Vec3>: Extend<A>,
    {
        self.points.clear();
        self.points.extend(s);
        self.l = length(&self.points);
        if self.points.is_empty() {
            panic!("cannot have empty polyline3");
        }
    }

    pub fn merge_close(&mut self, dist: f64) {
        let mut last = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        self.points.retain(|x| {
            let v = last.distance(*x) >= dist as f32;
            if v {
                last = *x;
            }
            v
        })
    }

    pub fn middle(&self) -> Vec3 {
        self.point_along(self.length() * 0.5)
    }

    /// Simplifies the polyline by keeping points
    /// where the dot product of the previous and next segment is greater than mindot
    /// mindot should be a bit more than -1
    /// mindist is the minimum distance between two points, below that and the point is removed
    /// maxdist is the maximum distance between two points, above that and the algorithm goes forward.
    ///         Keeping points close enough to avoid precision issues
    /// Complexity is O(N)
    pub fn simplify(&mut self, maxangle: Radians, mindist: f32, maxdist: f32) {
        let mut i = 1;
        let mut j = 1;

        let maxdot = maxangle.cos();

        while i < self.len() {
            if i != j {
                self.points[j] = self[i];
            }
            i += 1;

            if i == self.len() {
                j += 1;
                break;
            }

            let Some(&prev) = self.get(j - 1) else {
                j += 1;
                continue;
            };
            let cur = self[j];
            let Some(&next) = self.get(i) else {
                j += 1;
                continue;
            };

            let prevdiff = prev - cur;
            let prevdist = prevdiff.mag();
            if prevdist <= mindist {
                continue;
            }
            if prevdist >= maxdist {
                j += 1;
                continue;
            }

            let prevdir = prevdiff / prevdist;

            let Some(nextdir) = (next - cur).try_normalize() else {
                j += 1;
                continue;
            };

            if prevdir.dot(nextdir) < -maxdot {
                continue;
            }

            j += 1;
        }

        if j != i {
            self.points.truncate(j);
            self.l = length(&self.points);
        }
    }

    pub fn extend<A, T>(&mut self, s: T)
    where
        T: IntoIterator<Item = A>,
        Vec<Vec3>: Extend<A>,
    {
        let old_l = self.points.len();
        self.points.extend(s);
        self.l += length(&self.points[old_l - 1..]);
    }

    #[inline]
    pub fn pop(&mut self) -> Vec3 {
        let v = match self.points.pop() {
            Some(x) => x,
            None => unsafe { unreachable_unchecked() },
        };
        self.check_empty();
        self.l -= (v - self.last()).mag();
        v
    }

    #[inline]
    pub fn push(&mut self, item: Vec3) {
        self.l += (self.last() - item).mag();
        self.points.push(item);
    }

    #[inline]
    pub fn pop_first(&mut self) -> Vec3 {
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

    pub fn into_vec(self) -> Vec<Vec3> {
        self.points
    }

    /// Distance squared from the projection to p
    pub fn project_dist2(&self, p: Vec3) -> f32 {
        let proj = self.project(p);
        proj.distance2(p)
    }

    /// Distance from the projection to p
    pub fn project_dist(&self, p: Vec3) -> f32 {
        let proj = self.project(p);
        proj.distance(p)
    }

    pub fn segments(&self) -> impl Iterator<Item = Segment3> + '_ {
        self.array_windows::<2>()
            .map(|&[src, dst]| Segment3 { src, dst })
    }

    /// Closest point to p on the polyline using xy distance
    pub fn project(&self, p: Vec3) -> Vec3 {
        self.project_segment(p).0
    }

    pub fn project_segment_dir(&self, p: Vec3) -> (Vec3, usize, Vec3) {
        let (pos, segm) = self.project_segment(p);
        (
            pos,
            segm,
            self.segment_vec(segm - 1)
                .and_then(|x| x.try_normalize())
                .unwrap_or(Vec3::X),
        )
    }

    /// Returns the id of the point right after the projection along with the projection
    /// None if polyline is empty
    pub fn project_segment(&self, p: Vec3) -> (Vec3, usize) {
        match *self.points {
            [] => unsafe { unreachable_unchecked() },
            [p] => (p, 0),
            [src, dst] => (Segment3 { src, dst }.project(p), 1),
            _ => self
                .array_windows::<2>()
                .enumerate()
                .map(|(i, &[a, b])| {
                    let seg = Segment3 { src: a, dst: b };
                    (
                        seg.src + (seg.dst - seg.src) * seg.flatten().project_t(p.xy()),
                        i + 1,
                    )
                })
                .min_by_key(|&(proj, _)| OrderedFloat((p - proj).xy().mag()))
                .unwrap(), // Unwrap ok: n_points > 2
        }
    }

    /// Returns the id of the point right after the projection along with the projection
    /// None if polyline is empty
    pub fn project_segment_2d(&self, p: Vec2) -> (Vec3, usize) {
        match *self.points {
            [] => unsafe { unreachable_unchecked() },
            [p] => (p, 0),
            _ => self
                .array_windows::<2>()
                .enumerate()
                .map(|(i, &[a, b])| {
                    let seg = Segment {
                        src: a.xy(),
                        dst: b.xy(),
                    };
                    (a + (b - a) * seg.project_t(p), i + 1)
                })
                .min_by_key(|&(proj, _)| OrderedFloat((p - proj.xy()).mag()))
                .unwrap(), // Unwrap ok: n_points > 2
        }
    }

    pub fn project_2d(&self, p: Vec2) -> Vec3 {
        self.project_segment_2d(p).0
    }

    pub fn segment_vec(&self, id: usize) -> Option<Vec3> {
        Some(self.get(id + 1)? - self.get(id)?)
    }

    #[inline]
    /// Gives the direction pointing inward the polyline at the first point
    pub fn first_dir(&self) -> Option<Vec3> {
        if self.points.len() >= 2 {
            (self[1] - self[0]).try_normalize()
        } else {
            None
        }
    }

    #[inline]
    /// Gives the direction pointing outward the polyline at the last point
    pub fn last_dir(&self) -> Option<Vec3> {
        let l = self.points.len();
        if l >= 2 {
            (self[l - 1] - self[l - 2]).try_normalize()
        } else {
            None
        }
    }

    pub fn point_along(&self, l: f32) -> Vec3 {
        self.point_dir_along(l).0
    }

    pub fn point_dir_along(&self, l: f32) -> (Vec3, Vec3) {
        self.points_dirs_along(std::iter::once(l))
            .next()
            .unwrap_or((self.last(), self.last_dir().unwrap_or(Vec3::X))) // Unwrap ok: std::iter::once
    }

    pub fn equipoints_dir(&self, d: f32, nolimit: bool) -> impl Iterator<Item = (Vec3, Vec3)> + '_ {
        let l = self.length();
        let n_step = (l / d) as i32;
        let step = l / (n_step as f32 + 1.0);

        self.points_dirs_along(
            (nolimit as i32..n_step.min(100000) + 1)
                .map(move |i| i as f32 * step)
                .chain((!nolimit).then_some(l - 0.01)),
        )
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.l
    }

    /// dists should be in ascending order
    pub fn points_dirs_along<'a>(
        &'a self,
        dists: impl Iterator<Item = f32> + 'a,
    ) -> impl Iterator<Item = (Vec3, Vec3)> + 'a {
        self.points_dirs_manual().into_iter(dists)
    }

    pub fn points_dirs_manual(&self) -> PointsAlongs3<'_> {
        let mut windows = self.points.windows(2);
        let (dir, dist) = windows
            .next()
            .and_then(|w| (w[1] - w[0]).dir_dist())
            .unwrap_or((Vec3::X, 0.0));
        PointsAlongs3 {
            windows,
            lastp: self.first(),
            dir,
            dist,
            partial: 0.0,
        }
    }

    /// Inverse of `point_along`
    /// proj needs to be on the polyline for the result to be accurate
    pub fn length_at_proj(&self, proj: Vec3) -> f32 {
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

    /// Split the polyline3 into two at the given distance from the start
    /// The polylined returned look like this
    /// ([start ... cut], [cut ... end])
    pub fn split(mut self, dst_from_start: f32) -> (Self, Self) {
        let end = self.cut_start(dst_from_start);
        self.points.truncate(self.points.len() - end.n_points() + 1);
        self.points.push(end.first());
        self.l -= end.length();
        (self, end)
    }

    /// dst is distance from start to cut
    /// Returns the end of the points after cutting part of the start
    pub fn cut_start(&self, mut dst: f32) -> Self {
        if dst == 0.0 {
            return self.clone();
        }
        match *self.points {
            [] => unsafe { unreachable_unchecked() },
            [x] => Self::new(vec![x]),
            [f, l] => {
                let v = l - f;
                let m = v.mag();
                dst = dst.min(m);

                Self::new(vec![f + v * (dst / m), l])
            }
            _ => {
                let mut partial = 0.0;
                let mut v = None;
                if dst < f32::EPSILON {
                    v = Some(Self::new(vec![self.first()]));
                }
                for &[a, b] in self.array_windows::<2>() {
                    match v {
                        None => {
                            let d = a.distance(b);

                            if partial + d > dst {
                                let dir = (b - a).normalize();
                                v = Some(Self::new(vec![a + dir * (dst - partial)]));
                            }

                            partial += d;
                        }
                        Some(ref mut v) => {
                            v.push(a);
                        }
                    }
                }
                let mut end_poly = v.unwrap_or_else(|| {
                    Self::new(vec![self.last() - self.last_dir().unwrap() * 0.001])
                });
                end_poly.push(self.last());
                end_poly
            }
        }
    }

    // start is distance from start to cut
    // end is distance from end to cut
    pub fn cut(&self, dst_from_start: f32, dst_from_end: f32) -> Self {
        match self.n_points() {
            0 => unsafe { unreachable_unchecked() },
            1 => self.clone(),
            2 => {
                let n = match self.first_dir() {
                    Some(x) => x,
                    None => return self.clone(),
                };

                Self::new(vec![
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
    pub fn bbox(&self) -> AABB3 {
        let (min, max) = match super::minmax3(self.points.iter().copied()) {
            Some(x) => x,
            None => unsafe { unreachable_unchecked() },
        };

        AABB3::new(min, max)
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
        I: SliceIndex<[Vec3]>,
    {
        self.points.get(index)
    }

    #[inline]
    pub fn first(&self) -> Vec3 {
        unsafe { *self.points.get_unchecked(0) }
    }

    #[inline]
    pub fn last(&self) -> Vec3 {
        unsafe { *self.points.get_unchecked(self.points.len() - 1) }
    }

    pub fn as_slice(&self) -> &[Vec3] {
        self.points.as_slice()
    }

    pub fn iter(&self) -> Iter<'_, Vec3> {
        self.points.iter()
    }

    /// Length must be recalculated after using this function
    pub fn iter_mut_unchecked(&mut self) -> IterMut<'_, Vec3> {
        self.points.iter_mut()
    }

    pub fn recalculate_length(&mut self) {
        self.l = length(&self.points);
    }

    pub fn array_windows<const N: usize>(&self) -> impl Iterator<Item = &[Vec3; N]> + '_ {
        self.points.windows(N).map(|x| x.try_into().unwrap())
    }

    pub fn reserve(&mut self, additional: usize) {
        self.points.reserve(additional);
    }
}

impl Index<Range<usize>> for PolyLine3 {
    type Output = [Vec3];

    fn index(&self, r: Range<usize>) -> &[Vec3] {
        &self.points[r]
    }
}

impl Index<usize> for PolyLine3 {
    type Output = Vec3;

    fn index(&self, index: usize) -> &Vec3 {
        &self.points[index]
    }
}

pub struct PointsAlongs3<'a> {
    windows: Windows<'a, Vec3>,
    lastp: Vec3,
    dir: Vec3,
    dist: f32,
    partial: f32,
}

impl<'a> PointsAlongs3<'a> {
    pub fn next(&mut self, d: f32) -> Option<(Vec3, Vec3)> {
        while d > self.partial + self.dist {
            let w = self.windows.next()?;
            let (dir, dist) = (w[1] - w[0]).dir_dist().unwrap_or((Vec3::X, 0.01));
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
    ) -> impl Iterator<Item = (Vec3, Vec3)> + 'a {
        std::iter::from_fn(move || self.next(it.next()?))
    }
}

fn length(v: &[Vec3]) -> f32 {
    v.windows(2).map(|x| (x[1] - x[0]).mag()).sum()
}

#[cfg(test)]
mod tests {
    use crate::{vec3, Degrees, PolyLine3};

    #[test]
    fn test_simplify() {
        let mut p = PolyLine3::new(vec![
            vec3(0.0, 0.0, 0.0),
            vec3(1.0, 0.0, 0.0),
            vec3(2.0, 0.0, 0.0),
            vec3(3.0, 0.0, 0.0),
            vec3(4.0, 0.0, 0.0),
            vec3(5.0, 1.0, 0.0),
            vec3(5.0, 2.0, 0.0),
            vec3(5.0, 200.0, 0.0),
            vec3(5.0, 300.0, 0.0),
        ]);
        p.simplify(Degrees(10.0).into(), 0.1, 100.0);
        assert_eq!(
            p.into_vec(),
            vec![
                vec3(0.0, 0.0, 0.0),
                vec3(4.0, 0.0, 0.0),
                vec3(5.0, 1.0, 0.0),
                vec3(5.0, 200.0, 0.0),
                vec3(5.0, 300.0, 0.0),
            ]
        );
    }
}
