use anyhow::Result;
use rustpython_ast::{Mod, ModModule, StmtImport, StmtImportFrom, Visitor};
use rustpython_parser::{Mode, parse};
use std::{
    collections::HashSet,
    path::{MAIN_SEPARATOR_STR, Path, PathBuf},
};

#[derive(Debug)]
pub struct FileImports {
    // relative to current directory
    pub file: PathBuf,
    pub imports: Vec<Import>,
}

#[derive(Debug)]
pub struct ResolvedFileImports {
    pub file: PathBuf,
    pub imports: Vec<String>,
}

impl ResolvedFileImports {
    pub fn new(file: PathBuf, imports: Vec<String>) -> Self {
        Self { file, imports }
    }
}

const INIT_FILE: &str = "__init__.py";

impl FileImports {
    pub fn resolve_imports(
        &self,
        project_files: &HashSet<String>,
        first_level_dirs: &HashSet<PathBuf>,
    ) -> ResolvedFileImports {
        let imports = self.imports.iter().map(|import| {
            if import.is_relative() {
                // resolve relative imports
                let p = self
                    .file
                    .ancestors()
                    .nth(import.level)
                    .expect("Relative import level too high");
                // p = "python_code/"
                p.join(import.to_file_path())
            } else {
                // resolve absolute imports
                import.to_file_path()
            }
        });

        let local_imports = imports.filter(|import| {
            first_level_dirs.contains(&PathBuf::from(&import.components().next().unwrap()))
        });

        let resolved_imports = local_imports
            .map(
                |import| match determine_import_type(&import, project_files) {
                    ImportType::Package(p) => p,
                    ImportType::Module(f) => f,
                    ImportType::Object => {
                        match determine_import_type(
                            import.parent().expect("Import path has no parent"),
                            project_files,
                        ) {
                            ImportType::Package(p) => p,
                            ImportType::Module(f) => f,
                            ImportType::Object => unreachable!(),
                        }
                    }
                },
            )
            .collect();

        ResolvedFileImports::new(self.file.clone(), resolved_imports)
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
    if project_files.contains(&init_file) {
        ImportType::Package(init_file)
    } else {
        let module_name = import
            .with_extension(PY_EXTENSION)
            .to_string_lossy()
            .to_string();
        if project_files.contains(&module_name) {
            return ImportType::Module(module_name);
        }
        ImportType::Object
    }
}

#[derive(Debug, Clone)]
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

pub fn extract_imports(file: &PathBuf) -> Result<FileImports> {
    let file_contents = std::fs::read_to_string(file)?;

    match parse(&file_contents, Mode::Module, "<embedded>") {
        Ok(Mod::Module(ModModule {
            range: _,
            body,
            type_ignores: _t,
        })) => {
            let mut visitor = ImportVisitor { imports: vec![] };
            // it seems rustpython's asts don't implement accept
            body.iter()
                .for_each(|stmt| visitor.visit_stmt(stmt.clone()));
            Ok(FileImports {
                file: file.clone(),
                imports: visitor.imports,
            })
        }
        Err(e) => anyhow::bail!("Error parsing file: {:?}", e),
        _ => anyhow::bail!("Unexpected module type"),
    }
}

#[derive(Debug, Clone)]
struct ImportVisitor {
    pub imports: Vec<Import>,
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
            self.imports.push(import);
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
            self.imports.push(import);
        }
    }
}
