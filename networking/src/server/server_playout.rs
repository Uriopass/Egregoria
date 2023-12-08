use crate::authent::AuthentID;
use crate::ring::Ring;
use crate::{Frame, MergedInputs, PlayerInput};
use common::FastMap;

type PartialInputs = FastMap<AuthentID, Vec<PlayerInput>>;

///  Playback buffer
///  --------------------------------------
/// |    past   |         future          |
/// |  X  ;  X  |  .  ;  .  ;  X  ;  X    |
/// |        ^     ^                      |
/// | consumed    missing                 |
///  -------------------------------------
pub(crate) struct ServerPlayoutBuffer {
    next: PartialInputs,
    dedup: FastMap<AuthentID, Ring<bool>>,
    past: Ring<MergedInputs>,
    pub consumed_frame: Frame,
}

type PastInputs = Vec<(Frame, MergedInputs)>;

impl ServerPlayoutBuffer {
    pub fn new(start_frame: Frame) -> Self {
        Self {
            next: PartialInputs::default(),
            dedup: Default::default(),
            past: Ring::new(),
            consumed_frame: start_frame,
        }
    }

    pub fn insert_input(&mut self, auth: AuthentID, frame: Frame, input: PlayerInput) {
        if frame.0 + self.past.len() as u64 <= self.consumed_frame.0 {
            log::info!("input was far too late");
            return;
        }
        let seen = self
            .dedup
            .entry(auth)
            .or_insert_with(Ring::new)
            .get_mut(frame);

        if !*seen {
            self.next.entry(auth).or_default().push(input);
            *seen = true;
        }
    }

    pub fn lag(&self, f: Frame) -> Option<u64> {
        let lag = self.consumed_frame.0 - f.0;
        if lag < self.past.len() as u64 - 1 {
            Some(lag)
        } else {
            None
        }
    }

    // call when a user has disconnected
    pub fn disconnected(&mut self, user: AuthentID) {
        self.dedup.remove(&user);
    }

    /// acknowledge is iterator over last frame acknowledged per user
    ///
    ///   X  X  X N . .  
    ///   ^     ^ ^
    ///  ack cons next
    ///  lag = 2 = cons - ack
    pub fn consume(
        &mut self,
        acknowledged: impl Iterator<Item = Frame>,
    ) -> (MergedInputs, Vec<PastInputs>) {
        let next_frame = self.consumed_frame.incred();

        for v in self.dedup.values_mut() {
            *v.get_mut(next_frame) = false;
        }

        let mut result = vec![];
        let merged = merge_partial_inputs(&mut self.next);

        for ack_frame in acknowledged {
            let lag = self.lag(ack_frame).expect("lag is too big");
            debug_assert!(ack_frame <= self.consumed_frame);

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
        self.consumed_frame = next_frame;
        *self.past.get_mut(self.consumed_frame) = merged.clone();

        (merged, result)
    }
}

fn merge_partial_inputs(x: &mut PartialInputs) -> MergedInputs {
    x.iter_mut()
        .flat_map(|(&id, v)| v.drain(..).map(move |v| (id, v)))
        .collect()
}
