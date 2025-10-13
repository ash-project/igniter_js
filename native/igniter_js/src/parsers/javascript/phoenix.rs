// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
//
// SPDX-License-Identifier: MIT

//! # Phoenix Hook Helpers
//!
//! This module provides helper functions and utilities for working with Phoenix LiveView hooks
//! in JavaScript code.
//! It includes functionality to extend, modify, and remove objects from the `hooks` object in
//! `liveSocket` initialization.
//! Designed specifically for manipulating the JavaScript Abstract Syntax Tree (AST) using SWC.

use crate::parsers::javascript::helpers::*;

use super::ast::{FindCondition, Operation};
use swc_common::{SyntaxContext, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub struct HookExtender<'a> {
    target_var_name: &'a str,
    new_objects: Vec<&'a str>,
    operation: Operation,
    find: FindCondition,
}

impl<'a> HookExtender<'a> {
    pub fn new(target_var_name: &'a str, new_objects: Vec<&'a str>) -> Self {
        Self {
            target_var_name,
            new_objects,
            find: FindCondition::NotFound("".to_string()),
            operation: Operation::Edit,
        }
    }

    fn extend_or_create_hooks(&mut self, obj_expr: &mut ObjectLit) {
        // Find the hooks property
        let hooks_prop_index = obj_expr.props.iter().position(|prop| {
            if let PropOrSpread::Prop(prop) = prop {
                if let Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(ident),
                    ..
                }) = &**prop
                {
                    return ident.sym == *"hooks";
                }
            }
            false
        });

        if let Some(index) = hooks_prop_index {
            // Get the hooks property
            if let PropOrSpread::Prop(prop) = &mut obj_expr.props[index] {
                if let Prop::KeyValue(KeyValueProp { value, .. }) = &mut **prop {
                    match &mut **value {
                        // Case 1: hooks is an inline object literal
                        Expr::Object(hooks_obj) => {
                            // Extend existing inline object
                            for new_object in &self.new_objects {
                                let already_exists =
                                    hooks_obj.props.iter().any(|prop| match prop {
                                        PropOrSpread::Prop(prop) => {
                                            if let Prop::Shorthand(ident) = &**prop {
                                                ident.sym == *new_object
                                            } else {
                                                false
                                            }
                                        }
                                        PropOrSpread::Spread(spread) => {
                                            if let Expr::Ident(ident) = &*spread.expr {
                                                let spread_sym = format!("...{}", ident.sym);
                                                spread_sym == *new_object
                                            } else {
                                                false
                                            }
                                        }
                                    });

                                if !already_exists {
                                    hooks_obj.props.push(PropOrSpread::Prop(Box::new(
                                        Prop::Shorthand(Ident::new(
                                            (*new_object).into(),
                                            DUMMY_SP,
                                            SyntaxContext::empty(),
                                        )),
                                    )));
                                }
                            }
                        }
                        // Case 2: hooks is an identifier reference (e.g., hooks: hooks)
                        Expr::Ident(ident) => {
                            // Create a new object with spread of the original identifier
                            let mut new_props = vec![PropOrSpread::Spread(SpreadElement {
                                dot3_token: DUMMY_SP,
                                expr: Box::new(Expr::Ident(ident.clone())),
                            })];

                            // Add the new objects
                            for new_object in &self.new_objects {
                                new_props.push(PropOrSpread::Prop(Box::new(Prop::Shorthand(
                                    Ident::new(
                                        (*new_object).into(),
                                        DUMMY_SP,
                                        SyntaxContext::empty(),
                                    ),
                                ))));
                            }

                            // Replace the value with the new object
                            **value = Expr::Object(ObjectLit {
                                span: DUMMY_SP,
                                props: new_props,
                            });
                        }
                        _ => {
                            // Other expressions - we don't handle these
                        }
                    }
                }
            }
        } else {
            // Create hooks if it doesn't exist
            let new_hooks = ObjectLit {
                span: DUMMY_SP,
                props: self
                    .new_objects
                    .iter()
                    .map(|name| {
                        PropOrSpread::Prop(Box::new(Prop::Shorthand(Ident::new(
                            (*name).into(),
                            DUMMY_SP,
                            SyntaxContext::empty(),
                        ))))
                    })
                    .collect(),
            };

            obj_expr
                .props
                .push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(
                        Ident::new("hooks".into(), DUMMY_SP, SyntaxContext::empty()).into(),
                    ),
                    value: Box::new(Expr::Object(new_hooks)),
                }))));
        }
    }

    fn remove_objects_from_hooks(
        &mut self,
        obj_expr: &mut ObjectLit,
        objects_to_remove: Vec<&str>,
    ) {
        if let Some(hooks_property) = obj_expr.props.iter_mut().find_map(|prop| {
            if let PropOrSpread::Prop(prop) = prop {
                if let Prop::KeyValue(KeyValueProp {
                    key: PropName::Ident(ident),
                    value,
                }) = &mut **prop
                {
                    if ident.sym == *"hooks" {
                        if let Expr::Object(obj_expr) = &mut **value {
                            return Some(obj_expr);
                        }
                    }
                }
            }
            None
        }) {
            hooks_property.props.retain(|prop| match prop {
                PropOrSpread::Prop(prop) => {
                    if let Prop::Shorthand(ident) = &**prop {
                        !objects_to_remove.contains(&&*ident.sym)
                    } else {
                        true
                    }
                }
                PropOrSpread::Spread(spread) => {
                    if let Expr::Ident(ident) = &*spread.expr {
                        !objects_to_remove.contains(&format!("...{}", ident.sym).as_str())
                    } else {
                        true
                    }
                }
            });
        }
    }
}

impl VisitMut for HookExtender<'_> {
    fn visit_mut_var_decl(&mut self, var_decl: &mut VarDecl) {
        if matches!(self.operation, Operation::Edit) {
            for decl in &mut var_decl.decls {
                if let Some(ident) = decl.name.as_ident() {
                    if ident.sym == self.target_var_name {
                        if let Some(init) = &mut decl.init {
                            if let Expr::New(new_expr) = init.as_mut() {
                                if let Expr::Ident(callee_ident) = &*new_expr.callee {
                                    if callee_ident.sym == "LiveSocket" {
                                        self.find = FindCondition::FoundError("".to_string());

                                        if let Some(args) = &mut new_expr.args {
                                            if let Some(ExprOrSpread { expr, .. }) = args.last_mut()
                                            {
                                                if let Expr::Object(obj_expr) = &mut **expr {
                                                    self.find = FindCondition::Found;
                                                    self.extend_or_create_hooks(obj_expr);
                                                }
                                            }
                                        }
                                    }
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

/// Extends the `hooks` object in the JavaScript AST by adding new properties.
///
/// This function parses the given JavaScript source code, checks for the existence
/// of a `liveSocket` variable, and adds new properties to the `hooks` object.
/// If the `hooks` object or `liveSocket` variable is not found, it initializes
/// or returns an appropriate error.
///
/// # Arguments
/// - `file_content`: The JavaScript source code as a string slice.
/// - `names`: An iterable collection of property names to be added to the `hooks` object.
///
/// # Returns
/// A `Result` containing the updated JavaScript code as a `String` on success,
/// or an error message if parsing or manipulation fails.
///
/// # Behavior
/// - Checks for the presence of `liveSocket` in the AST.
/// - Finds or initializes the `hooks` object in the AST.
/// - Adds new properties to the `hooks` object without duplicating existing ones.
///
/// Warning: If you use the spread operator (e.g., ..Hooks) multiple times, the code does
/// not deduplicate it, and it will include each occurrence as is.
pub fn extend_hook_object_to_ast(
    file_content: &str,
    new_objects: Vec<&str>,
) -> Result<String, String> {
    let mut hook_extender = HookExtender::new("liveSocket", new_objects);

    let result = code_gen_from_ast_vist(file_content, &mut hook_extender);
    if hook_extender.find == FindCondition::Found {
        result
    } else {
        Err(hook_extender.find.message().to_string())
    }
}

pub fn find_live_socket_node_from_ast(file_content: &str) -> Result<bool, bool> {
    let mut hook_extender = HookExtender::new("liveSocket", vec![]);
    let _result = code_gen_from_ast_vist(file_content, &mut hook_extender);
    if hook_extender.find == FindCondition::Found {
        Ok(true)
    } else {
        Err(false)
    }
}

/// Removes specified objects from the `hooks` object in the JavaScript AST.
///
/// This function parses the given JavaScript source code, checks for the presence of a
/// `liveSocket` variable, and removes specified properties from the `hooks` object.
/// If the `hooks` object or `liveSocket` variable is not found, an appropriate error is returned.
///
/// # Arguments
/// - `file_content`: The JavaScript source code as a string slice.
/// - `objects_to_remove`: An iterable collection of object names (as strings) to be removed from the `hooks` object.
///
/// # Returns
/// A `Result` containing the updated JavaScript code as a `String` on success,
/// or an error message if parsing or manipulation fails.
///
/// # Behavior
/// - Ensures the `liveSocket` variable exists in the AST.
/// - Locates the `hooks` object or initializes it if absent.
/// - Removes specified properties from the `hooks` object while retaining all others.
pub fn remove_objects_of_hooks_from_ast(
    file_content: &str,
    objects_to_remove: Vec<&str>,
) -> Result<String, String> {
    let mut hook_extender = HookExtender::new("liveSocket", vec![]);

    let (mut module, comments, cm) = parse(file_content).expect("Failed to parse imports");

    module.visit_mut_with(&mut hook_extender);

    for item in &mut module.body {
        if let ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl))) = item {
            for decl in &mut var_decl.decls {
                if let Some(init) = &mut decl.init {
                    if let Expr::New(new_expr) = init.as_mut() {
                        if let Some(args) = &mut new_expr.args {
                            if let Some(ExprOrSpread { expr, .. }) = args.last_mut() {
                                if let Expr::Object(obj_expr) = &mut **expr {
                                    hook_extender.remove_objects_from_hooks(
                                        obj_expr,
                                        objects_to_remove.clone(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let result = code_gen_from_ast_module(&mut module, comments, cm);
    if hook_extender.find == FindCondition::Found {
        Ok(result)
    } else {
        Err(hook_extender.find.message().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extend_hook_object_to_ast() {
        let code = r#"
        let liveSocket = new LiveSocket("/live", Socket, {
          hooks: { ...Hooks, CopyMixInstallationHook },
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let new_objects = vec!["ObjectOne", "CopyMixInstallationHook", "ObjectTwo"];
        let result = extend_hook_object_to_ast(code, new_objects);
        assert!(result.is_ok());

        let code = r#"
        let NoneSocket = new LiveSocket("/live", Socket, {
          hooks: { ...Hooks, CopyMixInstallationHook },
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let new_objects = vec!["ObjectOne", "CopyMixInstallationHook", "ObjectTwo"];
        let result = extend_hook_object_to_ast(code, new_objects);
        assert!(result.is_err());

        let code = r#"
        let liveSocket = new LiveNoneSocket("/live", Socket, {
          hooks: { ...Hooks, CopyMixInstallationHook },
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let new_objects = vec!["ObjectOne", "CopyMixInstallationHook", "ObjectTwo"];
        let result = extend_hook_object_to_ast(code, new_objects);
        assert!(result.is_err());

        let code = r#"
        let liveSocket = new LiveSocket("/live", Socket, {
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let new_objects = vec!["ObjectOne", "CopyMixInstallationHook", "ObjectTwo"];
        let result = extend_hook_object_to_ast(code, new_objects);
        assert!(result.is_ok());

        let code = r#"
        let liveSocket = new LiveSocket("/live", Socket, {
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let new_objects = vec!["ObjectOne", "CopyMixInstallationHook", "...ObjectTwo"];
        let result = extend_hook_object_to_ast(code, new_objects);
        assert!(result.is_ok());

        let code = r#"
        let liveSocket = new LiveSocket("/live", Socket, {
          hooks: { ...Hooks, ObjectOneTwo, ...CopyMixInstallationHook },
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let new_objects = vec![
            "ObjectOne",
            "...CopyMixInstallationHook",
            "ObjectOneTwo",
            "...CopyMixInstallationHook",
        ];
        let result = extend_hook_object_to_ast(code, new_objects);
        assert!(result.is_ok());
        println!("{}", result.unwrap())
    }

    #[test]
    fn test_find_live_socket_node_from_ast() {
        let code = r#"
        let liveSocket = new LiveSocket("/live", Socket, {
          hooks: { ...Hooks, CopyMixInstallationHook },
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let result = find_live_socket_node_from_ast(code);
        assert!(result.is_ok());

        let code = r#"
        let liveNoneSocket = new LiveSocket("/live", Socket, {
          hooks: { ...Hooks, CopyMixInstallationHook },
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let result = find_live_socket_node_from_ast(code);
        assert!(result.is_err());

        let code = r#"
        let liveSocket = {};
        "#;

        let result = find_live_socket_node_from_ast(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_extend_hook_object_with_identifier_reference() {
        // Test case where hooks is referenced as an identifier rather than inline object
        let code = r#"
        let hooks = { ...colocatedHooks, KeepScrollPosition };
        hooks.map = mapHook;
        hooks.datalist = datalistHook;
        hooks.WebsitePreview = WebsitePreview;
        hooks.TreeSelect = TreeSelect;

        window.phxHooks = hooks;

        const csrfToken = document
          .querySelector("meta[name='csrf-token']")
          .getAttribute("content");
        const liveSocket = new LiveSocket("/live", Socket, {
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
          hooks: hooks,
          sessionStorage: process.env.NODE_ENV === "development",
        });
        "#;

        let new_objects = vec!["MishkaHooks", "OXCTestHook"];
        let result = extend_hook_object_to_ast(code, new_objects);
        assert!(result.is_ok());

        let updated_code = result.unwrap();
        println!("Updated code:\n{}", updated_code);

        // The result should have hooks: {...hooks, MishkaHooks, OXCTestHook} instead of duplicate hooks property
        assert!(updated_code.contains("hooks: {"));
        assert!(updated_code.contains("...hooks"));
        assert!(updated_code.contains("MishkaHooks"));
        assert!(updated_code.contains("OXCTestHook"));
        // Should not have duplicate hooks property
        assert_eq!(updated_code.matches("hooks:").count(), 1);
    }

    #[test]
    fn test_extend_hooks_with_key_value_properties() {
        // Test extending hooks that has key-value properties (not just shorthand)
        let code = r#"
        const liveSocket = new LiveSocket("/live", Socket, {
          hooks: {
            "StringKey": MyHook,
            normalKey: AnotherHook,
            123: NumericHook
          },
          params: { _csrf_token: csrfToken }
        });
        "#;

        let new_objects = vec!["NewHook"];
        let result = extend_hook_object_to_ast(code, new_objects);
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert!(updated.contains("NewHook"));
    }

    #[test]
    fn test_extend_hooks_with_const_let_var_declarations() {
        // Test with different variable declaration types
        let test_cases = vec![
            ("const", r#"const liveSocket = new LiveSocket("/live", Socket, { hooks: {} });"#),
            ("let", r#"let liveSocket = new LiveSocket("/live", Socket, { hooks: {} });"#),
            ("var", r#"var liveSocket = new LiveSocket("/live", Socket, { hooks: {} });"#),
        ];

        for (decl_type, code) in test_cases {
            let result = extend_hook_object_to_ast(code, vec!["TestHook"]);
            assert!(result.is_ok(), "Failed for {} declaration", decl_type);
            assert!(result.unwrap().contains("TestHook"));
        }
    }

    #[test]
    fn test_extend_hooks_preserves_other_properties() {
        // Ensure other LiveSocket properties remain untouched
        let code = r#"
        const liveSocket = new LiveSocket("/live", Socket, {
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
          hooks: { ExistingHook },
          dom: { onBeforeElUpdated: () => {} },
          metadata: { key: "value" }
        });
        "#;

        let result = extend_hook_object_to_ast(code, vec!["NewHook"]).unwrap();

        // Verify all properties are preserved
        assert!(result.contains("longPollFallbackMs: 2500"));
        assert!(result.contains("_csrf_token: csrfToken"));
        assert!(result.contains("ExistingHook"));
        assert!(result.contains("NewHook"));
        assert!(result.contains("onBeforeElUpdated"));
        assert!(result.contains("metadata"));
    }

    #[test]
    fn test_extend_empty_hooks_object() {
        // Test extending an empty hooks object
        let code = r#"
        let liveSocket = new LiveSocket("/live", Socket, {
          hooks: {},
          params: { _csrf_token: csrfToken }
        });
        "#;

        let result = extend_hook_object_to_ast(code, vec!["FirstHook", "SecondHook"]);
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert!(updated.contains("FirstHook"));
        assert!(updated.contains("SecondHook"));
    }

    #[test]
    fn test_extend_hooks_with_mixed_spread_and_properties() {
        // Test complex hooks object with multiple spreads and properties
        let code = r#"
        const liveSocket = new LiveSocket("/live", Socket, {
          hooks: {
            ...BaseHooks,
            CustomHook,
            ...MoreHooks,
            FinalHook
          }
        });
        "#;

        let result = extend_hook_object_to_ast(code, vec!["NewHook"]);
        assert!(result.is_ok());
        let updated = result.unwrap();

        // Verify original structure is maintained
        assert!(updated.contains("...BaseHooks"));
        assert!(updated.contains("CustomHook"));
        assert!(updated.contains("...MoreHooks"));
        assert!(updated.contains("FinalHook"));
        assert!(updated.contains("NewHook"));
    }

    #[test]
    fn test_remove_objects_of_hooks_from_ast() {
        let code = r#"
        let liveSocket = new LiveSocket("/live", Socket, {
          hooks: { ...Hooks, CopyMixInstallationHook, ObjectOne},
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let result = remove_objects_of_hooks_from_ast(
            code,
            vec!["...Hooks", "ObjectOne", "CopyMixInstallationHook"],
        );

        assert!(result.is_ok());

        let code = r#"
        let liveSocket = new None("/live", Socket, {
          hooks: { ...Hooks, CopyMixInstallationHook, ObjectOne},
          longPollFallbackMs: 2500,
          params: { _csrf_token: csrfToken },
        });
        "#;

        let result = remove_objects_of_hooks_from_ast(
            code,
            vec!["...Hooks", "ObjectOne", "CopyMixInstallationHook"],
        );

        assert!(result.is_err())
    }
}
