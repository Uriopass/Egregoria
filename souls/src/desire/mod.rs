use egregoria::api::Action;
use egregoria::Egregoria;
use ordered_float::OrderedFloat;

mod buyfood;
mod home;
mod work;

pub use buyfood::*;
pub use home::*;
pub use work::*;

pub trait Desire<T>: Send + Sync {
    fn name(&self) -> &'static str;
    fn score(&self, goria: &Egregoria, soul: &T) -> f32;
    fn apply(&mut self, goria: &Egregoria, soul: &mut T) -> Action;
}

pub trait Desires<T> {
    fn decision(&mut self, soul: &mut T, goria: &Egregoria) -> Action;
    fn scores_names<'a>(
        &'a self,
        goria: &'a Egregoria,
        soul: &'a T,
    ) -> Box<dyn Iterator<Item = (f32, &'static str)> + 'a>;
}

impl<T> Desires<T> for Vec<Box<dyn Desire<T>>> {
    fn decision(&mut self, soul: &mut T, goria: &Egregoria) -> Action {
        self.iter_mut()
            .max_by_key(|d| OrderedFloat(d.score(goria, soul)))
            .map(move |d| d.apply(goria, soul))
            .unwrap_or_default()
    }

    fn scores_names<'a>(
        &'a self,
        goria: &'a Egregoria,
        soul: &'a T,
    ) -> Box<dyn Iterator<Item = (f32, &'static str)> + '_> {
        Box::new(self.iter().map(move |x| (x.score(goria, soul), x.name())))
    }
}

impl<T> Desires<T> for () {
    fn decision(&mut self, _: &mut T, _: &Egregoria) -> Action {
        Action::DoNothing
    }

    fn scores_names<'a>(
        &'a self,
        _: &'a Egregoria,
        _: &'a T,
    ) -> Box<dyn Iterator<Item = (f32, &'static str)>> {
        Box::new(std::iter::empty())
    }
}

impl<T, A: Desire<T>> Desires<T> for (A,) {
    fn decision(&mut self, soul: &mut T, goria: &Egregoria) -> Action {
        self.0.apply(goria, soul)
    }

    fn scores_names<'a>(
        &'a self,
        goria: &'a Egregoria,
        soul: &'a T,
    ) -> Box<dyn Iterator<Item = (f32, &'static str)>> {
        Box::new(std::iter::once((self.0.score(goria, soul), self.0.name())))
    }
}

macro_rules! impl_desires_tuple {
        ( $($name:ident;$idx: literal)+) => (
            impl<T, $($name: Desire<T>),*> Desires<T> for ($($name,)*) {
                #[inline]
                #[allow(non_snake_case)]
                #[allow(unused_assignments)]
                fn decision(&mut self, soul: &mut T, goria: &Egregoria) -> Action {
                    let ($(ref mut $name,)*) = *self;
                    let mut max_score = f32::NEG_INFINITY;
                    let mut max_idx = 0;
                    $(
                      let score = $name.score(goria, soul);
                      if score > max_score {
                        max_score = score;
                        max_idx = $idx;
                      }
                    )*

                    match max_idx {
                    $(
                        $idx => $name.apply(goria, soul),
                    )*
                    _ => unsafe { std::hint::unreachable_unchecked() }
                    }
                }

                #[allow(non_snake_case)]
                fn scores_names<'a>(&'a self, goria: &'a Egregoria, soul: &'a T) -> Box<dyn Iterator<Item = (f32, &'static str)>> {
                    let ($(ref $name,)*) = *self;
                    let iter = std::iter::empty()
                    $(
                        .chain(std::iter::once(($name.score(goria, soul), $name.name())))

                    )*;
                    Box::new(iter)
                }
            }
        );
}

impl_desires_tuple!(A;0 B;1);
impl_desires_tuple!(A;0 B;1 C;2);
impl_desires_tuple!(A;0 B;1 C;2 D;3);
impl_desires_tuple!(A;0 B;1 C;2 D;3 E;4);
impl_desires_tuple!(A;0 B;1 C;2 D;3 E;4 F;5);
impl_desires_tuple!(A;0 B;1 C;2 D;3 E;4 F;5 G;6);
impl_desires_tuple!(A;0 B;1 C;2 D;3 E;4 F;5 G;6 H;7);
