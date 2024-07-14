#![allow(unused)]
/*
use geom::{Vec2, Vec3};
use lazy_static::lazy_static;
use rerun::{Points2D, Points3D};
use std::sync::Mutex;

lazy_static! {
static ref RERUN: Mutex<Option<rerun::RecordingStream>> = Mutex::new(None);
}

pub fn init_rerun() {
    let rec = rerun::RecordingStreamBuilder::new("rerun_example_dna_abacus")
        .connect()
        .unwrap();
    *RERUN.lock().unwrap() = Some(rec);
}

pub fn rr_poly(iter: impl Iterator<Item=Vec2>) {
    RERUN
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .log("/goria", &Points2D::new(iter.map(|v| [v.x, v.y])))
        .unwrap();
}

pub fn rr_v2(v: Vec2) {
    RERUN
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .log("/goria", &Points2D::new([[v.x, v.y]]))
        .unwrap();
}

pub fn rr_v3(v: Vec3) {
    RERUN
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .log("/goria", &Points3D::new([[v.x, v.y, v.z]]))
        .unwrap();
}
*/
