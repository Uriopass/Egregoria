pub struct DeferFn<V, T: FnOnce() -> V>(pub Option<T>);

impl<V, T: FnOnce() -> V> Drop for DeferFn<V, T> {
    fn drop(&mut self) {
        (self.0.take().unwrap())();
    }
}

#[macro_export]
macro_rules! defer {
    ($e: expr) => {
        let _guard = $crate::macros::DeferFn(Some(move || $e));
    };
}

#[macro_export]
macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
        assert!(
            $x - $y < $d || $y - $x < $d,
            "assert_delta failed: |{} - {}| < {}",
            $x,
            $y,
            $d
        );
    };
}

#[macro_export]
macro_rules! defer_serialize {
    ($me:ty, $defered:ty) => {
        impl serde::Serialize for $me {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
            where
                S: serde::Serializer,
            {
                let v: $defered = self.into();
                serde::Serialize::serialize(&v, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $me {
            fn deserialize<D>(
                deserializer: D,
            ) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let v: $defered = serde::Deserialize::deserialize(deserializer)?;
                Ok(v.into())
            }
        }
    };
}

#[macro_export]
macro_rules! unwrap_or {
    ($e: expr, $t: expr) => {
        match $e {
            Some(x) => x,
            None => $t,
        }
    };
}

#[macro_export]
macro_rules! assert_ret {
    ($e: expr) => {
        if !$e {
            return false;
        }
    };
}

#[macro_export]
macro_rules! unwrap_ret {
    ($e: expr) => {
        unwrap_ret!($e, ())
    };
    ($e: expr, $ret: expr) => {
        match $e {
            Some(x) => x,
            None => return $ret,
        }
    };
}

#[macro_export]
macro_rules! unwrap_cont {
    ($e: expr) => {
        match $e {
            Some(x) => x,
            None => continue,
        }
    };
}

#[macro_export]
macro_rules! unwrap_orr {
    ($e: expr, $t: expr) => {
        match $e {
            Ok(x) => x,
            Err(_) => $t,
        }
    };
}

#[macro_export]
macro_rules! unwrap_retlog {
    ($e: expr, $($t: expr),+) => {
        match $e {
            Some(x) => x,
            None => {
                log::error!($($t),+);
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! unwrap_contlog {
    ($e: expr, $($t: expr),+) => {
        match $e {
            Some(x) => x,
            None => {
                log::error!($($t),+);
                continue;
            }
        }
    };
}
