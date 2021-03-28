use crate::physics::Collider;
use crate::vehicles::Vehicle;
use crate::Egregoria;
use imgui::__core::any::TypeId;
use legion::storage::Component;
use legion::systems::Resource;
use legion::Entity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct Deleted<T>(Vec<T>);
impl<T> Default for Deleted<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<T> Deleted<T> {
    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.0.drain(..)
    }
}

type ExecType = Box<dyn for<'a> FnOnce(&'a mut Egregoria) + Send>;

register_resource_noserialize!(ParCommandBuffer);
#[derive(Default)]
pub struct ParCommandBuffer {
    to_kill: Mutex<Vec<Entity>>,
    add_comp: Mutex<BTreeMap<(u64, TypeId), ExecType>>,
    remove_comp: Mutex<BTreeMap<(u64, TypeId), ExecType>>,
    exec_ent: Mutex<BTreeMap<u64, ExecType>>,
    execs: Mutex<Vec<ExecType>>,
}

fn ent_id(e: Entity) -> u64 {
    unsafe { std::mem::transmute(e) }
}

impl ParCommandBuffer {
    pub fn kill(&self, e: Entity) {
        self.to_kill.lock().unwrap().push(e);
    }
    pub fn kill_all(&self, e: &[Entity]) {
        self.to_kill.lock().unwrap().extend_from_slice(e);
    }

    pub fn exec_ent(&self, e: Entity, f: impl for<'a> FnOnce(&'a mut Egregoria) + 'static + Send) {
        let key = ent_id(e);
        let v = self.exec_ent.lock().unwrap().insert(key, Box::new(f));
        if v.is_some() {
            log::error!("executing two closures relating to an entity. Might cause desyncs");
        }
    }

    /// Beware of desyncs
    pub fn exec_on<T: Resource>(&self, f: impl for<'a> FnOnce(&'a mut T) + 'static + Send) {
        self.execs
            .lock()
            .unwrap()
            .push(Box::new(move |goria| f(&mut *goria.write::<T>())))
    }

    pub fn add_component<T: Component>(&self, e: Entity, c: T) {
        let key = (ent_id(e), TypeId::of::<T>());
        let v = self.add_comp.lock().unwrap().insert(
            key,
            Box::new(move |w| {
                if let Some(mut x) = w.world.entry(e) {
                    x.add_component(c)
                }
            }),
        );
        if v.is_some() {
            log::error!("adding two times the same component to a struct. Might cause desyncs");
        }
    }

    pub fn remove_component<T: Component + Clone>(&self, e: Entity) {
        let key = (ent_id(e), TypeId::of::<T>());
        let v = self.remove_comp.lock().unwrap().insert(
            key,
            Box::new(move |w| {
                Self::parse_del::<T>(w, e);
                if let Some(mut x) = w.world.entry(e) {
                    x.remove_component::<T>();
                }
            }),
        );
        if v.is_some() {
            log::error!("adding two times the same component to a struct. Might cause desyncs");
        }
    }

    fn parse_del<T: Component + Clone>(goria: &mut Egregoria, entity: Entity) {
        if let Some(v) = goria.comp::<T>(entity).cloned() {
            if let Some(mut x) = goria.try_write::<Deleted<T>>() {
                x.0.push(v)
            }
        }
    }

    pub fn apply(goria: &mut Egregoria) {
        let mut deleted: Vec<Entity> =
            std::mem::take(&mut *goria.write::<ParCommandBuffer>().to_kill.get_mut().unwrap());

        deleted.sort_unstable_by_key(|&x| ent_id(x));

        for entity in deleted {
            Self::parse_del::<Collider>(goria, entity);
            Self::parse_del::<Vehicle>(goria, entity);
            goria.world.remove(entity);
        }

        let added = std::mem::take(
            &mut *goria
                .write::<ParCommandBuffer>()
                .add_comp
                .get_mut()
                .unwrap(),
        );

        for (_, add) in added {
            add(goria);
        }

        let removed = std::mem::take(
            &mut *goria
                .write::<ParCommandBuffer>()
                .remove_comp
                .get_mut()
                .unwrap(),
        );

        for (_, remove) in removed {
            remove(goria);
        }

        let exec_ent = std::mem::take(
            &mut *goria
                .write::<ParCommandBuffer>()
                .exec_ent
                .get_mut()
                .unwrap(),
        );

        for (_, exec) in exec_ent {
            exec(goria);
        }

        let funs: Vec<ExecType> =
            std::mem::take(&mut *goria.write::<ParCommandBuffer>().execs.get_mut().unwrap());
        for fun in funs {
            fun(goria);
        }
    }
}
