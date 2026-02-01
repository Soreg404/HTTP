type ChunkSize = usize;
enum HTTPTransferEncoding {
	Chunked(ChunkSize),
}
