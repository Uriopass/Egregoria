use crate::economy::Market;
use crate::map_dynamic::{DispatchKind, Router};
use crate::physics::Collider;
use crate::transportation::Vehicle;
use crate::utils::resources::Resources;
use crate::{Egregoria, SoulID};
use hecs::{Component, Entity};
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub trait ComponentDrop {
    fn drop(&mut self, goria: &mut Resources, ent: Entity);
}

type ExecType = Box<dyn for<'a> FnOnce(&'a mut Egregoria) + Send>;

#[derive(Default)]
pub struct ParCommandBuffer {
    to_kill: Mutex<Vec<Entity>>,
    add_comp: Mutex<BTreeMap<(Entity, TypeId), ExecType>>,
    remove_comp: Mutex<BTreeMap<(Entity, TypeId), ExecType>>,
    exec_ent: Mutex<BTreeMap<Entity, Vec<ExecType>>>,
    exec_on: Mutex<BTreeMap<(Entity, TypeId), ExecType>>,
}

#[allow(clippy::unwrap_used)]
impl ParCommandBuffer {
    pub fn kill(&self, e: Entity) {
        self.to_kill.lock().unwrap().push(e);
    }

    pub fn kill_all(&self, e: &[Entity]) {
        self.to_kill.lock().unwrap().extend_from_slice(e);
    }

    pub fn exec_ent(&self, e: Entity, f: impl for<'a> FnOnce(&'a mut Egregoria) + 'static + Send) {
        self.exec_ent
            .lock()
            .unwrap()
            .entry(e)
            .or_default()
            .push(Box::new(f))
    }

    pub fn exec_on<T: Any + Send + Sync>(
        &self,
        e: Entity,
        f: impl for<'a> FnOnce(&'a mut T) + 'static + Send,
    ) {
        let key = (e, TypeId::of::<T>());
        let v = self
            .exec_on
            .lock()
            .unwrap()
            .insert(key, Box::new(move |goria| f(&mut *goria.write::<T>())));
        if v.is_some() {
            log::error!(
                "executing two exec_on closures relating to an entity. Might cause desyncs"
            );
        }
    }

    pub fn add_component<T: Component>(&self, e: Entity, c: T) {
        let key = (e, TypeId::of::<T>());
        let v = self.add_comp.lock().unwrap().insert(
            key,
            Box::new(move |w| {
                let _ = w.world.insert_one(e, c);
            }),
        );
        if v.is_some() {
            log::error!("adding two times the same component to a struct. Might cause desyncs");
        }
    }

    pub fn remove_component<T: Component>(&self, e: Entity) {
        let key = (e, TypeId::of::<T>());
        self.remove_comp.lock().unwrap().insert(
            key,
            Box::new(move |w| {
                let _ = w.world.remove_one::<T>(e);
            }),
        );
    }

    pub fn remove_component_drop<T: Component + ComponentDrop>(&self, e: Entity) {
        let key = (e, TypeId::of::<T>());
        self.remove_comp.lock().unwrap().insert(
            key,
            Box::new(move |w| {
                if let Ok(mut c) = w.world.remove_one::<T>(e) {
                    ComponentDrop::drop(&mut c, &mut w.resources, e);
                }
            }),
        );
    }

    #[profiling::function]
    pub fn apply(goria: &mut Egregoria) {
        let mut deleted: Vec<Entity> =
            std::mem::take(&mut *goria.write::<ParCommandBuffer>().to_kill.get_mut().unwrap());

        deleted.sort_unstable();

        for entity in deleted {
            if goria.world.despawn(entity).is_err() {
                continue;
            }
            goria.write::<Market>().remove(SoulID(entity));
            if let Ok(mut v) = goria.world.get::<&mut Collider>(entity) {
                ComponentDrop::drop(&mut *v, &mut goria.resources, entity);
            }
            if let Ok(mut v) = goria.world.get::<&mut Vehicle>(entity) {
                ComponentDrop::drop(&mut *v, &mut goria.resources, entity);
            }
            if let Ok(mut v) = goria.world.get::<&mut Router>(entity) {
                ComponentDrop::drop(&mut *v, &mut goria.resources, entity);
            }
            if let Ok(mut v) = goria.world.get::<&mut DispatchKind>(entity) {
                ComponentDrop::drop(&mut *v, &mut goria.resources, entity);
            }
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

        for (_, execs) in exec_ent {
            for exec in execs {
                exec(goria);
            }
        }

        let exec_ent =
            std::mem::take(&mut *goria.write::<ParCommandBuffer>().exec_on.get_mut().unwrap());

        for (_, exec) in exec_ent {
            exec(goria);
        }
    }
}
