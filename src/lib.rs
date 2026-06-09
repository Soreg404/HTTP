
macro_rules! dtrace {
    ($tag:expr, $msg:expr) => {
        let file_info = if std::env::var("RUST_BACKTRACE").is_ok() {
            format!("\x1b[90m[{}] {}:\x1b[0m", file!(), line!())
        } else { String::new() };
        println!(
            "{}\
            [{}] {}",
            file_info,
            $tag,
            $msg
        );
    }
}

mod proto;

pub use proto::collector::RequestCollector;
pub use proto::collector::collect_error::CollectError;
