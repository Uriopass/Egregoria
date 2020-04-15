macro_rules! unwrap_ret {
    ($e: expr) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
}

pub trait Choose<'a> {
    type Output;
    fn choose(&'a self) -> Self::Output;
}

impl<'a, T: 'a> Choose<'a> for Vec<T> {
    type Output = Option<&'a T>;

    fn choose(&'a self) -> Self::Output {
        if self.is_empty() {
            None
        } else {
            let l = self.len();
            let ix = (l as f32 * rand::random::<f32>()) as usize;
            Some(&self[ix])
        }
    }
}

pub trait Restrict {
    fn restrict(self, min: Self, max: Self) -> Self;
}

impl<T: PartialOrd> Restrict for T {
    fn restrict(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}
