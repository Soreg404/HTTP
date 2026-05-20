pub struct CollectorConfig {
	max_request_url_length: usize,
	max_response_status_desc_length: usize,
	max_headers_count: usize,
	max_single_header_size: usize,
	max_headers_combined_size: usize,
	max_message_size: usize,
}
