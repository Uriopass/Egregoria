macro_rules! unwrap_ret {
    ($e: expr) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
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
