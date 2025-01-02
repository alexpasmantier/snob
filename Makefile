default: run

run:
	@cargo run -- --target-directory python `cat file_paths.txt`
