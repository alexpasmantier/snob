use anyhow::Result;
use log::{debug, warn};
use rustpython_ast::{Mod, ModModule, StmtImport, StmtImportFrom, Visitor};
use rustpython_parser::{parse, Mode};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf, MAIN_SEPARATOR_STR},
};

#[derive(Debug)]
pub struct FileImports {
    pub file: PathBuf,
    pub imports: Vec<Import>,
}

pub const INIT_FILE: &str = "__init__.py";

impl FileImports {
    pub fn resolve_imports(
        &self,
        project_files: &HashSet<String>,
        first_level_components: &[Vec<PathBuf>],
    ) -> HashSet<String> {
        let imports = self.imports.iter().filter_map(|import| {
            if import.is_relative() {
                // resolve relative imports
                let p = self
                    .file
                    .ancestors()
                    .nth(import.level)
                    .expect("Relative import level too high");
                Some(p.join(import.to_file_path()))
            } else {
                // resolve absolute (python) imports
                let p = import.to_file_path();
                // check first_level_components
                first_level_components
                    .iter()
                    .flatten()
                    .find(|c| c.file_name().unwrap() == p.components().next().unwrap().as_os_str())
                    .map(|component| component.parent().unwrap().join(p))
            }
        });

        let resolved_imports = imports
            .filter_map(
                |import| match determine_import_type(&import, project_files) {
                    ImportType::Package(p) => Some(p),
                    ImportType::Module(f) => Some(f),
                    ImportType::Object => {
                        debug!("Resolving object import {:?}", import);
                        match determine_import_type(
                            import.parent().expect("Import path has no parent"),
                            project_files,
                        ) {
                            ImportType::Package(p) => Some(p),
                            ImportType::Module(f) => Some(f),
                            ImportType::Object => {
                                warn!(
                                    "Failed to resolve import {:?} in file {:?}",
                                    import.file_name().unwrap(),
                                    self.file
                                );
                                None
                            }
                        }
                    }
                },
            )
            .collect();

        resolved_imports
    }
}

enum ImportType {
    Package(String),
    Module(String),
    Object,
}

const PY_EXTENSION: &str = "py";

fn determine_import_type(import: &Path, project_files: &HashSet<String>) -> ImportType {
    let init_file = import.join(INIT_FILE).to_string_lossy().to_string();
    debug!("Checking if {:?} is a package", init_file);
    if project_files.contains(&init_file) {
        ImportType::Package(init_file)
    } else {
        debug!("\tnot a package");
        let module_name = import
            .with_extension(PY_EXTENSION)
            .to_string_lossy()
            .to_string();
        debug!("Checking if {:?} is a module", module_name);
        if project_files.contains(&module_name) {
            return ImportType::Module(module_name);
        }
        debug!("\tnot a module");
        ImportType::Object
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Import {
    pub segments: Vec<String>,
    pub level: usize,
}

const IMPORT_SEPARATOR: &str = ".";

impl Import {
    fn to_file_path(&self) -> PathBuf {
        PathBuf::from(self.segments.join(MAIN_SEPARATOR_STR))
    }

    fn is_relative(&self) -> bool {
        self.level > 0
    }
}

pub fn extract_file_dependencies(
    file: &PathBuf,
    project_files: &HashSet<String>,
    first_level_components: &[Vec<PathBuf>],
) -> Result<HashMap<String, Vec<String>>> {
    let file_contents = std::fs::read_to_string(file)?;

    let mut graph = HashMap::new();

    match parse(&file_contents, Mode::Module, "<embedded>") {
        Ok(Mod::Module(ModModule {
            range: _,
            body,
            type_ignores: _t,
        })) => {
            let mut visitor = ImportVisitor {
                imports: HashSet::new(),
            };
            body.iter()
                .for_each(|stmt| visitor.visit_stmt(stmt.clone()));

            let file_imports = FileImports {
                file: file.clone(),
                imports: visitor.imports.into_iter().collect(),
            };

            let resolved_imports =
                file_imports.resolve_imports(project_files, first_level_components);

            for import in resolved_imports {
                graph
                    .entry(import)
                    .or_insert_with(Vec::new)
                    .push(file.to_string_lossy().to_string());
            }

            Ok(graph)
        }
        Err(e) => anyhow::bail!("Error parsing file {:?}: {:?}", file, e),
        _ => anyhow::bail!("Unexpected module type in file {:?}", file),
    }
}

#[derive(Debug, Clone)]
struct ImportVisitor {
    pub imports: HashSet<Import>,
}

impl Visitor for ImportVisitor {
    fn visit_stmt_import(&mut self, stmt: StmtImport) {
        // import a.b.c as c, d.e.f as f
        for alias in stmt.names {
            let import = Import {
                segments: alias
                    .name
                    .split(IMPORT_SEPARATOR)
                    .map(std::string::ToString::to_string)
                    .collect(),
                level: 0,
            };
            self.imports.insert(import);
        }
    }

    fn visit_stmt_import_from(&mut self, stmt: StmtImportFrom) {
        // from ..a.b import c, d
        let level: usize = stmt.level.map_or(0, |l| l.to_usize());

        for alias in stmt.names {
            let mut segments = Vec::new();
            if let Some(module) = &stmt.module {
                segments.extend(
                    module
                        .split(IMPORT_SEPARATOR)
                        .map(std::string::ToString::to_string),
                );
            }
            segments.extend(
                alias
                    .name
                    .split(IMPORT_SEPARATOR)
                    .map(std::string::ToString::to_string),
            );
            let import = Import { segments, level };
            self.imports.insert(import);
        }
    }
}
