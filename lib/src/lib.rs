pub use rutenspitz_macro::arbitrary_stateful_operations;

lazy_static::lazy_static! {
    pub static ref NON_DEBUG_PANIC_HOOK: () = {
        std::panic::set_hook(Box::new(|panic_info| {
            if panic_info.payload().is::<crate::OutcomePanic>() {
                std::process::abort();
            }
        }))
    };
}

#[macro_export]
macro_rules! panic {
    ($($arg:tt)*) => { std::panic!(rutenspitz::OutcomePanic(format!($($arg)*))) };
}

pub struct OutcomePanic(pub String);

pub use lazy_static;

pub mod derive {
    pub use arbitrary::Arbitrary;
    pub use strum_macros::IntoStaticStr;
}
