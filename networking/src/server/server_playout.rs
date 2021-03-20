use crate::ring::Ring;
use crate::{Frame, MergedInputs, PlayerInput, UserID};
use std::collections::HashMap;

type PartialInputs = HashMap<UserID, PlayerInput>;

///       Playback buffer
///  --------------------------------------
/// |    past   |         future          |
/// |  X  ;  X  |  .  ;  .  ;  X  ;  X    |
/// |        ^     ^                      |
/// | consumed    missing                 |
///  -------------------------------------
pub(crate) struct ServerPlayoutBuffer {
    future: Ring<PartialInputs>,
    past: Ring<MergedInputs>,
    pub consumed_frame: Frame,
}

impl ServerPlayoutBuffer {
    pub fn new(start_frame: Frame) -> Self {
        Self {
            future: Ring::new(),
            past: Ring::new(),
            consumed_frame: start_frame,
        }
    }

    pub fn insert_input(&mut self, frame: Frame, user: UserID, input: PlayerInput) -> InsertResult {
        if frame <= self.consumed_frame {
            return InsertResult::Ok; // already consumed, safely ignore
        }
        if frame.0 >= self.consumed_frame.0 + self.future.len() {
            return InsertResult::TooFarAhead;
        }
        self.future.get_mut(frame).insert(user, input);
        InsertResult::Ok
    }

    // call when a user has disconnected
    pub fn disconnected(&mut self, user: UserID) {
        for v in self.future.iter_mut() {
            v.remove(&user);
        }
    }

    /// acknowledge is iterator over last frame acknowledged per user
    ///
    ///   X  X  X N . .  
    ///   ^     ^ ^
    ///  ack cons next
    ///  lag = 2 = cons - ack
    pub fn try_consume(
        &mut self,
        acknowledged: impl Iterator<Item = Frame>,
        force_consume: bool,
        n_users: usize,
    ) -> Option<(MergedInputs, Vec<Vec<(Frame, MergedInputs)>>)> {
        let next_frame = self.consumed_frame + Frame(1);

        if self.future.get(next_frame).len() == n_users || force_consume {
            log::info!(
                "{}: len is {}/{} {}",
                self.consumed_frame.0,
                self.future.get(next_frame).len(),
                n_users,
                if force_consume { "force_consume" } else { "" }
            );
            let mut result = vec![];
            let merged = merge_partial_inputs(self.future.get(next_frame));

            for ack_frame in acknowledged {
                debug_assert!(ack_frame <= self.consumed_frame);
                let lag = self.consumed_frame.0 - ack_frame.0;
                debug_assert!(lag < self.past.len());

                let v = (1..=lag)
                    .map(|i| {
                        let frame = ack_frame + Frame(i);
                        (frame, self.past.get(frame).clone())
                    })
                    .chain(std::iter::once((next_frame, merged.clone())))
                    .collect::<Vec<_>>();

                result.push(v);
            }

            // advance
            self.consumed_frame.0 += 1;
            *self.past.get_mut(self.consumed_frame) = merged.clone();
            *self.future.get_mut(self.consumed_frame) = Default::default();

            return Some((merged, result));
        }
        None
    }
}

pub enum InsertResult {
    TooFarAhead,
    Ok,
}

fn merge_partial_inputs(x: &PartialInputs) -> MergedInputs {
    x.values().cloned().collect()
}
