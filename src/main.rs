
#[path = "../samples.rs"]
mod samples;

use http::request_collector::RequestCollector;

fn main() {
	let mut rc = RequestCollector::new();

	rc.push_bytes(samples::MULTIPART1);

	match rc.is_finished() {
		None => {
			println!("rc not finished");
		}
		Some(Err(e)) => {
			println!("rc finished err: {e:?}");
		}
		Some(Ok(())) => {
			println!("rc finished ok");
		}
	};

	// let req = rc.
}
