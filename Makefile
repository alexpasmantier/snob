default: run

run:
	@cargo run -- --target-directory python_code `cat file_paths.txt`
