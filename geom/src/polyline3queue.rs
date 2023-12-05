use crate::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Polyline3Queue is used to make entity follow the same path as another entity, mostly for trains, efficiently.
/// It is a queue of points that were at some point the position of the leader.
/// The follower update can be done in parallel, the idea is to split the leader update and the followers update with a
/// data driven approach.
/// When the leader moves, the queue is updated. Then the followers check what changed to update themselves.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Polyline3Queue {
    /// Past points of the leader
    data: VecDeque<Vec3>,
    /// curlength is the length of the data polyline up to the tail
    curlength: f32,
    /// maxlength is the maximum allowed length of the polyline until it is cleaned to avoid taking too much space
    maxlength: f32,
    /// frontcounter is the counter of the foremost point
    frontcounter: usize,
    /// dh (distance-head) is the distance between the leader and the foremost point (foremost point being behind the head)
    dh: f32,
    /// head is the position of the leader
    head: Vec3,
}

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Follower {
    /// counter is the counter of the follower to know at which data point it is, relative to the queue's frontcounter
    counter: usize,
    /// headcounter is the last known frontcounter of the queue, used to know when points where added to it
    headcounter: usize,
    /// Distance from the next point from counter to the foremost point of the queue
    df: f32,
    /// Distance to head, should stay constant
    dh: f32,
}

impl Polyline3Queue {
    pub fn new(past: impl Iterator<Item = Vec3>, head: Vec3, maxlength: f32) -> Self {
        let mut dist = 0.0;
        let mut last: Option<Vec3> = None;
        let data: VecDeque<_> = past
            .map(|x| {
                match last {
                    None => {}
                    Some(v) => {
                        dist += v.distance(x);
                    }
                };
                last = Some(x);
                x
            })
            .collect();
        let mut s = Self {
            curlength: dist,
            maxlength,
            frontcounter: data.len(),
            dh: head.distance(*data.front().unwrap()),
            head,
            data,
        };
        s.ensure_length();
        s
    }

    /// The foremost point of the queue
    pub fn front(&self) -> Vec3 {
        *self.data.front().unwrap()
    }

    pub fn push(&mut self, v: Vec3) {
        if v.is_close(self.front(), 0.2) {
            self.update_head(v);
            return;
        }
        if self.data.len() >= 2 {
            if let Some(behind) = (self.data[0] - self.data[1]).try_normalize() {
                if let Some(headdir) = (v - self.data[0]).try_normalize() {
                    if behind.dot(headdir) > 0.99999 {
                        self.update_head(v);
                        return;
                    }
                }
            }
        }
        self.real_push(v)
    }

    fn update_head(&mut self, v: Vec3) {
        self.head = v;
        self.dh = self.head.distance(self.front());
    }

    fn real_push(&mut self, v: Vec3) {
        self.head = v;
        self.dh = 0.0;

        let d = self.front().distance(self.head);
        self.data.push_front(self.head);
        self.curlength += d;
        self.frontcounter += 1;

        self.ensure_length();
    }

    fn ensure_length(&mut self) {
        while self.data.len() > 2 {
            let seglen = self.segmentlen(self.data.len() - 2);
            if self.curlength <= self.maxlength + seglen + self.dh {
                return;
            }
            self.data.pop_back();
            self.curlength -= seglen;
        }
    }

    fn segmentlen(&self, front: usize) -> f32 {
        self.data
            .get(front)
            .unwrap()
            .distance(*self.data.get(1 + front).unwrap())
    }

    pub fn mk_followers<'a>(
        &'a self,
        distances: impl Iterator<Item = f32> + 'a,
    ) -> impl Iterator<Item = Follower> + 'a {
        let mut counter = self.frontcounter;
        let mut accdist = self.dh;
        let mut lastp = self.front();

        distances.into_iter().map_while(move |dist| {
            loop {
                let Some(newp) = self.data.get(self.frontcounter - counter).copied() else {
                    return None;
                };
                let d = lastp.distance(newp);
                if accdist + d >= dist {
                    break;
                }
                accdist += d;
                lastp = newp;
                counter -= 1;
            }

            Some(Follower {
                counter,
                headcounter: self.frontcounter,
                df: accdist - self.dh,
                dh: dist,
            })
        })
    }
}

impl Follower {
    pub fn update(&mut self, queue: &Polyline3Queue) -> (Vec3, Vec3) {
        while self.headcounter < queue.frontcounter {
            self.headcounter += 1;
            if self.headcounter == self.counter + 1 {
                continue;
            }
            self.df += queue.segmentlen(queue.frontcounter - self.headcounter);
        }
        while queue.frontcounter > self.counter && queue.dh + self.df >= self.dh {
            self.counter += 1;
            if queue.frontcounter == self.counter {
                continue;
            }
            self.df -= queue.segmentlen(queue.frontcounter - self.counter - 1);
        }

        let dist_to_counter;
        let cback;
        let cfront;

        if queue.frontcounter == self.counter {
            cfront = queue.head;
            cback = queue.front();
            dist_to_counter = self.dh;
        } else {
            dist_to_counter = self.dh - queue.dh - self.df;
            cfront = *queue
                .data
                .get(queue.frontcounter - self.counter - 1)
                .unwrap();
            cback = *queue.data.get(queue.frontcounter - self.counter).unwrap();
        }
        let norm = (cback - cfront).normalize();

        (cfront + norm * dist_to_counter, -norm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_delta;

    fn v(x: f32) -> Vec3 {
        Vec3::x(x)
    }

    #[test]
    fn queue_new() {
        let p = vec![v(2.0), v(1.0), v(0.0)];

        let queue = Polyline3Queue::new(p.clone().into_iter(), v(2.5), 10.0);

        assert_eq!(
            queue,
            Polyline3Queue {
                data: p.into_iter().collect(),
                curlength: 2.0,
                maxlength: 10.0,
                frontcounter: 3,
                dh: 0.5,
                head: v(2.5)
            }
        );
        assert_eq!(queue.front(), v(2.0));
        assert_eq!(queue.segmentlen(0), 1.0);
        assert_eq!(queue.segmentlen(1), 1.0);
    }

    #[test]
    fn queue_push_close() {
        let p = vec![v(2.0), v(1.0), v(0.0)];
        let mut queue = Polyline3Queue::new(p.clone().into_iter(), v(2.05), 10.0);
        queue.push(v(2.1));

        assert_eq!(queue.curlength, 2.0);
        assert_eq!(queue.frontcounter, 3);
        assert_delta!(queue.dh, 0.1, 0.001);
    }

    #[test]
    fn queue_real_push() {
        let p = vec![v(2.0), v(1.0), v(0.0)];
        let mut queue = Polyline3Queue::new(p.clone().into_iter(), v(2.05), 10.0);
        queue.real_push(v(2.24));

        assert_delta!(queue.curlength, 2.24, 0.001);
        assert_eq!(queue.frontcounter, 4);
        assert!(queue.front().is_close(v(2.24), 0.001));
        assert_delta!(queue.dh, 0.0, 0.001);
    }

    #[test]
    fn queue_push_angle() {
        let p = vec![v(2.0), v(1.0), v(0.0)];
        let mut queue = Polyline3Queue::new(p.clone().into_iter(), v(2.05), 10.0);
        queue.push(Vec3::new(3.0, 1.0, 0.0));

        assert_eq!(queue.frontcounter, 4);
        assert_delta!(queue.dh, 0.0, 0.001);
    }

    #[test]
    fn queue_ensure_length() {
        let p = vec![v(2.0), v(1.0), v(0.0)];
        let mut queue = Polyline3Queue::new(p.clone().into_iter(), v(2.05), 10.0);
        queue.real_push(v(10.5));
        queue.ensure_length();
        assert_delta!(queue.curlength, 9.5, 0.001);
        assert_eq!(queue.frontcounter, 4);
        assert!(queue.front().is_close(v(10.5), 0.001));
        assert_delta!(queue.dh, 0.0, 0.001);
    }

    #[test]
    fn mk_followers() {
        let p = vec![v(3.0), v(1.0), v(0.0)];
        let queue = Polyline3Queue::new(p.clone().into_iter(), v(3.25), 10.0);
        let mut followers =
            queue.mk_followers(vec![0.0, 0.125, 0.125, 1.0, 1.5, 3.0, 3.125, 100.0].into_iter());
        assert_eq!(
            followers.next().unwrap(),
            Follower {
                counter: 3,
                headcounter: 3,
                df: 0.0,
                dh: 0.0,
            }
        );
        assert_eq!(
            followers.next().unwrap(),
            Follower {
                counter: 3,
                headcounter: 3,
                df: 0.0,
                dh: 0.125,
            }
        );
        assert_eq!(
            followers.next().unwrap(),
            Follower {
                counter: 3,
                headcounter: 3,
                df: 0.0,
                dh: 0.125,
            }
        );
        assert_eq!(
            followers.next().unwrap(),
            Follower {
                counter: 2,
                headcounter: 3,
                df: 0.0,
                dh: 1.0,
            }
        );
        assert_eq!(
            followers.next().unwrap(),
            Follower {
                counter: 2,
                headcounter: 3,
                df: 0.0,
                dh: 1.5,
            }
        );
        assert_eq!(
            followers.next().unwrap(),
            Follower {
                counter: 1,
                headcounter: 3,
                df: 2.0,
                dh: 3.0,
            }
        );
        assert_eq!(
            followers.next().unwrap(),
            Follower {
                counter: 1,
                headcounter: 3,
                df: 2.0,
                dh: 3.125,
            }
        );
        assert_eq!(followers.next(), None);
    }
}
