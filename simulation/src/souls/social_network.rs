use crate::utils::resources::Resources;
use crate::{HumanID, World};
use common::scroll::BTreeMapScroller;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Bound;

#[derive(Default, Serialize, Deserialize)]
pub struct SocialNetworkRes {
    // Symmetric
    kinship: BTreeMap<(HumanID, HumanID), f32>,

    social_maker_pivot: Option<HumanID>,

    cleaner: BTreeMapScroller<(HumanID, HumanID)>,
}

impl SocialNetworkRes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_kinship(&mut self, a: HumanID, b: HumanID, value: f32) {
        self.kinship.insert((a, b), value);
        self.kinship.insert((b, a), value);
    }

    /// If kinship is not set, returns 0.0
    pub fn kinship(&self, a: HumanID, b: HumanID) -> f32 {
        self.kinship.get(&(a, b)).copied().unwrap_or(0.0)
    }

    /// may return kinships to nonexistent humans
    pub fn nonzero_kinships(&self, a: HumanID) -> impl Iterator<Item = (HumanID, f32)> + '_ {
        self.kinship
            .range((a, HumanID::default())..)
            .take_while(move |(x, _)| x.0 == a)
            .map(|(x, y)| (x.1, *y))
    }
}

pub fn generate_social_events(world: &mut World, res: &mut Resources) {
    let mut social = res.write::<SocialNetworkRes>();

    fn next_pivot(kinship: &BTreeMap<(HumanID, HumanID), f32>, pivot: &mut Option<HumanID>) {
        let left_b = pivot
            .map(|v| Bound::Excluded((v, HumanID::default())))
            .unwrap_or(Bound::Unbounded);

        *pivot = kinship
            .range((left_b, Bound::Unbounded))
            .next()
            .map(|(x, _)| x.0);
    }

    let SocialNetworkRes {
        ref kinship,
        ref mut social_maker_pivot,
        ..
    } = *social;

    for _ in 0..10 {
        next_pivot(kinship, social_maker_pivot);
        let Some(v) = *social_maker_pivot else {
            break;
        };
    }
}

pub fn clean_kinships_system(world: &mut World, res: &mut Resources) {
    let mut social = res.write::<SocialNetworkRes>();

    let mut to_remove = Vec::new();

    let SocialNetworkRes {
        ref kinship,
        ref mut cleaner,
        ..
    } = *social;

    for (pair, _) in cleaner.iter(&kinship).take(10) {
        if !world.humans.contains_key(pair.0) || !world.humans.contains_key(pair.1) {
            to_remove.push(pair.clone());
        }
    }

    for pair in to_remove {
        social.kinship.remove(&pair);
        social.kinship.remove(&(pair.1, pair.0));
    }
}
