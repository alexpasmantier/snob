use rustc_hash::{FxHashMap, FxHashSet};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use anyhow::Result;

use clap::Parser;

mod ast;
mod cli;
mod config;
mod fs;
mod graph;
mod logging;
mod results;
mod stdin;
mod utils;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    logging::init_logging(&logging::LoggingConfiguration::new(
        cli.verbosity_level,
        cli.quiet,
    ));

    let current_dir = std::env::current_dir()?;
    let git_root = utils::get_repo_root(&current_dir)?;
    snob_debug!("Git root: {:?}", git_root);

    let config = config::Config::new(&git_root);
    snob_debug!("Config: {:?}", config);

    // files that were modified by the range of commits
    let input_files = if stdin::is_readable_stdin() {
        stdin::read_from_stdin()
    } else {
        cli.updated_files
    };
    let updated_files = fs::make_files_relative_to(&input_files, &current_dir)
        .iter()
        .cloned()
        .collect::<FxHashSet<String>>();
    snob_debug!("Updated files: {:?}", updated_files);

    fs::check_files_exist(&updated_files)?;

    let run_all_tests_on_change = fs::build_glob_set(&config.files.run_all_tests_on_change)?;
    if utils::should_run_all_tests(&updated_files, &run_all_tests_on_change, &git_root) {
        // exit early and run all tests
        snob_info!("Running all tests");
        println!(".");
        return Ok(());
    }

    std::env::set_current_dir(&cli.target_directory)?;
    snob_debug!("Current directory: {:?}", current_dir);
    let lookup_paths = utils::get_python_local_lookup_paths(&current_dir, &git_root);
    snob_debug!("Python lookup paths: {:?}", lookup_paths);

    // FIXME: we might want to measure time differently here
    let instant = std::time::Instant::now();

    // crawl the target directory
    let workspace_files = fs::crawl_workspace(&current_dir);

    // these need to retain some sort of order information
    let first_level_components: Vec<PathBuf> = fs::get_first_level_components(&lookup_paths);

    snob_debug!("First level components: {:?}", first_level_components);

    snob_debug!(
        "Crawled {:?} files and {:?} directories",
        workspace_files.len(),
        first_level_components.len()
    );

    // keep a copy of the tree (contains all workspace files)
    let project_files = workspace_files
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect::<FxHashSet<String>>();

    // build dependency graph (remove ignored files)
    let file_ignores = fs::build_glob_set(&config.files.ignores)?;
    let mut all_file_imports: Vec<FxHashMap<String, Vec<String>>> = graph::build_dependency_graph(
        &workspace_files,
        &project_files,
        &file_ignores,
        &first_level_components,
        &git_root,
    );

    // not deduplicated
    let dependency_graph =
        utils::deduplicate_dependencies(utils::merge_hashmaps(&mut all_file_imports));

    snob_debug!("Dependency graph:");
    for (k, v) in &dependency_graph {
        snob_debug!("\t{k} is used by:");
        v.iter().for_each(|v| snob_debug!("\t\t{v}"));
    }

    let impacted_nodes: FxHashSet<String> = if let Some(dot_graph) = &cli.dot_graph {
        graph::discover_impacted_nodes_with_graphviz(&dependency_graph, &updated_files, dot_graph)
    } else {
        graph::discover_impacted_nodes(&dependency_graph, &updated_files)
    };

    // filter impacted nodes to get the tests
    // just like `pytest` we consider files that looks like either test_*.py or *_test.py
    // see https://docs.pytest.org/en/stable/explanation/goodpractices.html#conventions-for-python-test-discovery
    let ignored_tests = fs::build_glob_set(&config.tests.ignores)?;
    let tests_to_always_run = fs::build_glob_set(&config.tests.always_run)?;

    let snob_results = results::SnobResult::new(
        impacted_nodes,
        project_files.clone(),
        &ignored_tests,
        &tests_to_always_run,
        &git_root,
    );
    snob_debug!(" impacted tests: {:?}", snob_results.impacted);
    snob_debug!(" ignored tests: {:?}", snob_results.ignored);
    snob_debug!(" always run tests: {:?}", snob_results.always_run);

    snob_info!(
        "Analyzed {:?} files in {:?}",
        workspace_files.len(),
        instant.elapsed()
    );
    snob_info!(
        "Found {}/{} impacted tests",
        snob_results.impacted.len(),
        workspace_files
            .iter()
            .filter(|f| utils::is_test_file(f))
            .collect::<Vec<_>>()
            .len()
    );

    // output resulting test files
    let stdout = std::io::stdout().lock();
    let mut writer = BufWriter::new(stdout);

    for test in snob_results.impacted {
        writeln!(writer, "{test}")?;
    }

    writer.flush()?;

    Ok(())
}
