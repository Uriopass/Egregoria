use crate::map_dynamic::Router;
use crate::physics::Collider;
use crate::vehicles::Vehicle;
use crate::{ent_id, Egregoria};
use legion::storage::Component;
use legion::systems::Resource;
use legion::{Entity, EntityStore, Resources};
use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::Mutex;

pub trait ComponentDrop {
    fn drop(&mut self, goria: &mut Resources, ent: Entity);
}

type ExecType = Box<dyn for<'a> FnOnce(&'a mut Egregoria) + Send>;

register_resource_noserialize!(ParCommandBuffer);
#[derive(Default)]
pub struct ParCommandBuffer {
    to_kill: Mutex<Vec<Entity>>,
    add_comp: Mutex<BTreeMap<(u64, TypeId), ExecType>>,
    remove_comp: Mutex<BTreeMap<(u64, TypeId), ExecType>>,
    exec_ent: Mutex<BTreeMap<u64, ExecType>>,
    exec_on: Mutex<BTreeMap<(u64, TypeId), ExecType>>,
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
        let key = ent_id(e);
        let v = self.exec_ent.lock().unwrap().insert(key, Box::new(f));
        if v.is_some() {
            log::error!("executing two closures relating to an entity. Might cause desyncs");
        }
    }

    pub fn exec_on<T: Resource>(
        &self,
        e: Entity,
        f: impl for<'a> FnOnce(&'a mut T) + 'static + Send,
    ) {
        let key = (ent_id(e), TypeId::of::<T>());
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

    pub fn remove_component<T: Component>(&self, e: Entity) {
        let key = (ent_id(e), TypeId::of::<T>());
        self.remove_comp.lock().unwrap().insert(
            key,
            Box::new(move |w| {
                if let Some(mut x) = w.world.entry(e) {
                    x.remove_component::<T>();
                }
            }),
        );
    }

    pub fn remove_component_drop<T: Component + ComponentDrop>(&self, e: Entity) {
        let key = (ent_id(e), TypeId::of::<T>());
        self.remove_comp.lock().unwrap().insert(
            key,
            Box::new(move |w| {
                Self::parse_del::<T>(w, e);
                if let Some(mut x) = w.world.entry(e) {
                    x.remove_component::<T>();
                }
            }),
        );
    }

    fn parse_del<T: Component + ComponentDrop>(goria: &mut Egregoria, entity: Entity) {
        if let Ok(mut v) = goria.world.entry_mut(entity) {
            if let Ok(v) = v.get_component_mut::<T>() {
                ComponentDrop::drop(v, &mut goria.resources, entity);
            }
        }
    }

    pub fn apply(goria: &mut Egregoria) {
        let mut deleted: Vec<Entity> =
            std::mem::take(&mut *goria.write::<ParCommandBuffer>().to_kill.get_mut().unwrap());

        deleted.sort_unstable_by_key(|&x| ent_id(x));

        for entity in deleted {
            if goria.world.remove(entity) {
                Self::parse_del::<Collider>(goria, entity);
                Self::parse_del::<Vehicle>(goria, entity);
                Self::parse_del::<Router>(goria, entity);
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

        for (_, exec) in exec_ent {
            exec(goria);
        }

        let exec_ent =
            std::mem::take(&mut *goria.write::<ParCommandBuffer>().exec_on.get_mut().unwrap());

        for (_, exec) in exec_ent {
            exec(goria);
        }
    }
}
