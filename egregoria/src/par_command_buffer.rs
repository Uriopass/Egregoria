use crate::physics::Collider;
use crate::vehicles::VehicleComponent;
use crate::{Deleted, Egregoria};
use legion::storage::Component;
use legion::Entity;
use std::sync::Mutex;

#[derive(Default)]
pub struct ParCommandBuffer {
    to_kill: Mutex<Vec<Entity>>,
    execs: Mutex<Vec<Box<dyn for<'a> FnOnce(&'a mut Egregoria) -> () + Send>>>,
}

impl ParCommandBuffer {
    pub fn kill(&self, e: Entity) {
        self.to_kill.lock().unwrap().push(e);
    }
    pub fn kill_all(&self, e: &[Entity]) {
        self.to_kill.lock().unwrap().extend_from_slice(e);
    }

    pub fn exec(&self, f: impl for<'a> FnOnce(&'a mut Egregoria) -> () + 'static + Send) {
        self.execs.lock().unwrap().push(Box::new(f));
    }

    pub fn add_component<T: Component>(&self, e: Entity, c: T) {
        self.exec(move |w| {
            if let Some(mut x) = w.world.entry(e) {
                x.add_component(c)
            }
        })
    }

    pub fn remove_component<T: Component + Clone>(&self, e: Entity) {
        self.exec(move |w| {
            Self::parse_del::<T>(w, e);
            if let Some(mut x) = w.world.entry(e) {
                x.remove_component::<T>()
            }
        })
    }

    fn parse_del<T: Component + Clone>(goria: &mut Egregoria, entity: Entity) {
        if let Some(v) = goria.comp::<T>(entity).cloned() {
            goria
                .try_write::<Deleted<T>>()
                .map(move |mut x| x.0.push(v));
        }
    }

    pub fn apply(goria: &mut Egregoria) {
        let deleted: Vec<Entity> = std::mem::take(
            goria
                .write::<ParCommandBuffer>()
                .to_kill
                .lock()
                .unwrap()
                .as_mut(),
        );
        for entity in deleted {
            Self::parse_del::<Collider>(goria, entity);
            Self::parse_del::<VehicleComponent>(goria, entity);
            goria.world.remove(entity);
        }

        let funs: Vec<Box<dyn for<'a> FnOnce(&'a mut Egregoria) -> () + Send>> = std::mem::take(
            goria
                .write::<ParCommandBuffer>()
                .execs
                .lock()
                .unwrap()
                .as_mut(),
        );
        for fun in funs {
            fun(goria);
        }
    }
}

impl Egregoria {
    pub fn coucou() {}
}
