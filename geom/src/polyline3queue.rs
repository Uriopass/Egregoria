use crate::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Serialize, Deserialize)]
pub struct Polyline3Queue {
    data: VecDeque<Vec3>,
    curlength: f32,
    maxlength: f32,
    frontcounter: usize,
    dh: f32,
    head: Vec3,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Follower {
    counter: usize,
    headcounter: usize,
    /// Distance to front
    df: f32,
    /// Distance to head
    dh: f32,
}

impl Polyline3Queue {
    pub fn new(past: impl Iterator<Item = Vec3>, head: Vec3, maxlength: f32) -> Self {
        let mut dist = 0.0;
        let mut first: Option<Vec3> = None;
        let data: VecDeque<_> = past
            .map(|x| {
                match first {
                    None => {}
                    Some(v) => {
                        dist += v.distance(x);
                    }
                };
                first = Some(x);
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

    pub fn front(&self) -> Vec3 {
        *self.data.front().unwrap()
    }

    pub fn push(&mut self, v: Vec3) {
        if v.is_close(self.head, 0.2) {
            self.update_front(v);
            return;
        }
        if v.is_close(self.head, 5.0) {
            self.real_push(v);
            return;
        }
        if self.data.len() >= 2 {
            if let Some(behind) = (self.data[0] - self.data[1]).try_normalize() {
                if let Some(headdir) = (self.head - self.data[0]).try_normalize() {
                    if behind.dot(headdir) > 0.99999 {
                        self.update_front(v);
                        return;
                    }
                }
            }
        }
        self.real_push(v)
    }

    fn update_front(&mut self, v: Vec3) {
        self.head = v;
        self.dh = self.head.distance(self.front());
    }

    fn real_push(&mut self, v: Vec3) {
        let topush = self.head;
        self.head = v;
        self.dh = v.distance(topush);

        let d = self.front().distance(topush);
        self.data.push_front(topush);
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

        distances.into_iter().map(move |dist| {
            if dist < self.dh {
                return Follower {
                    counter,
                    headcounter: self.frontcounter,
                    df: 0.0,
                    dh: dist,
                };
            }

            loop {
                counter -= 1;
                let newp = *self.data.get(self.frontcounter - counter).unwrap();
                let d = lastp.distance(newp);
                if accdist + d >= dist {
                    break;
                }
                accdist += d;
                lastp = newp;
            }

            Follower {
                counter,
                headcounter: self.frontcounter,
                df: accdist - self.dh,
                dh: dist,
            }
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
