use crate::import::*;

impl Codegen {
     
    pub fn set_import_context(&mut self, import_decls: &[ImportDecl]) {
        for decl in import_decls {
            match decl {
                ImportDecl::LibraryImport { name } => {
                    self.linked_libraries.push(name.clone());
                }
                ImportDecl::FileImport { name, from } => {
                    self.linked_libraries.push(from.clone());
                }
                ImportDecl::WildcardImport { from } => {
                    self.linked_libraries.push(from.clone());
                }
            }
        }
    }

    pub fn with_import_context(mut self, import_decls: &[ImportDecl]) -> Self {
        self.set_import_context(import_decls);
        self
    }

}