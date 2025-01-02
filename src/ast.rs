use anyhow::Result;
use rustpython_ast::{Mod, ModModule, StmtImport, StmtImportFrom, Visitor};
use rustpython_parser::{Mode, parse};
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileImports {
    pub file: PathBuf,
    pub imports: Vec<Import>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub segments: Vec<String>,
    pub level: usize,
}

const IMPORT_SEPARATOR: &str = ".";

impl Import {
    fn to_file_path(&self) -> String {
        self.segments.join(std::path::MAIN_SEPARATOR_STR)
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
