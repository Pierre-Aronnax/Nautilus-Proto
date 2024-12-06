// macros.rs

#[macro_export]
macro_rules! init_logging {
    () => {
        if let Err(e) = $crate::setup_logger(None) {
            eprintln!("Failed to initialize logger: {}", e);
        }
    };
    ($config_path:expr) => {
        if let Err(e) = $crate::setup_logger(Some($config_path)) {
            eprintln!("Failed to initialize logger: {}", e);
        }
    };
}

// #[macro_export]
// macro_rules! log_info {
//     ($($arg:tt)*) => {
//         tracing::info!($($arg)*);
//     };
// }

// #[macro_export]
// macro_rules! log_debug {
//     ($($arg:tt)*) => {
//         tracing::debug!($($arg)*);
//     };
// }

// #[macro_export]
// macro_rules! log_error {
//     ($($arg:tt)*) => {
//         tracing::error!($($arg)*);
//     };
// }

// #[macro_export]
// macro_rules! log_trace {
//     ($($arg:tt)*) => {
//         tracing::trace!($($arg)*);
//     };
// }

// // Remove this macro as it is not effective
// // macro_rules! define_events { ... }
// macro_rules! define_events {
//     ($($event_name:ident => $level:ident),* $(,)?) => {
//         $(
//             #[macro_export]
//             macro_rules! $event_name {
//                 ($msg:expr) => {
//                     tracing::event!(tracing::Level::$level, $msg);
//                 };
//                 ($msg:expr, $($key:tt = $value:expr),* $(,)?) => {
//                     tracing::event!(tracing::Level::$level, $msg, $($key = $value),*);
//                 };
//             }
//         )*
//     };
// }


#[macro_export]
macro_rules! log_event {
    ($level:ident, $event_name:expr, $($key:tt = $value:expr),* $(,)?) => {
        $crate::tracing::event!(
            $crate::tracing::Level::$level,
            event_name = $event_name,
            $($key = $value),*
        );
    };
}