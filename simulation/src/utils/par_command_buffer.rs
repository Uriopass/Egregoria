use crate::utils::resources::Resources;
use crate::world::Entity;
use crate::Simulation;
use std::sync::Mutex;

pub trait SimDrop: Entity {
    fn sim_drop(self, id: Self::ID, res: &mut Resources);
}

type ExecType = Box<dyn for<'a> FnOnce(&'a mut Simulation) + Send>;

pub struct ParCommandBuffer<E: SimDrop> {
    to_kill: Mutex<Vec<E::ID>>,
    exec_ent: Mutex<Vec<(E::ID, ExecType)>>,
}

impl<E: SimDrop> Default for ParCommandBuffer<E> {
    fn default() -> Self {
        Self {
            to_kill: Default::default(),
            exec_ent: Default::default(),
        }
    }
}

#[allow(clippy::unwrap_used)]
impl<E: SimDrop> ParCommandBuffer<E> {
    pub fn kill(&self, e: E::ID) {
        self.to_kill.lock().unwrap().push(e);
    }

    pub fn kill_all(&self, e: &[E::ID]) {
        self.to_kill.lock().unwrap().extend_from_slice(e);
    }

    pub fn exec_ent(&self, e: E::ID, f: impl for<'a> FnOnce(&'a mut Simulation) + 'static + Send) {
        self.exec_ent.lock().unwrap().push((e, Box::new(f)));
    }

    pub fn exec_on<T: Send + Sync + 'static>(
        &self,
        e: E::ID,
        f: impl for<'a> FnOnce(&'a mut T) + 'static + Send,
    ) {
        self.exec_ent(e, move |sim| {
            f(&mut *sim.write::<T>());
        })
    }

    pub fn apply(sim: &mut Simulation) {
        profiling::scope!("par_command_buffer::apply");
        let mut deleted: Vec<E::ID> = std::mem::take(
            &mut *sim
                .write::<ParCommandBuffer<E>>()
                .to_kill
                .get_mut()
                .unwrap(),
        );

        deleted.sort_unstable();

        for entity in deleted {
            let Some(v) = E::storage_mut(&mut sim.world).remove(entity) else {
                continue;
            };

            E::sim_drop(v, entity, &mut sim.resources);
        }

        let mut exec_ent = std::mem::take(
            &mut *sim
                .write::<ParCommandBuffer<E>>()
                .exec_ent
                .get_mut()
                .unwrap(),
        );

        exec_ent.sort_unstable_by_key(|(id, _)| *id);

        for (_, exec) in exec_ent {
            exec(sim);
        }
    }
}
