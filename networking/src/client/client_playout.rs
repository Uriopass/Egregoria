use crate::ring::Ring;
use crate::{Frame, MergedInputs, PlayerInput};

/// Playback buffer
///  --------------------------------------
/// |  X  ;  X  ;  .  ;  .  ;  X  ;  X    |
/// |        ^     ^           ^          |
/// | consumed    missing    advance      |
///  -------------------------------------
#[derive(Debug)]
pub(crate) struct ClientPlayoutBuffer {
    future: Ring<Option<MergedInputs>>,
    last_n_inputs: Vec<(Frame, PlayerInput)>,
    consumed_frame: Frame,
    n_input_resend: u32,
}

impl ClientPlayoutBuffer {
    pub fn new(start_frame: Frame, n_input_resend: u32) -> Self {
        Self {
            future: Ring::new(),
            last_n_inputs: vec![],
            consumed_frame: start_frame,
            n_input_resend,
        }
    }

    fn push_client_input(&mut self, frame: Frame, input: PlayerInput) {
        if frame <= self.consumed_frame {
            log::error!("unreachable.. weird");
            return;
        }
        self.last_n_inputs.push((frame, input));
        while self.last_n_inputs.len() > self.n_input_resend as usize {
            self.last_n_inputs.remove(0);
        }
    }

    pub fn insert_serv_input(&mut self, frame: Frame, input: MergedInputs) -> InsertResult {
        if frame <= self.consumed_frame {
            return InsertResult::Ok; // already consumed, safely ignore
        }
        if frame.0 > self.consumed_frame.0 + self.future.len() as u64 {
            return InsertResult::TooFarAhead;
        }
        *self.future.get_mut(frame) = Some(input);
        InsertResult::Ok
    }

    pub fn advance(&self) -> u64 {
        let mut advance = 0;
        while self
            .future
            .get(Frame(self.consumed_frame.0 + 1 + advance))
            .is_some()
            && advance < self.future.len() as u64
        {
            advance += 1;
        }
        advance
    }

    pub fn consumed_frame(&self) -> Frame {
        self.consumed_frame
    }

    /// acknowledge is iterator over last frame acknowledged per user
    ///
    ///   X  X  X N . .  
    ///   ^     ^ ^
    ///  ack cons next
    ///  lag = 3 = next - ack
    ///
    /// `mk_input` is closure which should be called and sent if `try_consume` works out
    pub fn try_consume(
        &mut self,
        mk_input: &mut impl FnMut() -> PlayerInput,
    ) -> Option<(MergedInputs, Vec<(Frame, PlayerInput)>)> {
        let next_frame = self.consumed_frame + Frame(1);

        if let Some(inputs) = self.future.get_mut(next_frame).take() {
            self.push_client_input(next_frame, mk_input());
            self.consumed_frame.0 += 1;
            return Some((inputs, self.last_n_inputs.clone()));
        }
        None
    }
}

pub enum InsertResult {
    TooFarAhead,
    Ok,
}
