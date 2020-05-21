#![allow(clippy::float_cmp)]

/**
 Original author: donbright
 See implementation at https://github.com/donbright/earcutr

 ISC License

 Copyright (c) 2016, Mapbox
 Copyright (c) 2018, Tree Cricket

 Permission to use, copy, modify, and/or distribute this software for any purpose
 with or without fee is hereby granted, provided that the above copyright notice
 and this permission notice appear in all copies.

 THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
 FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS
 OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER
 TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF
 THIS SOFTWARE.
*/

const DIM: usize = 2;

type NodeIdx = usize;
type VertIdx = usize;

#[derive(Clone, Debug)]
struct Node {
    i: VertIdx,         // vertex index in flat one-d array of 64bit float coords
    x: f32,             // vertex x coordinate
    y: f32,             // vertex y coordinate
    prev_idx: NodeIdx,  // previous vertex node in a polygon ring
    next_idx: NodeIdx,  // next vertex node in a polygon ring
    z: i32,             // z-order curve value
    prevz_idx: NodeIdx, // previous node in z-order
    nextz_idx: NodeIdx, // next node in z-order
    steiner: bool,      // indicates whether this is a steiner point
    idx: NodeIdx,       // index within LinkedLists vector that holds all nodes
}

impl Node {
    fn new(i: VertIdx, x: f32, y: f32, idx: NodeIdx) -> Node {
        Node {
            i,
            x,
            y,
            prev_idx: 0,
            next_idx: 0,
            z: 0,
            nextz_idx: 0,
            prevz_idx: 0,
            steiner: false,
            idx,
        }
    }
}

pub struct LinkedLists {
    nodes: Vec<Node>,
    invsize: f32,
    minx: f32,
    miny: f32,
    maxx: f32,
    maxy: f32,
    usehash: bool,
}

macro_rules! noderef {
    ($ll:expr,$idx:expr) => {
        unsafe { $ll.nodes.get_unchecked($idx) }
    };
}
macro_rules! node {
    ($ll:expr,$idx:expr) => {
        unsafe { $ll.nodes.get_unchecked($idx) }
    };
}
macro_rules! nodemut {
    ($ll:expr,$idx:expr) => {
        unsafe { $ll.nodes.get_unchecked_mut($idx) }
    };
}
// Note: none of the following macros work for Left-Hand-Side of assignment.
macro_rules! next {
    ($ll:expr,$idx:expr) => {
        unsafe {
            $ll.nodes
                .get_unchecked($ll.nodes.get_unchecked($idx).next_idx)
        }
    };
}
macro_rules! nextref {
    ($ll:expr,$idx:expr) => {
        unsafe {
            &$ll.nodes
                .get_unchecked($ll.nodes.get_unchecked($idx).next_idx)
        }
    };
}
macro_rules! prev {
    ($ll:expr,$idx:expr) => {
        unsafe {
            $ll.nodes
                .get_unchecked($ll.nodes.get_unchecked($idx).prev_idx)
        }
    };
}
macro_rules! prevref {
    ($ll:expr,$idx:expr) => {
        unsafe {
            &$ll.nodes
                .get_unchecked($ll.nodes.get_unchecked($idx).prev_idx)
        }
    };
}
macro_rules! prevz {
    ($ll:expr,$idx:expr) => {
        &$ll.nodes[$ll.nodes[$idx].prevz_idx]
    };
}

impl LinkedLists {
    fn iter(&self, r: std::ops::Range<NodeIdx>) -> NodeIterator {
        NodeIterator::new(self, r.start, r.end)
    }
    fn iter_pairs(&self, r: std::ops::Range<NodeIdx>) -> NodePairIterator {
        NodePairIterator::new(self, r.start, r.end)
    }
    fn insert_node(&mut self, i: VertIdx, x: f32, y: f32, last: NodeIdx) -> NodeIdx {
        let mut p = Node::new(i, x, y, self.nodes.len());
        if last == 0 {
            p.next_idx = p.idx;
            p.prev_idx = p.idx;
        } else {
            p.next_idx = noderef!(self, last).next_idx;
            p.prev_idx = last;
            let lastnextidx = noderef!(self, last).next_idx;
            nodemut!(self, lastnextidx).prev_idx = p.idx;
            nodemut!(self, last).next_idx = p.idx;
        };
        let id = p.idx;
        self.nodes.push(p);
        id
    }
    fn remove_node(&mut self, p_idx: NodeIdx) {
        let pi = noderef!(self, p_idx).prev_idx;
        let ni = noderef!(self, p_idx).next_idx;
        let pz = noderef!(self, p_idx).prevz_idx;
        let nz = noderef!(self, p_idx).nextz_idx;
        nodemut!(self, pi).next_idx = ni;
        nodemut!(self, ni).prev_idx = pi;
        nodemut!(self, pz).nextz_idx = nz;
        nodemut!(self, nz).prevz_idx = pz;
    }
    fn new(size_hint: usize) -> LinkedLists {
        let mut ll = LinkedLists {
            nodes: Vec::with_capacity(size_hint),
            invsize: 0.0,
            minx: std::f32::MAX,
            miny: std::f32::MAX,
            maxx: std::f32::MIN,
            maxy: std::f32::MIN,
            usehash: true,
        };
        // ll.nodes[0] is the 0 node. For example usage, see remove_node()
        ll.nodes.push(Node {
            i: 0,
            x: 0.0,
            y: 0.0,
            prev_idx: 0,
            next_idx: 0,
            z: 0,
            nextz_idx: 0,
            prevz_idx: 0,
            steiner: false,
            idx: 0,
        });
        ll
    }
}

struct NodeIterator<'a> {
    cur: NodeIdx,
    end: NodeIdx,
    ll: &'a LinkedLists,
    pending_result: Option<&'a Node>,
}

impl<'a> NodeIterator<'a> {
    fn new(ll: &LinkedLists, start: NodeIdx, end: NodeIdx) -> NodeIterator {
        NodeIterator {
            pending_result: Some(noderef!(ll, start)),
            cur: start,
            end,
            ll,
        }
    }
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a Node;
    fn next(&mut self) -> Option<Self::Item> {
        self.cur = noderef!(self.ll, self.cur).next_idx;
        let cur_result = self.pending_result;
        if self.cur == self.end {
            // only one branch, saves time
            self.pending_result = None;
        } else {
            self.pending_result = Some(noderef!(self.ll, self.cur));
        }
        cur_result
    }
}

struct NodePairIterator<'a> {
    cur: NodeIdx,
    end: NodeIdx,
    ll: &'a LinkedLists,
    pending_result: Option<(&'a Node, &'a Node)>,
}

impl<'a> NodePairIterator<'a> {
    fn new(ll: &LinkedLists, start: NodeIdx, end: NodeIdx) -> NodePairIterator {
        NodePairIterator {
            pending_result: Some((noderef!(ll, start), nextref!(ll, start))),
            cur: start,
            end,
            ll,
        }
    }
}

impl<'a> Iterator for NodePairIterator<'a> {
    type Item = (&'a Node, &'a Node);
    fn next(&mut self) -> Option<Self::Item> {
        self.cur = node!(self.ll, self.cur).next_idx;
        let cur_result = self.pending_result;
        if self.cur == self.end {
            // only one branch, saves time
            self.pending_result = None;
        } else {
            self.pending_result = Some((noderef!(self.ll, self.cur), nextref!(self.ll, self.cur)))
        }
        cur_result
    }
}

fn compare_x(a: &Node, b: &Node) -> std::cmp::Ordering {
    a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
}

// minx, miny and invsize are later used to transform coords
// into integers for z-order calculation
fn calc_invsize(minx: f32, miny: f32, maxx: f32, maxy: f32) -> f32 {
    let invsize = f32::max(maxx - minx, maxy - miny);
    if invsize == 0.0 {
        0.0
    } else {
        32767.0 / invsize
    }
}

// main ear slicing loop which triangulates a polygon (given as a linked
// list)
fn earcut_linked_hashed(
    ll: &mut LinkedLists,
    mut ear_idx: NodeIdx,
    on_triangle: &mut impl FnMut(usize, usize, usize) -> (),
    pass: usize,
) {
    // interlink polygon nodes in z-order
    if pass == 0 {
        index_curve(ll, ear_idx);
    }
    // iterate through ears, slicing them one by one
    let mut stop_idx = ear_idx;
    let mut prev_idx = 0;
    let mut next_idx = node!(ll, ear_idx).next_idx;
    while stop_idx != next_idx {
        prev_idx = node!(ll, ear_idx).prev_idx;
        next_idx = node!(ll, ear_idx).next_idx;
        if is_ear_hashed(ll, prev_idx, ear_idx, next_idx) {
            on_triangle(
                node!(ll, prev_idx).i,
                node!(ll, ear_idx).i,
                node!(ll, next_idx).i,
            );
            ll.remove_node(ear_idx);
            // skipping the next vertex leads to less sliver triangles
            ear_idx = node!(ll, next_idx).next_idx;
            stop_idx = ear_idx;
        } else {
            ear_idx = next_idx;
        }
    }

    if prev_idx == next_idx {
        return;
    };
    // if we looped through the whole remaining polygon and can't
    // find any more ears
    if pass == 0 {
        let tmp = filter_points(ll, next_idx, 0);
        earcut_linked_hashed(ll, tmp, on_triangle, 1);
    } else if pass == 1 {
        ear_idx = cure_local_intersections(ll, next_idx, on_triangle);
        earcut_linked_hashed(ll, ear_idx, on_triangle, 2);
    } else if pass == 2 {
        split_earcut(ll, next_idx, on_triangle);
    }
}

// main ear slicing loop which triangulates a polygon (given as a linked
// list)
fn earcut_linked_unhashed(
    ll: &mut LinkedLists,
    mut ear_idx: NodeIdx,
    on_triangle: &mut impl FnMut(usize, usize, usize) -> (),
    pass: usize,
) {
    // iterate through ears, slicing them one by one
    let mut stop_idx = ear_idx;
    let mut prev_idx = 0;
    let mut next_idx = node!(ll, ear_idx).next_idx;
    while stop_idx != next_idx {
        prev_idx = node!(ll, ear_idx).prev_idx;
        next_idx = node!(ll, ear_idx).next_idx;
        if is_ear(ll, prev_idx, ear_idx, next_idx) {
            on_triangle(
                node!(ll, prev_idx).i,
                node!(ll, ear_idx).i,
                node!(ll, next_idx).i,
            );

            ll.remove_node(ear_idx);
            // skipping the next vertex leads to less sliver triangles
            ear_idx = node!(ll, next_idx).next_idx;
            stop_idx = ear_idx;
        } else {
            ear_idx = next_idx;
        }
    }

    if prev_idx == next_idx {
        return;
    };
    // if we looped through the whole remaining polygon and can't
    // find any more ears
    if pass == 0 {
        let tmp = filter_points(ll, next_idx, 0);
        earcut_linked_unhashed(ll, tmp, on_triangle, 1);
    } else if pass == 1 {
        ear_idx = cure_local_intersections(ll, next_idx, on_triangle);
        earcut_linked_unhashed(ll, ear_idx, on_triangle, 2);
    } else if pass == 2 {
        split_earcut(ll, next_idx, on_triangle);
    }
}

// interlink polygon nodes in z-order
fn index_curve(ll: &mut LinkedLists, start: NodeIdx) {
    let invsize = ll.invsize;
    let mut p = start;
    loop {
        if node!(ll, p).z == 0 {
            nodemut!(ll, p).z = zorder(node!(ll, p).x, node!(ll, p).y, invsize);
        }
        nodemut!(ll, p).prevz_idx = node!(ll, p).prev_idx;
        nodemut!(ll, p).nextz_idx = node!(ll, p).next_idx;
        p = node!(ll, p).next_idx;
        if p == start {
            break;
        }
    }

    let pzi = prevz!(ll, start).idx;
    nodemut!(ll, pzi).nextz_idx = 0;
    nodemut!(ll, start).prevz_idx = 0;
    sort_linked(ll, start);
}

// Simon Tatham's linked list merge sort algorithm
// http://www.chiark.greenend.org.uk/~sgtatham/algorithms/listsort.html
fn sort_linked(ll: &mut LinkedLists, mut list: NodeIdx) {
    let mut p;
    let mut q;
    let mut e;
    let mut nummerges;
    let mut psize;
    let mut qsize;
    let mut insize = 1;
    let mut tail;

    loop {
        p = list;
        list = 0;
        tail = 0;
        nummerges = 0;

        while p != 0 {
            nummerges += 1;
            q = p;
            psize = 0;
            while q != 0 && psize < insize {
                psize += 1;
                q = noderef!(ll, q).nextz_idx;
            }
            qsize = insize;

            while psize > 0 || (qsize > 0 && q != 0) {
                if psize > 0 && (qsize == 0 || q == 0 || noderef!(ll, p).z <= noderef!(ll, q).z) {
                    e = p;
                    p = noderef!(ll, p).nextz_idx;
                    psize -= 1;
                } else {
                    e = q;
                    q = noderef!(ll, q).nextz_idx;
                    qsize -= 1;
                }

                if tail != 0 {
                    nodemut!(ll, tail).nextz_idx = e;
                } else {
                    list = e;
                }

                nodemut!(ll, e).prevz_idx = tail;
                tail = e;
            }

            p = q;
        }

        nodemut!(ll, tail).nextz_idx = 0;
        insize *= 2;
        if nummerges <= 1 {
            break;
        }
    }
}

// check whether a polygon node forms a valid ear with adjacent nodes
fn is_ear(ll: &LinkedLists, prev: NodeIdx, ear: NodeIdx, next: NodeIdx) -> bool {
    let (a, b, c) = (noderef!(ll, prev), noderef!(ll, ear), noderef!(ll, next));
    if area(a, b, c) >= 0.0 {
        false // reflex, cant be ear
    } else {
        !ll.iter(c.next_idx..a.idx).any(|p| {
            point_in_triangle(&a, &b, &c, &p)
                && (area(prevref!(ll, p.idx), &p, nextref!(ll, p.idx)) >= 0.0)
        })
    }
}

// helper for is_ear_hashed. needs manual inline (rust 2018)
#[inline(always)]
fn earcheck(a: &Node, b: &Node, c: &Node, prev: &Node, p: &Node, next: &Node) -> bool {
    (p.idx != a.idx)
        && (p.idx != c.idx)
        && point_in_triangle(&a, &b, &c, &p)
        && area(&prev, &p, &next) >= 0.0
}

#[inline(always)]
fn is_ear_hashed(
    ll: &mut LinkedLists,
    prev_idx: NodeIdx,
    ear_idx: NodeIdx,
    next_idx: NodeIdx,
) -> bool {
    let (prev, ear, next) = (
        &node!(ll, prev_idx).clone(),
        &node!(ll, ear_idx).clone(),
        &node!(ll, next_idx).clone(),
    );
    if area(prev, ear, next) >= 0.0 {
        return false;
    };

    let bbox_maxx = f32::max(prev.x, f32::max(ear.x, next.x));
    let bbox_maxy = f32::max(prev.y, f32::max(ear.y, next.y));
    let bbox_minx = f32::min(prev.x, f32::min(ear.x, next.x));
    let bbox_miny = f32::min(prev.y, f32::min(ear.y, next.y));
    // z-order range for the current triangle bbox;
    let min_z = zorder(bbox_minx, bbox_miny, ll.invsize);
    let max_z = zorder(bbox_maxx, bbox_maxy, ll.invsize);

    let mut p = ear.prevz_idx;
    let mut n = ear.nextz_idx;
    while (p != 0) && (node!(ll, p).z >= min_z) && (n != 0) && (node!(ll, n).z <= max_z) {
        if earcheck(
            prev,
            ear,
            next,
            prevref!(ll, p),
            noderef!(ll, p),
            nextref!(ll, p),
        ) {
            return false;
        }
        p = node!(ll, p).prevz_idx;

        if earcheck(
            prev,
            ear,
            next,
            prevref!(ll, n),
            noderef!(ll, n),
            nextref!(ll, n),
        ) {
            return false;
        }
        n = node!(ll, n).nextz_idx;
    }

    nodemut!(ll, 0).z = min_z - 1;
    while node!(ll, p).z >= min_z {
        if earcheck(
            prev,
            ear,
            next,
            prevref!(ll, p),
            noderef!(ll, p),
            nextref!(ll, p),
        ) {
            return false;
        }
        p = node!(ll, p).prevz_idx;
    }

    nodemut!(ll, 0).z = max_z + 1;
    while node!(ll, n).z <= max_z {
        if earcheck(
            prev,
            ear,
            next,
            prevref!(ll, n),
            noderef!(ll, n),
            nextref!(ll, n),
        ) {
            return false;
        }
        n = node!(ll, n).nextz_idx;
    }

    true
}

fn filter_points(ll: &mut LinkedLists, start: NodeIdx, mut end: NodeIdx) -> NodeIdx {
    if end == 0 {
        end = start;
    }
    if end >= ll.nodes.len() || start >= ll.nodes.len() {
        return 0;
    }

    let mut p = start;
    let mut again;

    // this loop "wastes" calculations by going over the same points multiple
    // times. however, altering the location of the 'end' node can disrupt
    // the algorithm of other code that calls the filter_points function.
    loop {
        again = false;
        if !node!(ll, p).steiner
            && (equals(noderef!(ll, p), nextref!(ll, p))
                || area(prevref!(ll, p), noderef!(ll, p), nextref!(ll, p)) == 0.0)
        {
            ll.remove_node(p);
            end = node!(ll, p).prev_idx;
            p = end;
            if p == node!(ll, p).next_idx {
                break end;
            }
            again = true;
        } else {
            p = node!(ll, p).next_idx;
        }
        if !again && p == end {
            break end;
        }
    }
}

// create a circular doubly linked list from polygon points in the
// specified winding order
fn linked_list(data: &[f32], start: usize, end: usize, clockwise: bool) -> (LinkedLists, NodeIdx) {
    let mut ll: LinkedLists = LinkedLists::new(data.len() / DIM);
    if data.len() < 80 {
        ll.usehash = false
    };
    let (last_idx, _) = linked_list_add_contour(&mut ll, data, start, end, clockwise);
    (ll, last_idx)
}

// add new nodes to an existing linked list.
fn linked_list_add_contour(
    ll: &mut LinkedLists,
    data: &[f32],
    start: usize,
    end: usize,
    clockwise: bool,
) -> (NodeIdx, NodeIdx) {
    if start > data.len() || end > data.len() || data.is_empty() {
        return (0, 0);
    }
    let mut lastidx = 0;
    let mut leftmost_idx = 0;
    let mut contour_minx = std::f32::MAX;

    if clockwise == (signed_area(&data, start, end) > 0.0) {
        for i in (start..end).step_by(DIM) {
            lastidx = ll.insert_node(i / DIM, data[i], data[i + 1], lastidx);
            if contour_minx > data[i] {
                contour_minx = data[i];
                leftmost_idx = lastidx
            };
            if ll.usehash {
                ll.miny = f32::min(data[i + 1], ll.miny);
                ll.maxx = f32::max(data[i], ll.maxx);
                ll.maxy = f32::max(data[i + 1], ll.maxy);
            }
        }
    } else {
        for i in (start..=(end - DIM)).rev().step_by(DIM) {
            lastidx = ll.insert_node(i / DIM, data[i], data[i + 1], lastidx);
            if contour_minx > data[i] {
                contour_minx = data[i];
                leftmost_idx = lastidx
            };
            if ll.usehash {
                ll.miny = f32::min(data[i + 1], ll.miny);
                ll.maxx = f32::max(data[i], ll.maxx);
                ll.maxy = f32::max(data[i + 1], ll.maxy);
            }
        }
    }

    ll.minx = f32::min(contour_minx, ll.minx);

    if equals(noderef!(ll, lastidx), nextref!(ll, lastidx)) {
        ll.remove_node(lastidx);
        lastidx = noderef!(ll, lastidx).next_idx;
    }
    (lastidx, leftmost_idx)
}

// z-order of a point given coords and inverse of the longer side of
// data bbox
#[inline(always)]
fn zorder(xf: f32, yf: f32, invsize: f32) -> i32 {
    // coords are transformed into non-negative 15-bit integer range
    // stored in two 32bit ints, which are combined into a single 64 bit int.
    let x: i64 = (xf * invsize) as i64;
    let y: i64 = (yf * invsize) as i64;
    let mut xy: i64 = x << 32 | y;

    // todo ... big endian?
    xy = (xy | (xy << 8)) & 0x00FF00FF00FF00FF;
    xy = (xy | (xy << 4)) & 0x0F0F0F0F0F0F0F0F;
    xy = (xy | (xy << 2)) & 0x3333333333333333;
    xy = (xy | (xy << 1)) & 0x5555555555555555;

    ((xy >> 32) | (xy << 1)) as i32
}

// check if a point lies within a convex triangle
fn point_in_triangle(a: &Node, b: &Node, c: &Node, p: &Node) -> bool {
    ((c.x - p.x) * (a.y - p.y) - (a.x - p.x) * (c.y - p.y) >= 0.0)
        && ((a.x - p.x) * (b.y - p.y) - (b.x - p.x) * (a.y - p.y) >= 0.0)
        && ((b.x - p.x) * (c.y - p.y) - (c.x - p.x) * (b.y - p.y) >= 0.0)
}

pub fn earcut(data: &[f32], mut on_triangle: impl FnMut(usize, usize, usize) -> ()) {
    let (mut ll, outer_node) = linked_list(data, 0, data.len(), true);
    if ll.nodes.len() == 1 {
        return;
    }

    if ll.usehash {
        ll.invsize = calc_invsize(ll.minx, ll.miny, ll.maxx, ll.maxy);

        // translate all points so min is 0,0. prevents subtraction inside
        // zorder. also note invsize does not depend on translation in space
        // if one were translating in a space with an even spaced grid of points.
        // floating point space is not evenly spaced, but it is close enough for
        // this hash algorithm
        let (mx, my) = (ll.minx, ll.miny);
        ll.nodes.iter_mut().for_each(|n| n.x -= mx);
        ll.nodes.iter_mut().for_each(|n| n.y -= my);
        earcut_linked_hashed(&mut ll, outer_node, &mut on_triangle, 0);
    } else {
        earcut_linked_unhashed(&mut ll, outer_node, &mut on_triangle, 0);
    }
}

// signed area of a parallelogram
fn area(p: &Node, q: &Node, r: &Node) -> f32 {
    (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y)
}

// check if two points are equal
fn equals(p1: &Node, p2: &Node) -> bool {
    p1.x == p2.x && p1.y == p2.y
}

/* go through all polygon nodes and cure small local self-intersections
what is a small local self-intersection? well, lets say you have four points
a,b,c,d. now imagine you have three line segments, a-b, b-c, and c-d. now
imagine two of those segments overlap each other. thats an intersection. so
this will remove one of those nodes so there is no more overlap.

but theres another important aspect of this function. it will dump triangles
into the 'triangles' variable, thus this is part of the triangulation
algorithm itself.*/
fn cure_local_intersections(
    ll: &mut LinkedLists,
    instart: NodeIdx,
    on_triangle: &mut impl FnMut(usize, usize, usize) -> (),
) -> NodeIdx {
    let mut p = instart;
    let mut start = instart;

    //        2--3  4--5 << 2-3 + 4-5 pseudointersects
    //           x  x
    //  0  1  2  3  4  5  6  7
    //  a  p  pn b
    //              eq     a      b
    //              psi    a p pn b
    //              li  pa a p pn b bn
    //              tp     a p    b
    //              rn       p pn
    //              nst    a      p pn b
    //                            st

    //
    //                            a p  pn b

    loop {
        let a = node!(ll, p).prev_idx;
        let b = next!(ll, p).next_idx;

        if !equals(noderef!(ll, a), noderef!(ll, b))
            && pseudo_intersects(
            noderef!(ll, a),
            noderef!(ll, p),
            nextref!(ll, p),
            noderef!(ll, b),
        )
            // prev next a, prev next b
            && locally_inside(ll, noderef!(ll, a), noderef!(ll, b))
            && locally_inside(ll, noderef!(ll, b), noderef!(ll, a))
        {
            on_triangle(noderef!(ll, a).i, noderef!(ll, p).i, noderef!(ll, b).i);

            // remove two nodes involved
            ll.remove_node(p);
            let nidx = noderef!(ll, p).next_idx;
            ll.remove_node(nidx);

            start = noderef!(ll, b).idx;
            p = start;
        }
        p = noderef!(ll, p).next_idx;
        if p == start {
            break;
        }
    }

    p
}

// try splitting polygon into two and triangulate them independently
fn split_earcut(
    ll: &mut LinkedLists,
    start_idx: NodeIdx,
    on_triangle: &mut impl FnMut(usize, usize, usize) -> (),
) {
    // look for a valid diagonal that divides the polygon into two
    let mut a = start_idx;
    loop {
        let mut b = next!(ll, a).next_idx;
        while b != noderef!(ll, a).prev_idx {
            if noderef!(ll, a).i != noderef!(ll, b).i
                && is_valid_diagonal(ll, noderef!(ll, a), noderef!(ll, b))
            {
                // split the polygon in two by the diagonal
                let mut c = split_bridge_polygon(ll, a, b);

                // filter colinear points around the cuts
                let an = noderef!(ll, a).next_idx;
                let cn = noderef!(ll, c).next_idx;
                a = filter_points(ll, a, an);
                c = filter_points(ll, c, cn);

                // run earcut on each half
                earcut_linked_hashed(ll, a, on_triangle, 0);
                earcut_linked_hashed(ll, c, on_triangle, 0);
                return;
            }
            b = noderef!(ll, b).next_idx;
        }
        a = noderef!(ll, a).next_idx;
        if a == start_idx {
            break;
        }
    }
}

// check if a diagonal between two polygon nodes is valid (lies in
// polygon interior)
fn is_valid_diagonal(ll: &LinkedLists, a: &Node, b: &Node) -> bool {
    next!(ll, a.idx).i != b.i
        && prev!(ll, a.idx).i != b.i
        && !intersects_polygon(ll, a, b)
        && locally_inside(ll, a, b)
        && locally_inside(ll, b, a)
        && middle_inside(ll, a, b)
}

/* check if two segments cross over each other. note this is different
from pure intersction. only two segments crossing over at some interior
point is considered intersection.

line segment p1-q1 vs line segment p2-q2.

note that if they are collinear, or if the end points touch, or if
one touches the other at one point, it is not considered an intersection.

please note that the other algorithms in this earcut code depend on this
interpretation of the concept of intersection - if this is modified
so that endpoint touching qualifies as intersection, then it will have
a problem with certain inputs.

bsed on https://www.geeksforgeeks.org/check-if-two-given-line-segments-intersect/

this has been modified from the version in earcut.js to remove the
detection for endpoint detection.

    a1=area(p1,q1,p2);a2=area(p1,q1,q2);a3=area(p2,q2,p1);a4=area(p2,q2,q1);
    p1 q1    a1 cw   a2 cw   a3 ccw   a4  ccw  a1==a2  a3==a4  fl
    p2 q2
    p1 p2    a1 ccw  a2 ccw  a3 cw    a4  cw   a1==a2  a3==a4  fl
    q1 q2
    p1 q2    a1 ccw  a2 ccw  a3 ccw   a4  ccw  a1==a2  a3==a4  fl
    q1 p2
    p1 q2    a1 cw   a2 ccw  a3 ccw   a4  cw   a1!=a2  a3!=a4  tr
    p2 q1
*/

fn pseudo_intersects(p1: &Node, q1: &Node, p2: &Node, q2: &Node) -> bool {
    if (equals(p1, p2) && equals(q1, q2)) || (equals(p1, q2) && equals(q1, p2)) {
        return true;
    }
    (area(p1, q1, p2) > 0.0) != (area(p1, q1, q2) > 0.0)
        && (area(p2, q2, p1) > 0.0) != (area(p2, q2, q1) > 0.0)
}

// check if a polygon diagonal intersects any polygon segments
fn intersects_polygon(ll: &LinkedLists, a: &Node, b: &Node) -> bool {
    ll.iter_pairs(a.idx..a.idx).any(|(p, n)| {
        p.i != a.i && n.i != a.i && p.i != b.i && n.i != b.i && pseudo_intersects(&p, &n, a, b)
    })
}

// check if a polygon diagonal is locally inside the polygon
fn locally_inside(ll: &LinkedLists, a: &Node, b: &Node) -> bool {
    if area(prevref!(ll, a.idx), a, nextref!(ll, a.idx)) < 0.0 {
        area(a, b, nextref!(ll, a.idx)) >= 0.0 && area(a, prevref!(ll, a.idx), b) >= 0.0
    } else {
        area(a, b, prevref!(ll, a.idx)) < 0.0 || area(a, nextref!(ll, a.idx), b) < 0.0
    }
}

// check if the middle point of a polygon diagonal is inside the polygon
fn middle_inside(ll: &LinkedLists, a: &Node, b: &Node) -> bool {
    let (mx, my) = ((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    ll.iter_pairs(a.idx..a.idx)
        .filter(|(p, n)| (p.y > my) != (n.y > my))
        .filter(|(p, n)| n.y != p.y)
        .filter(|(p, n)| (mx) < ((n.x - p.x) * (my - p.y) / (n.y - p.y) + p.x))
        .fold(false, |inside, _| !inside)
}

/* link two polygon vertices with a bridge;

if the vertices belong to the same linked list, this splits the list
into two new lists, representing two new polygons.

if the vertices belong to separate linked lists, it merges them into a
single linked list.

For example imagine 6 points, labeled with numbers 0 thru 5, in a single cycle.
Now split at points 1 and 4. The 2 new polygon cycles will be like this:
0 1 4 5 0 1 ...  and  1 2 3 4 1 2 3 .... However because we are using linked
lists of nodes, there will be two new nodes, copies of points 1 and 4. So:
the new cycles will be through nodes 0 1 4 5 0 1 ... and 2 3 6 7 2 3 6 7 .

splitting algorithm:

.0...1...2...3...4...5...     6     7
5p1 0a2 1m3 2n4 3b5 4q0      .c.   .d.

an<-2     an = a.next,
bp<-3     bp = b.prev;
1.n<-4    a.next = b;
4.p<-1    b.prev = a;
6.n<-2    c.next = an;
2.p<-6    an.prev = c;
7.n<-6    d.next = c;
6.p<-7    c.prev = d;
3.n<-7    bp.next = d;
7.p<-3    d.prev = bp;

result of split:
<0...1> <2...3> <4...5>      <6....7>
5p1 0a4 6m3 2n7 1b5 4q0      7c2  3d6
      x x     x x            x x  x x    // x shows links changed

a b q p a b q p  // begin at a, go next (new cycle 1)
a p q b a p q b  // begin at a, go prev (new cycle 1)
m n d c m n d c  // begin at m, go next (new cycle 2)
m c d n m c d n  // begin at m, go prev (new cycle 2)

Now imagine that we have two cycles, and
they are 0 1 2, and 3 4 5. Split at points 1 and
4 will result in a single, long cycle,
0 1 4 5 3 7 6 2 0 1 4 5 ..., where 6 and 1 have the
same x y f32s, as do 7 and 4.

 0...1...2   3...4...5        6     7
2p1 0a2 1m0 5n4 3b5 4q3      .c.   .d.

an<-2     an = a.next,
bp<-3     bp = b.prev;
1.n<-4    a.next = b;
4.p<-1    b.prev = a;
6.n<-2    c.next = an;
2.p<-6    an.prev = c;
7.n<-6    d.next = c;
6.p<-7    c.prev = d;
3.n<-7    bp.next = d;
7.p<-3    d.prev = bp;

result of split:
 0...1...2   3...4...5        6.....7
2p1 0a4 6m0 5n7 1b5 4q3      7c2   3d6
      x x     x x            x x   x x

a b q n d c m p a b q n d c m .. // begin at a, go next
a p m c d n q b a p m c d n q .. // begin at a, go prev

Return value.

Return value is the new node, at point 7.
*/
fn split_bridge_polygon(ll: &mut LinkedLists, a: NodeIdx, b: NodeIdx) -> NodeIdx {
    let cidx = ll.nodes.len();
    let didx = cidx + 1;
    let mut c = Node::new(
        noderef!(ll, a).i,
        noderef!(ll, a).x,
        noderef!(ll, a).y,
        cidx,
    );
    let mut d = Node::new(
        noderef!(ll, b).i,
        noderef!(ll, b).x,
        noderef!(ll, b).y,
        didx,
    );

    let an = noderef!(ll, a).next_idx;
    let bp = noderef!(ll, b).prev_idx;

    nodemut!(ll, a).next_idx = b;
    nodemut!(ll, b).prev_idx = a;

    c.next_idx = an;
    nodemut!(ll, an).prev_idx = cidx;

    d.next_idx = cidx;
    c.prev_idx = didx;

    nodemut!(ll, bp).next_idx = didx;
    d.prev_idx = bp;

    ll.nodes.push(c);
    ll.nodes.push(d);
    didx
}

fn signed_area(data: &[f32], start: usize, end: usize) -> f32 {
    let i = (start..end).step_by(DIM);
    let j = (start..end).cycle().skip((end - DIM) - start).step_by(DIM);
    i.zip(j).fold(0., |s, (i, j)| {
        s + (data[j] - data[i]) * (data[i + 1] + data[j + 1])
    })
}
