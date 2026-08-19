// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs.contributors>
//
// SPDX-License-Identifier: MIT

//! Utility functions for manipulating JavaScript Abstract Syntax Trees (ASTs).
//!
//! This module provides various tools for working with JavaScript ASTs, including:
//! - Parsing JavaScript code into an AST.
//! - Modifying AST nodes such as `hooks` objects or import declarations.
//! - Performing queries on the AST, such as checking for specific variable declarations.
//!
//! The module leverages a Rust-based parser and integrates seamlessly with Elixir through NIFs.

use crate::parsers::javascript::dialect::Dialect;
use crate::parsers::javascript::helpers::*;
use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};

#[derive(Debug, PartialEq, Eq)]
pub enum Operation {
    Add,
    Edit,
    Delete,
    Read,
    Replace,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FindCondition {
    Found,
    NotFound(String),
    FoundError(String),
}

impl FindCondition {
    pub fn message(&self) -> &str {
        match self {
            FindCondition::Found => "The requested item was successfully found and applied.",
            FindCondition::NotFound(msg) => {
                if msg.is_empty() {
                    "Unfortunately, the item you are looking for does not exist or has already been deleted."
                } else {
                    msg
                }
            }
            FindCondition::FoundError(msg) => {
                if msg.is_empty() {
                    "The requested item was found, but an error occurred while modifying it. It might not match the type you require."
                } else {
                    msg
                }
            }
        }
    }
}
// ###################################################################################
// ####################### (▰˘◡˘▰) Work with AST import (▰˘◡˘▰) ######################
// ###################################################################################

#[derive(Debug)]
struct ASTVisitImport<'a> {
    code: &'a str,
    /// The dialect `code` is parsed in. It is the caller's source dialect, because the imports
    /// being added have to parse under the same rules as the file receiving them.
    dialect: Dialect,
    duplicate_imports: Vec<String>,
    none_duplicate_imports: Vec<String>,
    operation: Operation,
    /// Set when `code` cannot be parsed as JavaScript. `VisitMut` methods
    /// return `()`, so callers must check this once the visit completes and
    /// surface it as an error.
    parse_error: Option<String>,
}

impl Default for ASTVisitImport<'_> {
    fn default() -> Self {
        Self {
            code: "",
            dialect: Dialect::Js,
            duplicate_imports: Vec::new(),
            none_duplicate_imports: Vec::new(),
            operation: Operation::Edit,
            parse_error: None,
        }
    }
}

/// Returned when the module/import argument is not valid JavaScript. The
/// argument is parsed as source, so a bare filesystem path such as
/// `../vendor/topbar` is a syntax error.
const INVALID_ARGUMENT_MESSAGE: &str =
    "The given module or import argument could not be interpreted. Provide a full import \
     statement (`import topbar from \"../vendor/topbar\";`), or, when removing imports, \
     a bare module specifier (`../vendor/topbar`).";

/// Collects the module sources a removal request refers to.
///
/// The argument is first parsed as JavaScript. If it contains import
/// declarations, only the sources those import from are used, which keeps
/// multi-line import statements intact. Only when the argument holds no import
/// declaration at all is each non-empty line taken literally as a module
/// specifier, which is what lets a bare `topbar` or a path such as
/// `../vendor/topbar` work even though neither is a valid import statement.
fn removal_targets(code: &str) -> Vec<String> {
    if let Ok((parsed, _comments, _cm)) = parse(code) {
        let sources: Vec<String> = parsed
            .body
            .iter()
            .filter_map(|item| match item {
                ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) => {
                    Some(decl.src.value.to_string_lossy().to_string())
                }
                _ => None,
            })
            .collect();

        if !sources.is_empty() {
            return sources;
        }
    }

    code.lines()
        .map(|line| line.trim().trim_end_matches(';').trim())
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

impl VisitMut for ASTVisitImport<'_> {
    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        if matches!(self.operation, Operation::Delete) {
            let targets = removal_targets(self.code);
            let mut indices_to_remove = vec![];

            for (index, item) in items.iter().enumerate() {
                if let ModuleItem::ModuleDecl(ModuleDecl::Import(existing_import)) = item {
                    let source = existing_import.src.value.to_string_lossy().to_string();

                    if targets.contains(&source) {
                        indices_to_remove.push(index);
                    }
                }
            }

            for &index in indices_to_remove.iter().rev() {
                items.remove(index);
            }
        }

        items.visit_mut_children_with(self);
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        if !matches!(self.operation, Operation::Add | Operation::Read) {
            module.visit_mut_children_with(self);
            return;
        }

        // We are using it to add imports and know it is duplicated or not
        let imports = match parse_as(self.code, self.dialect) {
            Ok((imports, _comments, _cm)) => imports,
            Err(_) => {
                self.parse_error = Some(INVALID_ARGUMENT_MESSAGE.to_string());
                return;
            }
        };

        for import in imports.body {
            if !is_duplicate_import(&import, &module.body) {
                if matches!(self.operation, Operation::Add | Operation::Read) {
                    let mut last_import_index = None;
                    for (i, item) in module.body.iter().enumerate() {
                        if matches!(item, ModuleItem::ModuleDecl(ModuleDecl::Import(_))) {
                            last_import_index = Some(i);
                        }
                    }

                    for imp in import.as_module_decl().iter() {
                        if let ModuleDecl::Import(import_decl) = imp {
                            let src_value = import_decl.src.value.to_string_lossy().to_string();
                            if !self.none_duplicate_imports.contains(&src_value) {
                                self.none_duplicate_imports.push(src_value);
                            }
                        }
                    }

                    if let Some(index) = last_import_index {
                        module.body.insert(index + 1, import);
                    } else {
                        module.body.insert(0, import);
                    }
                }
            } else if matches!(self.operation, Operation::Read) {
                if let ModuleItem::ModuleDecl(ModuleDecl::Import(new_import_decl)) = import {
                    self.duplicate_imports
                        .push(new_import_decl.src.value.to_string_lossy().to_string());
                }
            }
        }

        module.visit_mut_children_with(self);
    }
}

/// Checks if a specific module is imported in the JavaScript source code.
///
/// This function parses the given JavaScript source code into an AST
/// and determines if the specified `module_name` is imported.
///
/// # Arguments
/// - `file_content`: The JavaScript source code as a string slice.
/// - `module_name`: The name of the module to check for imports.
///
/// # Returns
/// A `Result` containing `true` if the module is imported, `false` otherwise,
/// or an error message if parsing fails.
pub fn is_module_imported_from_ast(
    file_content: &str,
    module_name: &str,
    dialect: Dialect,
) -> Result<bool, bool> {
    let mut import_visitor = ASTVisitImport {
        dialect,
        code: module_name,
        operation: Operation::Read,
        ..Default::default()
    };

    let _output = code_gen_from_ast_vist_as(file_content, &mut import_visitor, dialect);

    if import_visitor.parse_error.is_some() {
        return Err(false);
    }

    if import_visitor.none_duplicate_imports.is_empty()
        && import_visitor.duplicate_imports.is_empty()
    {
        Err(false)
    } else if import_visitor.none_duplicate_imports.is_empty() {
        Ok(true)
    } else {
        Err(false)
    }
}

/// Inserts new import statements into JavaScript source code.
///
/// Parses the provided JavaScript source code into an AST, adds the specified
/// `import_lines` as import declarations, and ensures no duplicate imports are added.
/// Returns the updated JavaScript code as a string.
///
/// # Arguments
/// - `file_content`: The JavaScript source code as a string slice.
/// - `import_lines`: The new import lines to be added, separated by newlines.
///
/// # Returns
/// A `Result` containing the updated JavaScript code as a `String` on success,
/// or an error message if parsing or insertion fails.
///
/// # Behavior
/// - Ensures duplicate imports are skipped.
/// - Inserts new import statements after existing ones or at the top if none exist.
pub fn insert_import_to_ast(
    file_content: &str,
    import_lines: &str,
    dialect: Dialect,
) -> Result<String, String> {
    let mut import_visitor = ASTVisitImport {
        dialect,
        code: import_lines,
        operation: Operation::Add,
        ..Default::default()
    };

    let output = code_gen_from_ast_vist_as(file_content, &mut import_visitor, dialect)?;

    match import_visitor.parse_error {
        Some(error) => Err(error),
        None => Ok(output),
    }
}

/// Removes specified import statements from JavaScript source code.
///
/// Parses the given JavaScript source code into an AST, locates the specified
/// modules in the `modules` iterator, and removes their corresponding import
/// declarations. Returns the updated JavaScript code as a string.
///
/// # Arguments
/// - `file_content`: The JavaScript source code as a string slice.
/// - `modules`: The modules to remove, one per line. Each line may be either a
///   full import statement (`import topbar from "../vendor/topbar";`) or a bare
///   module specifier (`../vendor/topbar`). Matching is done on the module
///   source, not on the local binding name, so removing `topbar` will not drop
///   `import topbar from "../vendor/topbar"`.
///
/// # Returns
/// A `Result` containing the updated JavaScript code as a `String` on success,
/// or an error message if parsing fails.
///
/// # Behavior
/// - Retains all other import statements and code structure.
/// - Removes only the specified modules from the import declarations.
pub fn remove_import_from_ast(
    file_content: &str,
    modules: &str,
    dialect: Dialect,
) -> Result<String, String> {
    let mut import_visitor = ASTVisitImport {
        dialect,
        code: modules,
        operation: Operation::Delete,
        ..Default::default()
    };

    let output = code_gen_from_ast_vist_as(file_content, &mut import_visitor, dialect)?;

    match import_visitor.parse_error {
        Some(error) => Err(error),
        None => Ok(output),
    }
}

// ###################################################################################
// ##################### (▰˘◡˘▰) Work with AST Statistics (▰˘◡˘▰) ####################
// ###################################################################################
pub struct ASTStatistics {
    pub functions: usize,
    pub classes: usize,
    pub debuggers: usize,
    pub imports: usize,
    pub trys: usize,
    pub throws: usize,
    pub operation: Operation,
}

impl Default for ASTStatistics {
    fn default() -> Self {
        Self {
            functions: 0,
            classes: 0,
            debuggers: 0,
            imports: 0,
            trys: 0,
            throws: 0,
            operation: Operation::Read,
        }
    }
}

impl VisitMut for ASTStatistics {
    fn visit_mut_function(&mut self, node: &mut Function) {
        if matches!(self.operation, Operation::Read) {
            self.functions += 1;
        }
        node.visit_mut_children_with(self)
    }

    fn visit_mut_class(&mut self, node: &mut Class) {
        if matches!(self.operation, Operation::Read) {
            self.classes += 1;
        }
        node.visit_mut_children_with(self)
    }

    fn visit_mut_debugger_stmt(&mut self, node: &mut DebuggerStmt) {
        if matches!(self.operation, Operation::Read) {
            self.debuggers += 1;
        }
        node.visit_mut_children_with(self)
    }

    fn visit_mut_import_decl(&mut self, node: &mut ImportDecl) {
        if matches!(self.operation, Operation::Read) {
            self.imports += 1;
        }
        node.visit_mut_children_with(self)
    }

    fn visit_mut_try_stmt(&mut self, node: &mut TryStmt) {
        if matches!(self.operation, Operation::Read) {
            self.trys += 1;
        }
        node.visit_mut_children_with(self)
    }

    fn visit_mut_throw_stmt(&mut self, node: &mut ThrowStmt) {
        if matches!(self.operation, Operation::Read) {
            self.throws += 1;
        }
        node.visit_mut_children_with(self)
    }
}

/// Parses the given JavaScript source code and collects statistics about the AST nodes.
///
/// # Arguments
/// - `file_content`: A string slice containing the JavaScript source code.
///
/// # Returns
/// A result containing `ASTStatistics` with statistics about the parsed source code or an
/// error message if parsing fails.
///
/// # Example
/// ```rust
/// let result = statistics_from_ast(file_content);
/// assert!(result.is_ok());
/// ```
pub fn statistics_from_ast(file_content: &str, dialect: Dialect) -> Result<ASTStatistics, String> {
    let mut import_visitor = ASTStatistics {
        operation: Operation::Read,
        ..Default::default()
    };

    let _ = code_gen_from_ast_vist_as(file_content, &mut import_visitor, dialect);

    Ok(import_visitor)
}

// ###################################################################################
// ################### (▰˘◡˘▰) Work with AST Var and Object (▰˘◡˘▰) ##################
// ###################################################################################
struct ObjectExtender {
    target_var_name: String,
    new_properties: Vec<Prop>,
    operation: Operation,
    find: FindCondition,
}

impl Default for ObjectExtender {
    fn default() -> Self {
        Self {
            target_var_name: "".to_string(),
            new_properties: Vec::new(),
            operation: Operation::Edit,
            find: FindCondition::NotFound("".to_string()),
        }
    }
}

impl VisitMut for ObjectExtender {
    fn visit_mut_var_decl(&mut self, var_decl: &mut VarDecl) {
        if matches!(self.operation, Operation::Edit) {
            for decl in &mut var_decl.decls {
                if let Some(ident) = decl.name.as_ident() {
                    if ident.sym == self.target_var_name {
                        if let Some(init) = &mut decl.init {
                            self.find = FindCondition::FoundError("".to_string());
                            if let Expr::Object(obj_expr) = init.as_mut() {
                                if matches!(self.operation, Operation::Edit) {
                                    self.find = FindCondition::Found;
                                    let existing_keys: Vec<String> = obj_expr
                                        .props
                                        .iter()
                                        .filter_map(|prop| match prop {
                                            PropOrSpread::Prop(prop) => match &**prop {
                                                Prop::Shorthand(ident) => {
                                                    Some(ident.sym.to_string())
                                                }
                                                Prop::KeyValue(key_value) => match &key_value.key {
                                                    PropName::Ident(ident) => {
                                                        Some(ident.sym.to_string())
                                                    }
                                                    _ => None,
                                                },
                                                _ => None,
                                            },
                                            PropOrSpread::Spread(spread) => match &*spread.expr {
                                                Expr::Ident(ident) => {
                                                    Some(format!("...{}", ident.sym))
                                                }
                                                _ => None,
                                            },
                                        })
                                        .collect();

                                    let new_props: Vec<PropOrSpread> = self
                                        .new_properties
                                        .clone()
                                        .into_iter()
                                        .filter(|prop| {
                                            if let Prop::Shorthand(ident) = prop {
                                                !existing_keys.contains(&ident.sym.to_string())
                                            } else {
                                                true
                                            }
                                        })
                                        .map(|prop| PropOrSpread::Prop(Box::new(prop)))
                                        .collect();

                                    obj_expr.props.extend(new_props);
                                }
                            }
                        }
                    }
                }
            }
        }
        var_decl.visit_mut_children_with(self)
    }
}

/// Extends an object variable by adding new properties to its declaration in the JavaScript AST.
///
/// This function searches for a variable with the name `var_name` and modifies its
/// object properties by appending new ones from `object_names`. If the variable is found,
/// it modifies the AST and returns the updated JavaScript code.
///
/// # Arguments
/// * `file_content` - The JavaScript source code as a string.
/// * `var_name` - The name of the variable (object) to extend.
/// * `object_names` - A list of new property names to add to the object.
///
/// # Returns
/// * `Ok(String)` - The updated JavaScript source code with the extended object.
/// * `Err(String)` - If the target variable is not found or an error occurs.
///
/// # Errors
/// * Returns an error if the target variable does not exist in the AST.
/// * Returns an error if the AST transformation fails.
///
/// # Example
/// ```rust
/// let js_code = "let obj = { a: 1, b: 2 };";
/// let updated_code = extend_var_object_property_by_names_to_ast(js_code, "obj", ["c", "d"]);
/// assert!(updated_code.is_ok());
/// let result = updated_code.unwrap();
/// assert!(result.contains("c"));
/// assert!(result.contains("d"));
/// ```
pub fn extend_var_object_property_by_names_to_ast<'a>(
    file_content: &str,
    var_name: &str,
    object_names: impl IntoIterator<Item = &'a str> + Clone,
    dialect: Dialect,
) -> Result<String, String> {
    let new_properties: Vec<Prop> = object_names
        .into_iter()
        .map(|name| Prop::Shorthand(Ident::new(name.into(), DUMMY_SP, SyntaxContext::empty())))
        .collect();

    let mut object_extender = ObjectExtender {
        target_var_name: var_name.to_string(),
        new_properties,
        operation: Operation::Edit,
        ..Default::default()
    };

    let result = code_gen_from_ast_vist_as(file_content, &mut object_extender, dialect);
    if object_extender.find == FindCondition::Found {
        result
    } else {
        Err(object_extender.find.message().to_string())
    }
}

/// Checks if a given variable is declared in the JavaScript AST.
///
/// This function parses the provided JavaScript `file_content` and searches for
/// a variable declaration (`let`) with the specified `variable_name`. If the variable
/// is found, it returns `Ok(true)`, otherwise, it returns `Err(false)`.
///
/// # Arguments
/// * `file_content` - The JavaScript source code as a string.
/// * `variable_name` - The name of the variable to search for.
///
/// # Returns
/// * `Ok(true)` - If the variable is found.
/// * `Err(false)` - If the variable is not found or an error occurs during parsing.
///
/// # Errors
/// * Returns `Err(false)` if the variable is not found in the AST.
///
/// # Example
/// ```rust
/// let js_code = "let myVar = 42;";
/// let result = contains_variable_from_ast(js_code, "myVar");
/// assert_eq!(result, Ok(true));
///
/// let result = contains_variable_from_ast(js_code, "anotherVar");
/// assert_eq!(result, Err(false));
/// ```
pub fn contains_variable_from_ast(
    file_content: &str,
    variable_name: &str,
    dialect: Dialect,
) -> Result<bool, bool> {
    let (module, _, _) = match parse_as(file_content, dialect) {
        Ok(result) => result,
        Err(_) => return Err(false),
    };

    for item in &module.body {
        if let ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl))) = item {
            for decl in &var_decl.decls {
                if let Pat::Ident(BindingIdent { id, .. }) = &decl.name {
                    if id.sym == variable_name {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Err(false)
}

/// Inserts a new JavaScript AST at a specified index in the existing AST.
///
/// This function takes an existing JavaScript source code (`file_content`) and inserts
/// the AST of `insert_code` at the given `index`.
///
/// # Arguments
/// * `file_content` - The original JavaScript source code as a string.
/// * `insert_code` - The JavaScript code whose AST will be inserted.
/// * `index` - The zero-based index where the new AST should be inserted.
///
/// # Returns
/// * `Ok(String)` - The updated JavaScript source code after insertion.
/// * `Err(String)` - An error message if the index is out of bounds.
///
/// # Errors
/// * Returns `"Index out of bounds"` if the given `index` is greater than the number of AST nodes.
/// * Returns `"Failed to parse module"` if either `file_content` or `insert_code` cannot be parsed.
///
/// # Example
/// ```rust
/// let file_content = "function a() {} function b() {}";
/// let insert_code = "function newFunc() {}";
///
/// // Insert after index 1 (after function `a`)
/// let result = insert_ast_at_index(file_content, insert_code, 1);
/// assert!(result.is_ok());
/// let updated_code = result.unwrap();
/// assert!(updated_code.contains("newFunc"));
///
/// let result = insert_ast_at_index(file_content, insert_code, 0);
/// assert!(result.is_ok());
/// let updated_code = result.unwrap();
/// assert!(updated_code.starts_with("function newFunc"));
/// ```
pub fn insert_ast_at_index(
    file_content: &str,
    insert_code: &str,
    index: usize,
    dialect: Dialect,
) -> Result<String, String> {
    let (mut module, comments, cm) = parse_as(file_content, dialect)?;
    let (insert_module, _, _) = parse_as(insert_code, dialect)?;

    if index > module.body.len() {
        return Err("Index out of bounds".to_string());
    }

    module.body.splice(index..index, insert_module.body);

    code_gen_from_ast_module(&mut module, comments, cm)
}

/// Replaces the AST node at a specified index with a new JavaScript AST.
///
/// This function takes an existing JavaScript source code (`file_content`) and replaces
/// the AST node at the given `index` with the parsed AST of `replace_code`.
/// If the `index` is out of bounds or the replacement AST is empty, it returns an error.
///
/// # Arguments
/// * `file_content` - The original JavaScript source code as a string.
/// * `replace_code` - The JavaScript code whose AST will replace the existing one.
/// * `index` - The zero-based index of the AST node to be replaced.
///
/// # Returns
/// * `Ok(String)` - The updated JavaScript source code after replacement.
/// * `Err(String)` - An error message if the index is out of bounds or the replacement AST is empty.
///
/// # Errors
/// * Returns `"Index out of bounds"` if the given `index` is greater than or equal to the number of AST nodes.
/// * Returns `"Replacement AST is empty"` if the `replace_code` does not produce a valid AST.
///
/// # Example
/// ```rust
/// let file_content = "function a() {} function b() {}";
/// let replace_code = "function newFunc() {}";
/// let result = replace_ast_at_index(file_content, replace_code, 1);
///
/// assert!(result.is_ok());
/// let updated_code = result.unwrap();
/// assert!(updated_code.contains("newFunc"));
/// ```
pub fn replace_ast_at_index(
    file_content: &str,
    replace_code: &str,
    index: usize,
    dialect: Dialect,
) -> Result<String, String> {
    let (mut module, comments, cm) = parse_as(file_content, dialect)?;
    let (replace_module, _, _) = parse_as(replace_code, dialect)?;

    if index >= module.body.len() {
        return Err("Index out of bounds".to_string());
    }

    if replace_module.body.is_empty() {
        return Err("Replacement AST is empty".to_string());
    }

    module.body.splice(index..=index, replace_module.body);

    code_gen_from_ast_module(&mut module, comments, cm)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// Adding and querying still need real import statements, so a bare
    /// specifier must surface as an error rather than a panic.
    #[test]
    fn test_unparseable_argument_returns_error_instead_of_panicking() {
        let code = "import topbar from \"../vendor/topbar\";\nlet Hooks = {};\n";
        let invalid = "../vendor/topbar";

        let result = insert_import_to_ast(code, invalid, Dialect::Js);
        assert!(
            result.is_err(),
            "insert_import_to_ast should error, got {:?}",
            result
        );

        let result = is_module_imported_from_ast(code, invalid, Dialect::Js);
        assert_eq!(
            result,
            Err(false),
            "is_module_imported_from_ast should report not-imported"
        );
    }

    /// The same guarantee for unparseable source rather than an unparseable
    /// argument.
    #[test]
    fn test_unparseable_source_returns_error_instead_of_panicking() {
        let broken = "let x = ;;; import * from;";

        assert_eq!(contains_variable_from_ast(broken, "x", Dialect::Js), Err(false));
        assert!(remove_import_from_ast(broken, "topbar", Dialect::Js).is_err());
    }

    const TWO_IMPORTS: &str =
        "import { Socket } from \"phoenix\";\nimport topbar from \"../vendor/topbar\";\nlet Hooks = {};\n";

    #[test]
    fn test_remove_import_by_bare_module_specifier() {
        let result = remove_import_from_ast(TWO_IMPORTS, "../vendor/topbar", Dialect::Js).unwrap();

        assert!(!result.contains("topbar"), "got: {result}");
        assert!(result.contains("phoenix"), "got: {result}");
        assert!(result.contains("let Hooks"), "got: {result}");
    }

    #[test]
    fn test_remove_import_by_full_statement_still_works() {
        let result =
            remove_import_from_ast(
                TWO_IMPORTS,
                "import topbar from \"../vendor/topbar\";",
                Dialect::Js,
            )
            .unwrap();

        assert!(!result.contains("topbar"), "got: {result}");
        assert!(result.contains("phoenix"), "got: {result}");
    }

    #[test]
    fn test_remove_several_bare_specifiers_one_per_line() {
        let result = remove_import_from_ast(TWO_IMPORTS, "phoenix\n../vendor/topbar", Dialect::Js).unwrap();

        assert!(!result.contains("topbar"), "got: {result}");
        assert!(!result.contains("phoenix"), "got: {result}");
        assert!(result.contains("let Hooks"), "got: {result}");
    }

    /// A multi-line import statement must not have its inner lines mistaken for
    /// bare module specifiers.
    #[test]
    fn test_multiline_import_statement_does_not_leak_literal_targets() {
        let code = "import x from \"foo\";\nimport { foo } from \"module-name\";\n";
        let argument = "import {\n  foo\n} from \"module-name\";";

        let result = remove_import_from_ast(code, argument, Dialect::Js).unwrap();

        assert!(result.contains("import x from \"foo\";"), "got: {result}");
        assert!(!result.contains("module-name"), "got: {result}");
    }

    /// Matching is on the module source, so the local binding name is not a
    /// removal key.
    #[test]
    fn test_remove_import_does_not_match_local_binding_name() {
        let result = remove_import_from_ast(TWO_IMPORTS, "topbar", Dialect::Js).unwrap();

        assert!(result.contains("../vendor/topbar"), "got: {result}");
    }

    #[test]
    fn test_remove_import_of_an_absent_module_is_a_no_op() {
        let result = remove_import_from_ast(TWO_IMPORTS, "not-imported", Dialect::Js).unwrap();

        assert_eq!(result.trim(), TWO_IMPORTS.trim());
    }

    #[test]
    fn test_remove_import_with_a_blank_argument_is_a_no_op() {
        let result = remove_import_from_ast(TWO_IMPORTS, "   ", Dialect::Js).unwrap();

        assert_eq!(result.trim(), TWO_IMPORTS.trim());
    }

    #[test]
    fn test_is_module_imported_from_ast() {
        let code = r#"
            import "phoenix_html";
            import { Socket, SocketV1 } from "phoenix";
            import { TS } from "tsobject";

            // This is first test we need to have
            console.log("We are here");

            const min = ()          => {return "Shahryar" + "Tavakkoli"};
            "#;

        let import = r#"
                import "phoenix_html";
                import { Socket, SocketV1 } from "phoenix";
                import { TS } from "tsobject";
            "#;
        let result = is_module_imported_from_ast(code, import, Dialect::Js);

        assert!(result.is_ok(), "Expected Ok(true), but got {:?}", result);

        let import = r#"
                import { NoneRepeated } from "orepeat";
            "#;
        let result = is_module_imported_from_ast(code, import, Dialect::Js);
        assert!(result.is_err(), "Expected Ok(true), but got {:?}", result);

        let import = r#"
                import "phoenix_html";
                import { NoneRepeated } from "orepeat";
                import { TS } from "tsobject";
            "#;
        let result = is_module_imported_from_ast(code, import, Dialect::Js);

        assert!(result.is_err(), "Expected Ok(true), but got {:?}", result);
    }
    #[test]
    fn test_insert_import_to_ast() {
        let code = r#"
            import "phoenix_html";
            import { Socket, SocketV1 } from "phoenix";
            import { TS } from "tsobject";
            import ScrollArea from "./scrollArea.js";

            // This is first test we need to have
            console.log("We are here");

            const min = ()          => {return "Shahryar" + "Tavakkoli"};
            "#;

        let import = r#"
                import "phoenix_html";
                import { Socket, SocketV1 } from "phoenix";
                import { TS } from "tsobject";
                import { NoneRepeated } from "orepeat";
                import ScrollArea from "./scrollArea.js";
            "#;
        let result = insert_import_to_ast(code, import, Dialect::Js).expect("Failed to generate code");

        assert!(result.contains("import \"phoenix_html\";"));
        assert!(result.contains("import { Socket, SocketV1 } from \"phoenix\";"));
        assert!(result.contains("import { TS } from \"tsobject\";"));
        assert!(result.contains("import { NoneRepeated } from \"orepeat\";"));

        let imports_start = result.find("import \"phoenix_html\";").unwrap();
        let imports_end = result
            .find("import { NoneRepeated } from \"orepeat\";")
            .unwrap();
        assert!(imports_start < imports_end);

        assert!(result.contains("// This is first test we need to have"));

        println!("{}", result)
    }

    #[test]
    fn test_remove_import_from_ast() {
        let code = r#"
            import "phoenix_html";
            import { Socket, SocketV1 } from "phoenix";
            import { TS } from "tsobject";

            // This is first test we need to have
            console.log("We are here");

            const min = ()          => {return "Shahryar" + "Tavakkoli"};
            "#;

        let import = r#"
                import { TS } from "tsobject";
                import { Socket, SocketV1 } from "phoenix";
                import { NoneRepeated } from "orepeat";
                import { NoneRepeated1 } from "orepeat1";
            "#;
        let result = remove_import_from_ast(code, import, Dialect::Js).expect("Failed to generate code");

        assert!(result.contains("import \"phoenix_html\";"));
        assert!(!result.contains("import { Socket, SocketV1 } from \"phoenix\";"));
        assert!(!result.contains("import { TS } from \"tsobject\";"));

        let code = r#"
        import { foo } from "module-name";
        import bar from "another-module";
        let Hooks = {};
        "#;

        let result = remove_import_from_ast(code, "import bar from \"another-module\";", Dialect::Js)
            .expect("Failed to generate code");

        println!("{}", result);
    }

    #[test]
    fn test_statistics_from_ast() {
        let code = r#"
            import { foo } from 'bar';
            import * as jar from 'jar';
            console.log('Start JS file');
            class Foo {
                constructor() {
                    debugger;
                    console.log('Hello');
                }
            }
            function bar() {
                console.log('World');
                debugger;
            }
        "#;
        let parsed = statistics_from_ast(code, Dialect::Js).unwrap();
        assert_eq!(parsed.functions, 1);
        assert_eq!(parsed.classes, 1);
        assert_eq!(parsed.debuggers, 2);
        assert_eq!(parsed.imports, 2);
        assert_eq!(parsed.trys, 0);
        assert_eq!(parsed.throws, 0);
    }

    #[test]
    fn test_extend_var_object_property_by_names_to_ast() {
        let code = r#"
            const Components = {...Hoks, PreOrderd};

            // Export the components as default
            export default Components;
            "#;

        let object_names = [
            "...ExtendedObject".to_string(),
            "ObjectOne".to_string(),
            "PreOrderd".to_string(),
            "CopyCodeHooks".to_string(),
            "...Hoks".to_string(),
            "ObjectOne".to_string(),
        ];
        let unique_names: HashSet<String> = object_names.into_iter().collect();
        let mut vec_of_strs: Vec<&str> = unique_names.iter().map(|s| s.as_str()).collect();
        vec_of_strs.sort();

        let result =
            extend_var_object_property_by_names_to_ast(code, "Components", vec_of_strs.clone(), Dialect::Js);
        assert!(result.is_ok());
        println!("{}", result.unwrap());

        let result =
            extend_var_object_property_by_names_to_ast(code, "NoneComponent", vec_of_strs.clone(), Dialect::Js);
        assert!(result.is_err());

        let code = r#"
            const Components = () => {1 + 1};

            // Export the components as default
            export default Components;
            "#;

        let result =
            extend_var_object_property_by_names_to_ast(code, "Components", vec_of_strs.clone(), Dialect::Js);
        assert!(result.is_err());

        let code = r#"
            import ScrollArea from "./scrollArea.js";

            const Components = {
              ScrollArea,
            };

            export default Components;
            "#;

        let object_names = ["ScrollArea", "NoneComponent"];

        let result = extend_var_object_property_by_names_to_ast(code, "Components", object_names, Dialect::Js);
        assert!(result.is_ok());

        let code = r#"
            import ScrollArea from "./scrollArea.js";

            const Components = {};

            export default Components;
            "#;

        let object_names = ["ScrollArea", "NoneComponent", "...NoneComponent"];
        let result = extend_var_object_property_by_names_to_ast(code, "Components", object_names, Dialect::Js);

        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_variable_from_ast() {
        let code = r#"
            let liveSocket = new LiveSocket("/live", Socket, {
              hooks: { ...Hooks, CopyMixInstallationHook },
              longPollFallbackMs: 2500,
              params: { _csrf_token: csrfToken },
            });
            "#;

        let result = contains_variable_from_ast(code, "liveSocket", Dialect::Js);

        println!("{:#?}", result.unwrap())
    }

    mod index_operations {
        use super::*;

        #[test]
        fn test_insert_ast_at_index() {
            let file_content = "function a() {} function b() {}";
            let insert_code = "function newFunc() {}";

            let result = insert_ast_at_index(file_content, insert_code, 1, Dialect::Js);

            assert!(result.is_ok());
            let updated_ast = result.unwrap();
            println!("{}", updated_ast);

            let code = r#"
                let liveSocket = new LiveSocket("/live", Socket, {
                  hooks: { ...Hooks, CopyMixInstallationHook },
                  longPollFallbackMs: 2500,
                  params: { _csrf_token: csrfToken },
                });

                const newFunc = () => {
                  console.log('New function called');
                };

                let newVar = 'Hello';
                "#;

            let insert_code = r#"
                function addedNewFunc1() {
                  console.log('addedNewFunc1 called');
                }

                function addedNewFunc2() {
                  console.log('addedNewFunc2 called');
                }
            "#;

            let result = insert_ast_at_index(code, insert_code, 0, Dialect::Js);

            assert!(result.is_ok());

            let updated_ast = result.unwrap();
            println!("{}", updated_ast);
        }

        #[test]
        fn test_insert_ast_at_index_out_of_bounds() {
            let file_content = "function a() {}";
            let insert_code = "function newFunc() {}";

            let result = insert_ast_at_index(file_content, insert_code, 5, Dialect::Js);

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Index out of bounds");

            let file_content = r#"
                function addedNewFunc1() {
                  console.log('addedNewFunc1 called');
                }

                function addedNewFunc2() {
                  console.log('addedNewFunc2 called');
                }
            "#;
            let insert_code = "function newFunc() {}";

            let result = insert_ast_at_index(file_content, insert_code, 3, Dialect::Js);

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Index out of bounds");
        }

        #[test]
        fn test_replace_ast_at_index() {
            let file_content = "function a() {} function b() {}";
            let insert_code = "function newFunc() {}";

            let result = replace_ast_at_index(file_content, insert_code, 0, Dialect::Js);

            assert!(result.is_ok());
            let updated_ast = result.unwrap();
            println!("{}", updated_ast);

            let code = r#"
                let liveSocket = new LiveSocket("/live", Socket, {
                  hooks: { ...Hooks, CopyMixInstallationHook },
                  longPollFallbackMs: 2500,
                  params: { _csrf_token: csrfToken },
                });

                const newFunc = () => {
                  console.log('New function called');
                };

                let newVar = 'Hello';
                "#;

            let insert_code = r#"
                function addedNewFunc1() {
                  console.log('addedNewFunc1 called');
                }

                function addedNewFunc2() {
                  console.log('addedNewFunc2 called');
                }
            "#;

            let result = replace_ast_at_index(code, insert_code, 2, Dialect::Js);

            assert!(result.is_ok());

            let updated_ast = result.unwrap();
            println!("{}", updated_ast);
        }

        #[test]
        fn test_replace_ast_at_index_of_bounds() {
            let file_content = "function a() {}";
            let insert_code = "function newFunc() {}";

            let result = replace_ast_at_index(file_content, insert_code, 5, Dialect::Js);

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Index out of bounds");
        }
    }
}

// Sample code
// ---------------------------------------------
// struct RenameFunction;

// impl VisitMut for RenameFunction {
//     fn visit_mut_fn_decl(&mut self, node: &mut FnDecl) {
//         if node.ident.sym == "add" {
//             node.ident.sym = "adds".into();
//         }
//         node.visit_mut_children_with(self);
//     }

//     fn visit_mut_var_decl(&mut self, node: &mut VarDecl) {
//         for decl in &mut node.decls {
//             println!("{:#?}", decl);
//             if let Pat::Ident(ident) = &mut decl.name {
//                 if ident.id.sym == "add" {
//                     ident.id.sym = "adds".into();
//                     if let Some(init) = &mut decl.init {
//                         if let Expr::Arrow(_arrow_expr) = &**init {}
//                     }
//                 }
//             }
//         }
//         node.visit_mut_children_with(self);
//     }
// }

// pub fn change_var_name(file_content: &str) -> String {
//     let rename_function = RenameFunction;
//     let output = code_gen_from_ast_vist(file_content, rename_function);
//     println!("{}", output);
//     output
// }
// let new_import = ImportDecl {
//     span: DUMMY_SP,
//     specifiers: vec![],
//     src: Box::new(Str {
//         span: DUMMY_SP,
//         value: "module_name_test".into(),
//         raw: None,
//     }),
//     type_only: false,
//     phase: ImportPhase::Evaluation,
//     with: None,
// };
