use crate::utils::resources::Resources;
use crate::world::Entity;
use crate::Egregoria;
use std::sync::Mutex;

pub trait GoriaDrop: Entity {
    fn goria_drop(self, id: Self::ID, res: &mut Resources);
}

type ExecType = Box<dyn for<'a> FnOnce(&'a mut Egregoria) + Send>;

pub struct ParCommandBuffer<E: GoriaDrop> {
    to_kill: Mutex<Vec<E::ID>>,
    exec_ent: Mutex<Vec<(E::ID, ExecType)>>,
}

impl<E: GoriaDrop> Default for ParCommandBuffer<E> {
    fn default() -> Self {
        Self {
            to_kill: Default::default(),
            exec_ent: Default::default(),
        }
    }
}

#[allow(clippy::unwrap_used)]
impl<E: GoriaDrop> ParCommandBuffer<E> {
    pub fn kill(&self, e: E::ID) {
        self.to_kill.lock().unwrap().push(e);
    }

    pub fn kill_all(&self, e: &[E::ID]) {
        self.to_kill.lock().unwrap().extend_from_slice(e);
    }

    pub fn exec_ent(&self, e: E::ID, f: impl for<'a> FnOnce(&'a mut Egregoria) + 'static + Send) {
        self.exec_ent.lock().unwrap().push((e, Box::new(f)));
    }

    pub fn exec_on<T: Send + Sync + 'static>(
        &self,
        e: E::ID,
        f: impl for<'a> FnOnce(&'a mut T) + 'static + Send,
    ) {
        self.exec_ent(e, move |goria| {
            f(&mut *goria.write::<T>());
        })
    }

    pub fn apply(goria: &mut Egregoria) {
        profiling::scope!("par_command_buffer::apply");
        let mut deleted: Vec<E::ID> = std::mem::take(
            &mut *goria
                .write::<ParCommandBuffer<E>>()
                .to_kill
                .get_mut()
                .unwrap(),
        );

        deleted.sort_unstable();

        for entity in deleted {
            let Some(v) = E::storage_mut(&mut goria.world).remove(entity) else { continue };

            E::goria_drop(v, entity, &mut goria.resources);
        }

        let mut exec_ent = std::mem::take(
            &mut *goria
                .write::<ParCommandBuffer<E>>()
                .exec_ent
                .get_mut()
                .unwrap(),
        );

        exec_ent.sort_unstable_by_key(|(id, _)| *id);

        for (_, exec) in exec_ent {
            exec(goria);
        }
    }
}
