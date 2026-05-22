#[path = "../samples.rs"]
mod samples;

fn main() {


	let mut rc = http::RequestCollector::new();

	rc.push_bytes(samples::MULTIPART1);

	match rc.is_finished() {
		None => {
			println!("rc not finished");
			return;
		}
		Some(Err(e)) => {
			println!("rc finished err: {e:?}");
			return;
		}
		Some(Ok(())) => {
			println!("rc finished ok");
		}
	};

	// let req = rc.
}
