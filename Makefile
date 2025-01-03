default: run

run:
	@cargo run -- `cat file_paths.txt`
