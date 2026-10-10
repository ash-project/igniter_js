// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs/contributors>
//
// SPDX-License-Identifier: MIT

//! Edits that change a source file **in place**, leaving every other byte — and every comment —
//! exactly as it was.
//!
//! The codemods in [`super::ast`] parse, change the tree and print it back, and printing reformats
//! the whole file: the developer's line breaks, alignment, blank lines and comments are replaced by
//! the printer's own taste, in a diff that claims to have added one line. That is the right trade
//! for generated code and the wrong one for a `vite.config.ts` somebody wrote by hand.
//!
//! These edits are built on biome's concrete syntax tree. Unlike an AST that drops what is not
//! code, it is **lossless**: every comment and every run of whitespace is trivia attached to a
//! token, and every node knows its exact byte range both with and without that trivia. So the tree
//! says precisely where a node begins and ends, and the change itself is a splice into the
//! original text — nothing the developer wrote is reprinted, and a comment above a member stays
//! above it.
//!
//! Offsets are UTF-8 bytes throughout, which is also what a splice needs; there is no UTF-16
//! conversion to forget.
//!
//! ## What can be edited
//!
//! * **Object and array members** — add a member to an object or array literal reached by a
//!   [`Target`] and a path of keys, creating missing levels (`resolve.alias` in a Vite config,
//!   `translations` in `i18n.ts`, `createRsk({ i18n })`, `workspaces` in `package.json`).
//! * **JSX** — wrap an element in another, unwrap one, add or set props.
//! * **Imports** — make sure names are imported from a module, or move them to another.
//!
//! Every edit is idempotent: running it on its own output changes nothing, which is what lets a
//! caller re-run a setup that failed halfway.
//!
//! ## JSON
//!
//! The dialect `"json"` is a JSON or JSONC document (`package.json`, `tsconfig.json`), parsed by
//! biome's JSON parser with comments and trailing commas allowed — `tsconfig.app.json` ships with
//! `/* Bundler mode */` in it.

use super::dialect::Dialect;
use biome_js_parser::{parse, JsParserOptions};
use biome_js_syntax::{
    AnyJsArrayElement, AnyJsBinding, AnyJsBindingPattern, AnyJsCombinedSpecifier,
    AnyJsDeclarationClause, AnyJsExportClause, AnyJsExportNamedSpecifier, AnyJsExpression,
    AnyJsFunctionBody, AnyJsImportClause, AnyJsNamedImportSpecifier, AnyJsObjectMember,
    AnyJsObjectMemberName, AnyJsStatement, AnyJsxAttribute, AnyJsxAttributeValue, AnyJsxChild,
    JsArrayExpression, JsCallExpression, JsExportDefaultExpressionClause, JsFunctionBody, JsImport,
    JsModule, JsNamedImportSpecifiers, JsObjectExpression, JsSyntaxKind, JsSyntaxNode,
    JsVariableDeclarator, JsxElement, JsxSelfClosingElement,
};
use biome_json_parser::{parse_json, JsonParserOptions};
use biome_json_syntax::{AnyJsonValue, JsonArrayValue, JsonObjectValue};
use biome_parser::diagnostic::ParseDiagnostic;
use biome_rowan::{AstNode, AstNodeList, AstSeparatedList, TextRange};
use serde::Deserialize;

// ── requests ────────────────────────────────────────────────────────────────

/// Which object or array literal an edit is aimed at.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    /// The initialiser of `const NAME = …`, seen through `as const`, `satisfies` and parentheses.
    Variable { name: String },
    /// An argument of a call: `createRsk({…})` is `callee: "createRsk", index: 0`.
    CallArgument {
        callee: String,
        #[serde(default)]
        index: usize,
    },
    /// What the file exports by default: an object, `defineConfig({…})`, `defineConfig(() => ({…}))`
    /// or a variable that holds one.
    DefaultExport,
    /// The whole document, for `package.json` and `tsconfig.json`.
    Document,
}

/// What a path ends in.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Container {
    #[default]
    Object,
    Array,
}

/// What to do with a member that is already there.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Leave it alone. The default: the developer's value wins.
    #[default]
    Add,
    /// Replace it with the given one.
    Set,
}

/// Where new members go in a container that already has some.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    /// Before the existing members. The default: it never has to reason about the last member's
    /// comma, which differs per project and per formatter.
    #[default]
    Head,
    /// After the existing members, the way a person adds one — `package.json`'s `workspaces`.
    End,
}

/// A member to put in an object or array.
#[derive(Debug, Clone, Deserialize)]
pub struct Member {
    /// What "already there" is asked about: an object's key, or an array element's string value.
    pub key: String,
    /// What is written: the whole `key: value` for an object, the element for an array.
    pub code: String,
}

/// A JSX attribute.
#[derive(Debug, Clone, Deserialize)]
pub struct Prop {
    pub name: String,
    /// The text after the `=`, braces and quotes included (`{rsk}`, `"x"`); absent for a bare
    /// boolean attribute.
    #[serde(default)]
    pub value: Option<String>,
}

/// One edit.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    MergeObjectMembers {
        target: Target,
        #[serde(default)]
        path: Vec<String>,
        #[serde(default)]
        container: Container,
        members: Vec<Member>,
        #[serde(default)]
        mode: Mode,
        #[serde(default)]
        at: Placement,
    },
    WrapJsx {
        element: String,
        wrapper: String,
        #[serde(default)]
        props: Vec<Prop>,
    },
    UnwrapJsx {
        element: String,
    },
    AddJsxProps {
        element: String,
        props: Vec<Prop>,
        #[serde(default)]
        mode: Mode,
    },
    EnsureImports {
        module: String,
        names: Vec<String>,
    },
    MoveImports {
        from: String,
        to: String,
        names: Vec<String>,
    },
    RemoveNamedImports {
        module: String,
        names: Vec<String>,
    },
}

/// A question about a source, answered as JSON.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Query {
    /// The member names of the container a target and path lead to.
    ObjectMembers {
        target: Target,
        #[serde(default)]
        path: Vec<String>,
        #[serde(default)]
        container: Container,
    },
    /// The members of the container a target and path lead to, each as `[name, value-as-written]`.
    MemberValues {
        target: Target,
        #[serde(default)]
        path: Vec<String>,
        #[serde(default)]
        container: Container,
    },
    /// Every JSX attribute of a name, in source order, as `{element, name, value, literal}`.
    JsxAttributes { name: String },
    /// Every JSX element name in the file, in source order, with duplicates.
    JsxElements,
    /// Every `import … from "…"`: module, named imports, whether it is type-only.
    Imports,
    /// What the file exports: the names, and the modules it re-exports everything from
    /// (`export * from "m"`), whose names this cannot know.
    Exports,
}

// ── the document ────────────────────────────────────────────────────────────

/// A parsed source: its text, and the tree for whichever language it is.
enum Tree {
    Js(JsSyntaxNode),
    Json(biome_json_syntax::JsonSyntaxNode),
}

struct Doc {
    text: String,
    tree: Tree,
}

impl Doc {
    fn parse(source: &str, dialect: &str) -> Result<Doc, String> {
        let text = source.to_string();

        if dialect.eq_ignore_ascii_case("json") {
            let parsed = parse_json(
                &text,
                JsonParserOptions::default()
                    .with_allow_comments()
                    .with_allow_trailing_commas(),
            );
            if parsed.has_errors() {
                return Err(first_error(&text, parsed.diagnostics().first()));
            }
            return Ok(Doc {
                tree: Tree::Json(parsed.syntax()),
                text,
            });
        }

        let dialect = Dialect::from_name(dialect)?;
        let parsed = parse(&text, dialect.biome_source(), JsParserOptions::default());
        if parsed.has_errors() {
            return Err(first_error(&text, parsed.diagnostics().first()));
        }
        Ok(Doc {
            tree: Tree::Js(parsed.syntax()),
            text,
        })
    }

    fn js(&self) -> Result<&JsSyntaxNode, String> {
        match &self.tree {
            Tree::Js(node) => Ok(node),
            Tree::Json(_) => Err("this edit needs JavaScript, not JSON".to_string()),
        }
    }

    /// The leading whitespace of the line a byte offset falls on.
    fn line_indent(&self, at: usize) -> &str {
        let line_start = self.text[..at].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line = &self.text[line_start..];
        let width = line
            .char_indices()
            .find(|(_, c)| *c != ' ' && *c != '\t')
            .map(|(i, _)| i)
            .unwrap_or(line.len());
        &line[..width]
    }

    /// Is there anything but whitespace between the start of the line and `at`?
    fn follows_text_on_line(&self, at: usize) -> bool {
        let line_start = self.text[..at].rfind('\n').map(|i| i + 1).unwrap_or(0);
        !self.text[line_start..at].trim().is_empty()
    }
}

/// The first thing wrong with a source, in words: `line 3: expected an expression`.
fn first_error(text: &str, diagnostic: Option<&ParseDiagnostic>) -> String {
    use biome_diagnostics::Diagnostic;

    struct Words<'a>(&'a ParseDiagnostic);

    impl std::fmt::Display for Words<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.description(f)
        }
    }

    match diagnostic {
        Some(d) => {
            let line = d.location().span.map(|span| {
                text[..usize::from(span.start()).min(text.len())]
                    .matches('\n')
                    .count()
                    + 1
            });
            match line {
                Some(line) => format!("could not parse the source: line {line}: {}", Words(d)),
                None => format!("could not parse the source: {}", Words(d)),
            }
        }
        None => "could not parse the source".to_string(),
    }
}

fn bytes(range: TextRange) -> (usize, usize) {
    (usize::from(range.start()), usize::from(range.end()))
}

fn trimmed(node: &JsSyntaxNode) -> (usize, usize) {
    bytes(node.text_trimmed_range())
}

/// A name as written, without its quotes.
fn unquote(text: &str) -> String {
    let text = text.trim();
    let bare = text
        .strip_prefix(['"', '\'', '`'])
        .and_then(|t| t.strip_suffix(['"', '\'', '`']))
        .unwrap_or(text);
    bare.to_string()
}

// ── containers: object and array literals, in either language ───────────────

/// An object or array literal, reduced to what an edit needs: where it opens and closes, and what
/// is in it. Built from either language's tree, so the editing below is written once.
struct Cont {
    kind: Container,
    /// Byte offset of `{` or `[`.
    open: usize,
    /// Byte offset just after the opener's trailing trivia on its own line — where a member that
    /// starts on a new line may go without splitting `{ // comment`.
    open_line_end: usize,
    /// Byte offset of `}` or `]`.
    close: usize,
    members: Vec<Mem>,
}

struct Mem {
    name: Option<String>,
    start: usize,
    end: usize,
    /// Where the member's value is: the part after `key:`, or the element itself.
    value: (usize, usize),
    child: Option<Cont>,
}

fn strip(expr: &AnyJsExpression) -> Option<AnyJsExpression> {
    let mut current = expr.clone();
    loop {
        current = match &current {
            AnyJsExpression::JsParenthesizedExpression(e) => e.expression().ok()?,
            AnyJsExpression::TsAsExpression(e) => e.expression().ok()?,
            AnyJsExpression::TsSatisfiesExpression(e) => e.expression().ok()?,
            AnyJsExpression::TsNonNullAssertionExpression(e) => e.expression().ok()?,
            AnyJsExpression::TsTypeAssertionExpression(e) => e.expression().ok()?,
            other => return Some(other.clone()),
        };
    }
}

fn cont_of_expr(expr: &AnyJsExpression) -> Option<Cont> {
    match strip(expr)? {
        AnyJsExpression::JsObjectExpression(o) => cont_of_object(&o),
        AnyJsExpression::JsArrayExpression(a) => cont_of_array(&a),
        _ => None,
    }
}

fn cont_of_object(object: &JsObjectExpression) -> Option<Cont> {
    let l = object.l_curly_token().ok()?;
    let r = object.r_curly_token().ok()?;
    let mut members = Vec::new();

    for member in object.members().iter() {
        let member = member.ok()?;
        let (start, end) = trimmed(member.syntax());
        let (name, child, value) = match &member {
            AnyJsObjectMember::JsPropertyObjectMember(p) => {
                let name = match p.name().ok()? {
                    AnyJsObjectMemberName::JsLiteralMemberName(n) => {
                        Some(unquote(n.value().ok()?.text_trimmed()))
                    }
                    AnyJsObjectMemberName::JsComputedMemberName(_) => None,
                };
                let value = p
                    .value()
                    .ok()
                    .map(|v| bytes(v.syntax().text_trimmed_range()))
                    .unwrap_or((start, end));
                (name, p.value().ok().and_then(|v| cont_of_expr(&v)), value)
            }
            AnyJsObjectMember::JsShorthandPropertyObjectMember(s) => (
                Some(
                    s.name()
                        .ok()?
                        .value_token()
                        .ok()?
                        .text_trimmed()
                        .to_string(),
                ),
                None,
                (start, end),
            ),
            _ => (None, None, (start, end)),
        };
        members.push(Mem {
            name,
            start,
            end,
            value,
            child,
        });
    }

    Some(Cont {
        kind: Container::Object,
        open: usize::from(l.text_trimmed_range().start()),
        open_line_end: usize::from(l.text_range().end()),
        close: usize::from(r.text_trimmed_range().start()),
        members,
    })
}

fn cont_of_array(array: &JsArrayExpression) -> Option<Cont> {
    let l = array.l_brack_token().ok()?;
    let r = array.r_brack_token().ok()?;
    let mut members = Vec::new();

    for element in array.elements().iter() {
        let element = element.ok()?;
        let (start, end) = trimmed(element.syntax());
        let (name, child) = match &element {
            AnyJsArrayElement::AnyJsExpression(e) => (expression_name(e), cont_of_expr(e)),
            _ => (None, None),
        };
        members.push(Mem {
            name,
            start,
            end,
            value: (start, end),
            child,
        });
    }

    Some(Cont {
        kind: Container::Array,
        open: usize::from(l.text_trimmed_range().start()),
        open_line_end: usize::from(l.text_range().end()),
        close: usize::from(r.text_trimmed_range().start()),
        members,
    })
}

/// What an array element is called: its string value, or the identifier it names.
fn expression_name(expr: &AnyJsExpression) -> Option<String> {
    match expr {
        AnyJsExpression::AnyJsLiteralExpression(l) => {
            Some(unquote(l.syntax().text_trimmed().to_string().as_str()))
        }
        AnyJsExpression::JsIdentifierExpression(i) => Some(
            i.name()
                .ok()?
                .value_token()
                .ok()?
                .text_trimmed()
                .to_string(),
        ),
        _ => None,
    }
}

fn cont_of_json(value: &AnyJsonValue) -> Option<Cont> {
    match value {
        AnyJsonValue::JsonObjectValue(o) => cont_of_json_object(o),
        AnyJsonValue::JsonArrayValue(a) => cont_of_json_array(a),
        _ => None,
    }
}

fn cont_of_json_object(object: &JsonObjectValue) -> Option<Cont> {
    let l = object.l_curly_token().ok()?;
    let r = object.r_curly_token().ok()?;
    let mut members = Vec::new();

    for member in object.json_member_list().iter() {
        let member = member.ok()?;
        let range = bytes(member.syntax().text_trimmed_range());
        let name = unquote(member.name().ok()?.value_token().ok()?.text_trimmed());
        let value = member
            .value()
            .ok()
            .map(|v| bytes(v.syntax().text_trimmed_range()))
            .unwrap_or(range);
        members.push(Mem {
            name: Some(name),
            start: range.0,
            end: range.1,
            value,
            child: member.value().ok().and_then(|v| cont_of_json(&v)),
        });
    }

    Some(Cont {
        kind: Container::Object,
        open: usize::from(l.text_trimmed_range().start()),
        open_line_end: usize::from(l.text_range().end()),
        close: usize::from(r.text_trimmed_range().start()),
        members,
    })
}

fn cont_of_json_array(array: &JsonArrayValue) -> Option<Cont> {
    let l = array.l_brack_token().ok()?;
    let r = array.r_brack_token().ok()?;
    let mut members = Vec::new();

    for element in array.elements().iter() {
        let element = element.ok()?;
        let range = bytes(element.syntax().text_trimmed_range());
        let name = match &element {
            AnyJsonValue::JsonStringValue(s) => Some(unquote(s.value_token().ok()?.text_trimmed())),
            _ => None,
        };
        members.push(Mem {
            name,
            start: range.0,
            end: range.1,
            value: range,
            child: cont_of_json(&element),
        });
    }

    Some(Cont {
        kind: Container::Array,
        open: usize::from(l.text_trimmed_range().start()),
        open_line_end: usize::from(l.text_range().end()),
        close: usize::from(r.text_trimmed_range().start()),
        members,
    })
}

// ── finding what an edit is aimed at ────────────────────────────────────────

fn resolve(doc: &Doc, target: &Target) -> Result<Cont, String> {
    if let Tree::Json(root) = &doc.tree {
        return match target {
            Target::Document => biome_json_syntax::JsonRoot::cast(root.clone())
                .and_then(|r| r.value().ok())
                .and_then(|v| cont_of_json(&v))
                .ok_or_else(|| "the document is not an object or an array".to_string()),
            _ => Err("a JSON document can only be edited through the document target".to_string()),
        };
    }

    let root = doc.js()?;
    match target {
        Target::Variable { name } => variable(root, name),
        Target::CallArgument { callee, index } => root
            .descendants()
            .filter_map(JsCallExpression::cast)
            .filter(|call| {
                call.callee()
                    .map(|c| callee_is(&c, callee))
                    .unwrap_or(false)
            })
            .find_map(|call| {
                let args = call.arguments().ok()?.args();
                let arg = args.iter().nth(*index)?.ok()?;
                match arg {
                    biome_js_syntax::AnyJsCallArgument::AnyJsExpression(e) => {
                        object_from(root, &e, 0)
                    }
                    _ => None,
                }
            })
            .ok_or_else(|| format!("no call to {callee}() with an object as argument {index}")),
        Target::DefaultExport => root
            .descendants()
            .filter_map(JsExportDefaultExpressionClause::cast)
            .find_map(|clause| object_from(root, &clause.expression().ok()?, 0))
            .ok_or_else(|| "the file has no default export of an object".to_string()),
        Target::Document => Err("the document target is for JSON".to_string()),
    }
}

fn variable(root: &JsSyntaxNode, name: &str) -> Result<Cont, String> {
    root.descendants()
        .filter_map(JsVariableDeclarator::cast)
        .filter(|d| {
            d.id()
                .map(|id| id.syntax().text_trimmed() == name)
                .unwrap_or(false)
        })
        .find_map(|d| {
            let init = d.initializer()?.expression().ok()?;
            cont_of_expr(&init)
        })
        .ok_or_else(|| format!("no `const {name}` holding an object or array literal"))
}

/// The object an expression stands for, through the wrappers real configs use.
///
/// `defineConfig({…})`, `defineConfig(() => ({…}))`, `defineConfig(({ mode }) => { return {…} })`
/// and `export default config` are all "the config object", and a codemod that only knew the first
/// would refuse most Vite projects in the wild.
fn object_from(root: &JsSyntaxNode, expr: &AnyJsExpression, depth: u8) -> Option<Cont> {
    if depth > 6 {
        return None;
    }
    match strip(expr)? {
        AnyJsExpression::JsObjectExpression(o) => cont_of_object(&o),
        AnyJsExpression::JsCallExpression(call) => {
            let args = call.arguments().ok()?.args();
            args.iter().find_map(|a| match a.ok()? {
                biome_js_syntax::AnyJsCallArgument::AnyJsExpression(e) => {
                    object_from(root, &e, depth + 1)
                }
                _ => None,
            })
        }
        AnyJsExpression::JsArrowFunctionExpression(f) => match f.body().ok()? {
            AnyJsFunctionBody::AnyJsExpression(e) => object_from(root, &e, depth + 1),
            AnyJsFunctionBody::JsFunctionBody(b) => returned_object(root, &b, depth),
        },
        AnyJsExpression::JsFunctionExpression(f) => returned_object(root, &f.body().ok()?, depth),
        AnyJsExpression::JsIdentifierExpression(i) => {
            let name = i
                .name()
                .ok()?
                .value_token()
                .ok()?
                .text_trimmed()
                .to_string();
            variable(root, &name)
                .ok()
                .filter(|c| c.kind == Container::Object)
        }
        _ => None,
    }
}

fn returned_object(root: &JsSyntaxNode, body: &JsFunctionBody, depth: u8) -> Option<Cont> {
    body.statements().iter().find_map(|s| match s {
        AnyJsStatement::JsReturnStatement(r) => object_from(root, &r.argument()?, depth + 1),
        _ => None,
    })
}

fn callee_is(callee: &AnyJsExpression, name: &str) -> bool {
    match callee {
        AnyJsExpression::JsIdentifierExpression(i) => i
            .name()
            .ok()
            .and_then(|n| n.value_token().ok())
            .map(|t| t.text_trimmed() == name)
            .unwrap_or(false),
        AnyJsExpression::JsStaticMemberExpression(m) => m
            .member()
            .ok()
            .map(|n| n.syntax().text_trimmed() == name)
            .unwrap_or(false),
        _ => false,
    }
}

/// Walk `path` down from `cont`. Returns the deepest container reached and the levels of the path
/// that do not exist yet.
fn descend<'a, 'p>(cont: &'a Cont, path: &'p [String]) -> Result<(&'a Cont, &'p [String]), String> {
    let mut current = cont;
    for (i, key) in path.iter().enumerate() {
        match current
            .members
            .iter()
            .find(|m| m.name.as_deref() == Some(key.as_str()))
        {
            None => return Ok((current, &path[i..])),
            Some(member) => match &member.child {
                Some(child) => current = child,
                None => {
                    return Err(format!(
                        "`{key}` is not an object or array literal, so it cannot be edited here"
                    ))
                }
            },
        }
    }
    Ok((current, &path[path.len()..]))
}

// ── editing ─────────────────────────────────────────────────────────────────

/// A change to the text: replace `[start, end)` with `text`.
#[derive(Debug, Clone)]
struct Edit {
    start: usize,
    end: usize,
    text: String,
}

/// Apply edits from the back, so none of them moves the others. Overlapping edits are merged into
/// their union — which is what removing two neighbouring list items needs.
fn apply(text: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by_key(|e| (e.start, e.end));
    let mut merged: Vec<Edit> = Vec::new();
    for edit in edits {
        match merged.last_mut() {
            Some(last) if edit.start < last.end => {
                last.end = last.end.max(edit.end);
                last.text.push_str(&edit.text);
            }
            _ => merged.push(edit),
        }
    }

    let mut out = text.to_string();
    for edit in merged.into_iter().rev() {
        out.replace_range(edit.start..edit.end, &edit.text);
    }
    out
}

/// Apply one request to a source.
pub fn edit(source: &str, dialect: &str, request: &Request) -> Result<String, String> {
    let doc = Doc::parse(source, dialect)?;
    let edits = match request {
        Request::MergeObjectMembers {
            target,
            path,
            container,
            members,
            mode,
            at,
        } => merge_members(&doc, target, path, *container, members, *mode, *at)?,
        Request::WrapJsx {
            element,
            wrapper,
            props,
        } => wrap_jsx(&doc, element, wrapper, props)?,
        Request::UnwrapJsx { element } => unwrap_jsx(&doc, element)?,
        Request::AddJsxProps {
            element,
            props,
            mode,
        } => add_jsx_props(&doc, element, props, *mode)?,
        Request::EnsureImports { module, names } => ensure_imports(&doc, module, names)?,
        Request::MoveImports { from, to, names } => {
            return move_imports(&doc, dialect, from, to, names)
        }
        Request::RemoveNamedImports { module, names } => {
            let all = imports(&doc)?;
            remove_names(&doc, &all, module, names).0
        }
    };

    Ok(apply(&doc.text, in_line_endings_of(&doc.text, edits)))
}

/// Give inserted text the line endings the file already uses.
///
/// Edits are written with `\n`; a file that uses `\r\n` (a project edited on Windows) would
/// otherwise end up with both, which shows as a diff of every line the next time somebody's editor
/// normalises it.
fn in_line_endings_of(text: &str, edits: Vec<Edit>) -> Vec<Edit> {
    if !text.contains("\r\n") {
        return edits;
    }
    edits
        .into_iter()
        .map(|e| Edit {
            text: e.text.replace("\r\n", "\n").replace('\n', "\r\n"),
            ..e
        })
        .collect()
}

/// Answer a question about a source, as JSON text.
pub fn query(source: &str, dialect: &str, query: &Query) -> Result<String, String> {
    let doc = Doc::parse(source, dialect)?;
    let answer = match query {
        Query::ObjectMembers {
            target,
            path,
            container,
        } => {
            let root = resolve(&doc, target)?;
            let (cont, rest) = descend(&root, path)?;
            if !rest.is_empty() || cont.kind != *container {
                return Err(format!(
                    "no {} at that path",
                    match container {
                        Container::Object => "object",
                        Container::Array => "array",
                    }
                ));
            }
            let names: Vec<&str> = cont
                .members
                .iter()
                .filter_map(|m| m.name.as_deref())
                .collect();
            serde_json::json!(names)
        }
        Query::MemberValues {
            target,
            path,
            container,
        } => {
            let root = resolve(&doc, target)?;
            let (cont, rest) = descend(&root, path)?;
            if !rest.is_empty() || cont.kind != *container {
                return Err("nothing at that path".to_string());
            }
            let pairs: Vec<(&str, &str)> = cont
                .members
                .iter()
                .filter_map(|m| {
                    m.name
                        .as_deref()
                        .map(|n| (n, &doc.text[m.value.0..m.value.1]))
                })
                .collect();
            serde_json::json!(pairs)
        }
        Query::JsxAttributes { name } => {
            let found: Vec<serde_json::Value> = jsx_elements(&doc)?
                .into_iter()
                .flat_map(|el| {
                    let element = el.name.clone();
                    el.attr_values
                        .into_iter()
                        .filter(|(n, _, _)| n == name)
                        .map(move |(n, value, literal)| {
                            serde_json::json!({
                                "element": element,
                                "name": n,
                                "value": value,
                                "literal": literal,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
            serde_json::json!(found)
        }
        Query::JsxElements => {
            let names: Vec<String> = jsx_elements(&doc)?.into_iter().map(|e| e.name).collect();
            serde_json::json!(names)
        }
        Query::Exports => {
            let (names, stars) = exports(&doc)?;
            serde_json::json!({ "names": names, "stars": stars })
        }
        Query::Imports => {
            let imports: Vec<serde_json::Value> = imports(&doc)?
                .into_iter()
                .map(|i| {
                    serde_json::json!({
                        "module": i.module,
                        "names": i.named.iter().map(|n| n.name.clone()).collect::<Vec<_>>(),
                        "default": i.default.is_some(),
                        "namespace": i.namespace,
                        "type_only": i.type_only,
                    })
                })
                .collect();
            serde_json::json!(imports)
        }
    };
    serde_json::to_string(&answer).map_err(|e| e.to_string())
}

// ── object and array members ────────────────────────────────────────────────

fn merge_members(
    doc: &Doc,
    target: &Target,
    path: &[String],
    container: Container,
    members: &[Member],
    mode: Mode,
    at: Placement,
) -> Result<Vec<Edit>, String> {
    let root = resolve(doc, target)?;
    let (cont, rest) = descend(&root, path)?;

    if rest.is_empty() {
        if cont.kind != container {
            return Err(format!(
                "the target is not {} literal",
                match container {
                    Container::Object => "an object",
                    Container::Array => "an array",
                }
            ));
        }
        return Ok(fill(doc, cont, members, mode, at));
    }

    // Levels are missing: write the whole nest at the head of the deepest container that exists.
    let json = matches!(doc.tree, Tree::Json(_));
    let mut block = nest(rest, container, members, json);
    // The comma that separates the new level from what follows it — only when something does,
    // or when the language allows a trailing one.
    if at == Placement::End && !cont.members.is_empty() {
        return Ok(insert_at_end(doc, cont, &[block]));
    }
    if !json || !cont.members.is_empty() {
        block.push(',');
    }
    Ok(vec![insert_block(doc, cont, &block)])
}

/// Members into a container that exists.
fn fill(doc: &Doc, cont: &Cont, members: &[Member], mode: Mode, at: Placement) -> Vec<Edit> {
    let mut edits = Vec::new();
    let mut missing: Vec<&Member> = Vec::new();

    for member in members {
        let found = cont
            .members
            .iter()
            .find(|m| m.name.as_deref() == Some(member.key.as_str()));

        match (found, mode) {
            (None, _) => missing.push(member),
            (Some(current), Mode::Set)
                if cont.kind == Container::Object
                    && doc.text[current.start..current.end].trim() != member.code.trim() =>
            {
                edits.push(Edit {
                    start: current.start,
                    end: current.end,
                    text: member.code.clone(),
                });
            }
            _ => {}
        }
    }

    if !missing.is_empty() {
        if at == Placement::End && !cont.members.is_empty() {
            let codes: Vec<String> = missing.iter().map(|m| m.code.clone()).collect();
            edits.extend(insert_at_end(doc, cont, &codes));
        } else {
            edits.push(insert_members(doc, cont, &missing));
        }
    }
    edits
}

/// Add members after the existing ones, in the layout the container already has.
///
/// A trailing comma the developer wrote is kept as the style (and the new last member gets one);
/// without one, the old last member gets the comma that now separates it and the new last member
/// has none. A comment after the old last member stays on its line — the new member goes on the
/// line after it.
fn insert_at_end(doc: &Doc, cont: &Cont, codes: &[String]) -> Vec<Edit> {
    let Some(last) = cont.members.last() else {
        return Vec::new();
    };

    let after = &doc.text[last.end..cont.close];
    let gap = after.len() - after.trim_start().len();
    let trailing_comma = after[gap..].starts_with(',');
    let comma_end = if trailing_comma {
        last.end + gap + 1
    } else {
        last.end
    };

    if !doc.text[cont.open..cont.close].contains('\n') {
        // On one line: `{ a }` becomes `{ a, b }`, and `{ a, }` becomes `{ a, b, }`.
        let joined = codes.join(", ");
        return if trailing_comma {
            vec![Edit {
                start: comma_end,
                end: comma_end,
                text: format!(" {joined},"),
            }]
        } else {
            vec![Edit {
                start: last.end,
                end: last.end,
                text: format!(", {joined}"),
            }]
        };
    }

    let indent = doc.line_indent(last.start).to_string();
    let mut line_end = doc.text[comma_end..cont.close]
        .find('\n')
        .map(|i| comma_end + i)
        .unwrap_or(cont.close);
    // Before the `\r` of a CRLF line break, not between it and the `\n`.
    if doc.text[..line_end].ends_with('\r') {
        line_end -= 1;
    }

    let mut text = String::new();
    for (i, code) in codes.iter().enumerate() {
        text.push('\n');
        text.push_str(&indent);
        text.push_str(&code.replace('\n', &format!("\n{indent}")));
        if i + 1 < codes.len() || trailing_comma {
            text.push(',');
        }
    }

    let mut edits = Vec::new();
    if !trailing_comma {
        edits.push(Edit {
            start: last.end,
            end: last.end,
            text: ",".to_string(),
        });
    }
    edits.push(Edit {
        start: line_end,
        end: line_end,
        text,
    });
    edits
}

/// Insert members at the head of a container, in the layout the container already has.
///
/// At the head rather than the tail: appending means reasoning about whether the last member has a
/// trailing comma, which differs per project and per formatter config. Inserting after the opening
/// delimiter only ever needs a comma of its own, valid in objects and arrays alike.
fn insert_members(doc: &Doc, cont: &Cont, missing: &[&Member]) -> Edit {
    let after_open = cont.open + 1;

    let multiline = match cont.members.first() {
        Some(first) => doc.text[after_open..first.start].contains('\n'),
        None => doc.text[after_open..cont.close].contains('\n'),
    };

    if multiline {
        let indent = match cont.members.first() {
            Some(first) => doc.line_indent(first.start).to_string(),
            None => format!("{}  ", doc.line_indent(cont.open)),
        };
        let mut text = String::new();
        for (i, member) in missing.iter().enumerate() {
            text.push('\n');
            text.push_str(&indent);
            text.push_str(&member.code);
            // Strict JSON has no trailing comma, so the last new member of an empty container
            // goes without one. In JavaScript a trailing comma is the usual style.
            let last_and_alone = i + 1 == missing.len() && cont.members.is_empty();
            if !(last_and_alone && matches!(doc.tree, Tree::Json(_))) {
                text.push(',');
            }
        }
        // After the opener's own trailing comment, so `{ // note` keeps its note.
        return Edit {
            start: cont.open_line_end,
            end: cont.open_line_end,
            text,
        };
    }

    let codes: Vec<&str> = missing.iter().map(|m| m.code.as_str()).collect();
    if cont.members.is_empty() {
        // `{}` becomes `{ a, b }` and `[]` becomes `[a, b]`, each in the style its own kind is
        // normally written. Nothing follows, so no trailing comma is needed.
        let text = match cont.kind {
            Container::Object => format!(" {} ", codes.join(", ")),
            Container::Array => codes.join(", "),
        };
        return Edit {
            start: after_open,
            end: cont.close,
            text,
        };
    }

    // Follow the padding the container already has: `{ a }` takes ` x,` and `["a"]` takes `"x", `.
    let padded = doc.text[after_open..]
        .chars()
        .next()
        .map(char::is_whitespace)
        .unwrap_or(false);
    Edit {
        start: after_open,
        end: after_open,
        text: if padded {
            format!(" {},", codes.join(", "))
        } else {
            format!("{}, ", codes.join(", "))
        },
    }
}

/// A block of several lines at the head of a container, for levels that do not exist yet.
fn insert_block(doc: &Doc, cont: &Cont, block: &str) -> Edit {
    let after_open = cont.open + 1;
    let indent = doc.line_indent(cont.open).to_string();

    // Padding that follows is consumed when the container was written on one line, so the block
    // does not end with the old padding glued to the next member.
    let rest = &doc.text[after_open..];
    let one_line = !rest.starts_with('\n') && !rest.starts_with("\r\n");
    let consumed: usize = if one_line {
        rest.chars()
            .take_while(|c| *c == ' ' || *c == '\t')
            .map(char::len_utf8)
            .sum()
    } else {
        0
    };

    let mut text = String::new();
    for line in block.lines() {
        text.push('\n');
        text.push_str(&indent);
        text.push_str("  ");
        text.push_str(line);
    }
    if one_line {
        text.push('\n');
        text.push_str(&indent);
        // Members that follow stay on their own line; an empty container closes on this one.
        if cont.members.is_empty() {
            return Edit {
                start: after_open,
                end: cont.close,
                text,
            };
        }
        text.push_str("  ");
    }

    Edit {
        start: after_open,
        end: after_open + consumed,
        text,
    }
}

/// `a: {\n  b: {\n    members…\n  }\n}` for the levels `path` names — no comma after the whole
/// of it; the caller knows whether something follows.
///
/// `json` is strict JSON: keys are always quoted and nothing gets a trailing comma. In JavaScript a
/// trailing comma after each member and level is the usual style.
fn nest(path: &[String], container: Container, members: &[Member], json: bool) -> String {
    let (open, close) = match container {
        Container::Object => ("{", "}"),
        Container::Array => ("[", "]"),
    };
    let key = &path[0];
    let label = if !json
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
    {
        key.clone()
    } else {
        format!("\"{key}\"")
    };
    let trailing = if json { "" } else { "," };

    if path.len() == 1 {
        let mut text = format!("{label}: {open}\n");
        for (n, member) in members.iter().enumerate() {
            let lines: Vec<&str> = member.code.lines().collect();
            let last_member = n + 1 == members.len();
            for (i, line) in lines.iter().enumerate() {
                text.push_str("  ");
                text.push_str(line);
                if i + 1 == lines.len() && (!last_member || !json) {
                    text.push(',');
                }
                text.push('\n');
            }
        }
        text.push_str(close);
        return text;
    }

    let inner = nest(&path[1..], container, members, json);
    let mut text = format!("{label}: {{\n");
    let lines: Vec<&str> = inner.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        text.push_str("  ");
        text.push_str(line);
        if i + 1 == lines.len() {
            text.push_str(trailing);
        }
        text.push('\n');
    }
    text.push('}');
    text
}

// ── JSX ─────────────────────────────────────────────────────────────────────

/// A JSX element, reduced to what an edit needs.
struct El {
    name: String,
    start: usize,
    end: usize,
    /// Children that are not whitespace-only text.
    children: Vec<(usize, usize, bool)>,
    /// Where a new attribute goes: right after the name (and its type arguments).
    props_at: usize,
    attributes: Vec<(String, usize, usize)>,
    /// Each attribute's value as written, and whether it is a plain string (`name="x"`) rather
    /// than an expression (`name={x}`) or nothing (`disabled`).
    attr_values: Vec<(String, Option<String>, bool)>,
    /// Is this element a child of another JSX element or fragment?
    in_jsx: bool,
}

fn jsx_elements(doc: &Doc) -> Result<Vec<El>, String> {
    let root = doc.js()?;
    let mut out = Vec::new();

    for node in root.descendants() {
        if let Some(el) = JsxElement::cast(node.clone()) {
            let Ok(opening) = el.opening_element() else {
                continue;
            };
            let Ok(name) = opening.name() else { continue };
            let (start, end) = trimmed(el.syntax());
            let mut props_at = bytes(name.syntax().text_trimmed_range()).1;
            if let Some(args) = opening.type_arguments() {
                props_at = props_at.max(bytes(args.syntax().text_trimmed_range()).1);
            }

            let mut children = Vec::new();
            for child in el.children().iter() {
                let range = bytes(child.syntax().text_trimmed_range());
                let blank = matches!(&child, AnyJsxChild::JsxText(t)
                    if t.value_token().map(|v| v.text().trim().is_empty()).unwrap_or(true));
                if !blank {
                    children.push((
                        range.0,
                        range.1,
                        matches!(
                            child,
                            AnyJsxChild::JsxElement(_)
                                | AnyJsxChild::JsxSelfClosingElement(_)
                                | AnyJsxChild::JsxFragment(_)
                        ),
                    ));
                }
            }

            out.push(El {
                name: name.syntax().text_trimmed().to_string(),
                start,
                end,
                children,
                props_at,
                attributes: attributes_of(opening.attributes().iter()),
                attr_values: attribute_values(opening.attributes().iter()),
                in_jsx: in_jsx(el.syntax()),
            });
        } else if let Some(el) = JsxSelfClosingElement::cast(node) {
            let Ok(name) = el.name() else { continue };
            let (start, end) = trimmed(el.syntax());
            let mut props_at = bytes(name.syntax().text_trimmed_range()).1;
            if let Some(args) = el.type_arguments() {
                props_at = props_at.max(bytes(args.syntax().text_trimmed_range()).1);
            }
            out.push(El {
                name: name.syntax().text_trimmed().to_string(),
                start,
                end,
                children: Vec::new(),
                props_at,
                attributes: attributes_of(el.attributes().iter()),
                attr_values: attribute_values(el.attributes().iter()),
                in_jsx: in_jsx(el.syntax()),
            });
        }
    }

    out.sort_by_key(|e| e.start);
    Ok(out)
}

fn in_jsx(node: &JsSyntaxNode) -> bool {
    node.parent()
        .map(|p| p.kind() == JsSyntaxKind::JSX_CHILD_LIST)
        .unwrap_or(false)
}

fn attributes_of(attributes: impl Iterator<Item = AnyJsxAttribute>) -> Vec<(String, usize, usize)> {
    attributes
        .filter_map(|a| match a {
            AnyJsxAttribute::JsxAttribute(attr) => {
                let (s, e) = trimmed(attr.syntax());
                Some((attr.name().ok()?.syntax().text_trimmed().to_string(), s, e))
            }
            AnyJsxAttribute::JsxSpreadAttribute(_) => None,
        })
        .collect()
}

fn attribute_values(
    attributes: impl Iterator<Item = AnyJsxAttribute>,
) -> Vec<(String, Option<String>, bool)> {
    attributes
        .filter_map(|a| match a {
            AnyJsxAttribute::JsxAttribute(attr) => {
                let name = attr.name().ok()?.syntax().text_trimmed().to_string();
                let value = attr.initializer().and_then(|i| i.value().ok());
                let literal = matches!(&value, Some(AnyJsxAttributeValue::JsxString(_)));
                let text = value.map(|v| v.syntax().text_trimmed().to_string());
                Some((name, text, literal))
            }
            AnyJsxAttribute::JsxSpreadAttribute(_) => None,
        })
        .collect()
}

fn render_props(props: &[Prop]) -> String {
    props
        .iter()
        .map(|p| match &p.value {
            Some(value) => format!(" {}={}", p.name, value),
            None => format!(" {}", p.name),
        })
        .collect()
}

/// Shift every line after the first by `by` spaces (or back by that many when negative).
fn reindent(text: &str, by: isize) -> String {
    let mut lines = text.split('\n');
    let mut out = String::from(lines.next().unwrap_or(""));
    for line in lines {
        out.push('\n');
        if line.trim().is_empty() {
            continue;
        }
        if by >= 0 {
            out.push_str(&" ".repeat(by as usize));
            out.push_str(line);
        } else {
            let remove = (-by) as usize;
            let lead = line.len() - line.trim_start_matches(' ').len();
            out.push_str(&line[lead.min(remove)..]);
        }
    }
    out
}

fn wrap_jsx(doc: &Doc, element: &str, wrapper: &str, props: &[Prop]) -> Result<Vec<Edit>, String> {
    let all = jsx_elements(doc)?;
    let targets: Vec<&El> = all.iter().filter(|e| e.name == element).collect();
    let first = targets
        .first()
        .ok_or_else(|| format!("no <{element}> element to wrap"))?;

    // Already wrapped, anywhere: nothing to do. The check is on the wrapper's own children, so a
    // second run over its own output finds the work done instead of nesting another one.
    let already = targets.iter().any(|t| {
        all.iter()
            .filter(|w| w.name == wrapper)
            .any(|w| w.children.iter().any(|c| (c.0, c.1) == (t.start, t.end)))
    });
    if already {
        return Ok(Vec::new());
    }

    let inner = &doc.text[first.start..first.end];
    let open = format!("<{wrapper}{}>", render_props(props));

    let text = if doc.follows_text_on_line(first.start) && !inner.contains('\n') {
        format!("{open}{inner}</{wrapper}>")
    } else {
        let indent = doc.line_indent(first.start);
        format!(
            "{open}\n{indent}  {}\n{indent}</{wrapper}>",
            reindent(inner, 2)
        )
    };

    Ok(vec![Edit {
        start: first.start,
        end: first.end,
        text,
    }])
}

fn unwrap_jsx(doc: &Doc, element: &str) -> Result<Vec<Edit>, String> {
    let all = jsx_elements(doc)?;
    let Some(target) = all.iter().find(|e| e.name == element) else {
        return Ok(Vec::new());
    };

    let (first, last) = match (target.children.first(), target.children.last()) {
        (Some(f), Some(l)) => (f.0, l.1),
        _ => return Err(format!("<{element}> has no children to keep")),
    };

    let kept = reindent(&doc.text[first..last], -2);
    let single_element = target.children.len() == 1 && target.children[0].2;

    // Inside another element its children can simply take its place; anywhere else several
    // children need a fragment to be one expression.
    let text = if single_element || target.in_jsx {
        kept
    } else {
        format!("<>{kept}</>")
    };

    Ok(vec![Edit {
        start: target.start,
        end: target.end,
        text,
    }])
}

fn add_jsx_props(
    doc: &Doc,
    element: &str,
    props: &[Prop],
    mode: Mode,
) -> Result<Vec<Edit>, String> {
    let all = jsx_elements(doc)?;
    let target = all
        .iter()
        .find(|e| e.name == element)
        .ok_or_else(|| format!("no <{element}> element"))?;

    let mut edits = Vec::new();
    let mut missing = Vec::new();

    for prop in props {
        match (target.attributes.iter().find(|a| a.0 == prop.name), mode) {
            (None, _) => missing.push(prop.clone()),
            (Some((_, s, e)), Mode::Set) => {
                let wanted = render_props(std::slice::from_ref(prop));
                if doc.text[*s..*e].trim() != wanted.trim() {
                    edits.push(Edit {
                        start: *s,
                        end: *e,
                        text: wanted.trim().to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    if !missing.is_empty() {
        edits.push(Edit {
            start: target.props_at,
            end: target.props_at,
            text: render_props(&missing),
        });
    }
    Ok(edits)
}

// ── inserting whole import statements ───────────────────────────────────────

/// Why an insert could not be made.
#[derive(Debug, PartialEq, Eq)]
pub enum InsertError {
    /// The statements to insert do not parse.
    Argument,
    /// The file does not parse.
    Source(String),
}

/// What a caller asked to be inserted.
enum Wanted {
    Import(Box<ImportInfo>),
    /// Anything that is not an import, as written. Inserted as it is, never deduplicated.
    Other(String),
}

/// Insert whole import statements into a file, leaving the rest of it exactly as written.
///
/// The statements are what a caller would write — `import { z } from "zod"`, several of them if
/// they are separated by line breaks — and each one is dealt with on its own:
///
/// * **Already imported** — every name it brings is already imported from that module: nothing is
///   written. Importing something twice is not harmless, it is bad code, so this is not an option.
/// * **The module is imported, with other names** — the new names are added at the end of that
///   import's braces (or after its default name), so the file never ends up with two imports of one
///   module.
/// * **Otherwise** — a new line after the last import (after any `"use strict"` directive when
///   there is none), in the quote and semicolon style the file's imports already use.
///
/// A type-only import is only compared with type-only imports, and a bare `import "m"` counts as
/// already imported when the module is imported in any form.
pub fn insert_imports(source: &str, dialect: &str, lines: &str) -> Result<String, InsertError> {
    let wanted = wanted_statements(lines, dialect)?;
    let mut text = source.to_string();
    for item in wanted {
        text = insert_one(&text, dialect, &item)?;
    }
    Ok(text)
}

fn wanted_statements(lines: &str, dialect: &str) -> Result<Vec<Wanted>, InsertError> {
    let doc = Doc::parse(lines, dialect).map_err(|_| InsertError::Argument)?;
    let root = doc.js().map_err(|_| InsertError::Argument)?;
    let module = JsModule::cast(root.clone()).ok_or(InsertError::Argument)?;
    let mut infos = imports(&doc)
        .map_err(|_| InsertError::Argument)?
        .into_iter();

    Ok(module
        .items()
        .iter()
        .filter_map(|item| match item {
            biome_js_syntax::AnyJsModuleItem::JsImport(_) => {
                infos.next().map(|i| Wanted::Import(Box::new(i)))
            }
            other => Some(Wanted::Other(other.syntax().text_trimmed().to_string())),
        })
        .collect())
}

fn bindings(import: &ImportInfo) -> Vec<&str> {
    import
        .default_local
        .iter()
        .chain(import.namespace_local.iter())
        .map(String::as_str)
        .chain(import.named.iter().map(|n| n.local.as_str()))
        .collect()
}

fn insert_one(text: &str, dialect: &str, wanted: &Wanted) -> Result<String, InsertError> {
    let doc = Doc::parse(text, dialect).map_err(InsertError::Source)?;
    let all = imports(&doc).map_err(InsertError::Source)?;

    let statement = match wanted {
        Wanted::Other(code) => code.clone(),
        Wanted::Import(import) => {
            let same: Vec<&ImportInfo> = all
                .iter()
                .filter(|i| i.module == import.module && i.type_only == import.type_only)
                .collect();
            let have: Vec<&str> = same.iter().flat_map(|i| bindings(i)).collect();
            let asked = bindings(import);

            // Nothing to bind: only the module's presence matters.
            if asked.is_empty() && !same.is_empty() {
                return Ok(text.to_string());
            }
            if !asked.is_empty() && asked.iter().all(|b| have.contains(b)) {
                return Ok(text.to_string());
            }

            // The module is imported already: bring the new names into its braces.
            let plain = import.default_local.is_none() && import.namespace_local.is_none();
            if plain {
                if let Some(target) = same.iter().find(|i| i.braces.is_some()) {
                    let add: Vec<&str> = import
                        .named
                        .iter()
                        .filter(|n| !have.contains(&n.local.as_str()))
                        .map(|n| n.text.as_str())
                        .collect();
                    return Ok(apply(text, vec![append_names(&doc, target, &add)]));
                }
            }

            // Only a default import so far: `import A from "m"` becomes `import A, { x } from "m"`.
            if plain {
                if let Some(default) = same
                    .iter()
                    .find(|i| i.braces.is_none() && !i.namespace)
                    .and_then(|i| i.default)
                {
                    let add: Vec<&str> = import
                        .named
                        .iter()
                        .filter(|n| !have.contains(&n.local.as_str()))
                        .map(|n| n.text.as_str())
                        .collect();
                    return Ok(apply(
                        text,
                        vec![Edit {
                            start: default.1,
                            end: default.1,
                            text: format!(", {{ {} }}", add.join(", ")),
                        }],
                    ));
                }
            }

            in_file_style(import, all.last())
        }
    };

    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let edit = match all.last() {
        Some(last) => Edit {
            start: last.line_end,
            end: last.line_end,
            text: format!("{newline}{statement}"),
        },
        None => match prologue_end(&doc) {
            Some(at) => Edit {
                start: at,
                end: at,
                text: format!("{newline}{statement}"),
            },
            None => Edit {
                start: 0,
                end: 0,
                text: format!("{statement}{newline}"),
            },
        },
    };
    Ok(apply(text, vec![edit]))
}

/// The statement, with its module's quotes and its semicolon the way the file's imports have them.
///
/// A file with no imports has no style to follow, so the caller's own is kept.
fn in_file_style(import: &ImportInfo, last: Option<&ImportInfo>) -> String {
    let (quote, semicolon) = match last {
        Some(last) => (last.quote, last.semicolon),
        None => (import.quote, import.semicolon),
    };

    let mut statement = import.text.clone();
    let (from, to) = (
        import.source_range.0 - import.start,
        import.source_range.1 - import.start,
    );
    let token = &import.text[from..to];
    let inner = token.trim_matches(|c| c == '"' || c == '\'');
    if !inner.contains(quote) && inner.len() + 2 == token.len() {
        statement.replace_range(from..to, &format!("{quote}{inner}{quote}"));
    }

    let bare = statement.trim_end().trim_end_matches(';').to_string();
    if semicolon {
        format!("{bare};")
    } else {
        bare
    }
}

/// Where a new first import goes: after a `#!` line and after `"use strict"`-style directives,
/// which only mean anything while they come first.
fn prologue_end(doc: &Doc) -> Option<usize> {
    let module = JsModule::cast(doc.js().ok()?.clone())?;
    let interpreter = module
        .interpreter_token()
        .map(|t| usize::from(t.text_trimmed_range().end()));
    let directive = module
        .directives()
        .iter()
        .last()
        .and_then(|d| d.syntax().last_token())
        .map(|t| usize::from(t.text_range().end()));
    interpreter.max(directive)
}

// ── exports ─────────────────────────────────────────────────────────────────

/// The names a module exports, and the modules it re-exports in full.
///
/// `export { a, b as c }`, `export { x } from "m"`, `export const …`, `export function …`,
/// `export class …`, `export type …` / `interface` / `enum`. A default export has no name here,
/// and `export * from "m"` is reported as a module, because what is in it is that file's business.
fn exports(doc: &Doc) -> Result<(Vec<String>, Vec<String>), String> {
    let module = JsModule::cast(doc.js()?.clone()).ok_or("not a module")?;
    let mut names = Vec::new();
    let mut stars = Vec::new();

    for item in module.items().iter() {
        let biome_js_syntax::AnyJsModuleItem::JsExport(export) = item else {
            continue;
        };
        let Ok(clause) = export.export_clause() else {
            continue;
        };

        match clause {
            AnyJsExportClause::JsExportNamedClause(c) => {
                for specifier in c.specifiers().iter().flatten() {
                    match specifier {
                        AnyJsExportNamedSpecifier::JsExportNamedShorthandSpecifier(s) => {
                            if let Some(n) = s.name().ok().and_then(|n| n.value_token().ok()) {
                                names.push(n.text_trimmed().to_string());
                            }
                        }
                        AnyJsExportNamedSpecifier::JsExportNamedSpecifier(s) => {
                            if let Some(n) = s.exported_name().ok().and_then(|n| n.value().ok()) {
                                names.push(unquote(n.text_trimmed()));
                            }
                        }
                    }
                }
            }
            AnyJsExportClause::JsExportNamedFromClause(c) => {
                for specifier in c.specifiers().iter().flatten() {
                    let renamed = specifier
                        .export_as()
                        .and_then(|a| a.exported_name().ok())
                        .and_then(|n| n.value().ok());
                    let original = specifier.source_name().ok().and_then(|n| n.value().ok());
                    if let Some(token) = renamed.or(original) {
                        names.push(unquote(token.text_trimmed()));
                    }
                }
            }
            AnyJsExportClause::JsExportFromClause(c) => match c.export_as() {
                Some(alias) => {
                    if let Some(n) = alias.exported_name().ok().and_then(|n| n.value().ok()) {
                        names.push(unquote(n.text_trimmed()));
                    }
                }
                None => {
                    if let Some(source) = c.source().ok().and_then(|s| s.value_token().ok()) {
                        stars.push(unquote(source.text_trimmed()));
                    }
                }
            },
            AnyJsExportClause::AnyJsDeclarationClause(d) => declared_names(&d, &mut names),
            _ => {}
        }
    }

    Ok((names, stars))
}

fn declared_names(clause: &AnyJsDeclarationClause, names: &mut Vec<String>) {
    let binding = |b: Option<AnyJsBinding>| b.map(|b| b.syntax().text_trimmed().to_string());

    match clause {
        AnyJsDeclarationClause::JsVariableDeclarationClause(c) => {
            if let Ok(declaration) = c.declaration() {
                for declarator in declaration.declarators().iter().flatten() {
                    if let Ok(AnyJsBindingPattern::AnyJsBinding(b)) = declarator.id() {
                        names.push(b.syntax().text_trimmed().to_string());
                    }
                }
            }
        }
        AnyJsDeclarationClause::JsFunctionDeclaration(d) => names.extend(binding(d.id().ok())),
        AnyJsDeclarationClause::JsClassDeclaration(d) => names.extend(binding(d.id().ok())),
        AnyJsDeclarationClause::TsEnumDeclaration(d) => names.extend(binding(d.id().ok())),
        AnyJsDeclarationClause::TsDeclareFunctionDeclaration(d) => {
            names.extend(binding(d.id().ok()))
        }
        AnyJsDeclarationClause::TsTypeAliasDeclaration(d) => names.extend(
            d.binding_identifier()
                .ok()
                .map(|b| b.syntax().text_trimmed().to_string()),
        ),
        AnyJsDeclarationClause::TsInterfaceDeclaration(d) => {
            names.extend(d.id().ok().map(|b| b.syntax().text_trimmed().to_string()))
        }
        _ => {}
    }
}

// ── removing imports ────────────────────────────────────────────────────────

/// Why a removal could not be made.
#[derive(Debug, PartialEq, Eq)]
pub enum RemoveError {
    /// The file does not parse.
    Source(String),
}

/// What a removal request refers to: module names.
///
/// The argument is first read as JavaScript. If it holds import statements, the modules those
/// import from are the targets — which is what keeps a multi-line statement from leaking its lines
/// as targets. Only when it holds none is each non-empty line a module name as it stands, which is
/// what lets a bare `topbar` or `../vendor/topbar` work though neither is a valid statement.
fn removal_targets(modules: &str, dialect: &str) -> Vec<String> {
    if let Ok(doc) = Doc::parse(modules, dialect) {
        if let Ok(found) = imports(&doc) {
            if !found.is_empty() {
                return found.into_iter().map(|i| i.module).collect();
            }
        }
    }

    modules
        .lines()
        .map(|line| line.trim().trim_end_matches(';').trim())
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// Remove every import of the named modules, leaving the rest of the file exactly as written.
///
/// An import goes whole: its line, a comment after it on that line, and the line break. A comment
/// on a line of its own above it is left where it is — it may be a licence header or a note about
/// the file, and keeping a stray comment is a smaller mistake than deleting one that mattered. If
/// the removal would leave two blank lines in a row, one is taken too.
///
/// Matching is on the module a statement imports from, not on what it binds, so removing
/// `"../vendor/topbar"` takes `import topbar from "../vendor/topbar"` and removing `"topbar"`
/// does not.
pub fn remove_imports(source: &str, dialect: &str, modules: &str) -> Result<String, RemoveError> {
    let targets = removal_targets(modules, dialect);
    let doc = Doc::parse(source, dialect).map_err(RemoveError::Source)?;
    if targets.is_empty() {
        return Ok(source.to_string());
    }

    let edits: Vec<Edit> = imports(&doc)
        .map_err(RemoveError::Source)?
        .iter()
        .filter(|i| targets.contains(&i.module))
        .map(|i| whole_line(&doc, i))
        .collect();

    Ok(apply(source, in_line_endings_of(source, edits)))
}

/// The range of the line an import is on, including its trailing comment and line break.
fn whole_line(doc: &Doc, import: &ImportInfo) -> Edit {
    let text = &doc.text;

    // Back over the indentation, if the import starts its line.
    let mut start = import.start;
    let line_start = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    if text[line_start..start].trim().is_empty() {
        start = line_start;
    }

    // Forward over a comment on the same line, then the line break.
    let mut end = import.end;
    let rest_of_line = text[end..].find('\n').unwrap_or(text.len() - end);
    let tail = &text[end..end + rest_of_line];
    let only_comment = {
        let t = tail.trim();
        t.is_empty() || t.starts_with("//") || (t.starts_with("/*") && t.ends_with("*/"))
    };
    if only_comment {
        end += rest_of_line;
        if text[end..].starts_with("\r\n") {
            end += 2;
        } else if text[end..].starts_with('\n') {
            end += 1;
        }
    }

    // Two blank lines in a row are one too many: take the one after.
    let after_is_blank = text[end..]
        .lines()
        .next()
        .map(|l| l.trim().is_empty())
        .unwrap_or(false)
        && end < text.len();
    let before_is_blank = previous_line(&text[..start])
        .map(|l| l.trim().is_empty())
        .unwrap_or(false);
    if after_is_blank && before_is_blank {
        let skip = text[end..]
            .find('\n')
            .map(|i| i + 1)
            .unwrap_or(text.len() - end);
        end += skip;
    }

    Edit {
        start,
        end,
        text: String::new(),
    }
}

/// The line that ends just before this text ends, if there is one.
fn previous_line(text: &str) -> Option<&str> {
    let body = text.strip_suffix('\n')?;
    let body = body.strip_suffix('\r').unwrap_or(body);
    Some(body.rsplit('\n').next().unwrap_or(body))
}

// ── imports ─────────────────────────────────────────────────────────────────

struct Spec {
    /// The imported name, before any `as`.
    name: String,
    /// The name it is bound to in this file.
    local: String,
    /// The specifier as written: `Hooks as BackpexHooks`.
    text: String,
    start: usize,
    end: usize,
}

struct ImportInfo {
    module: String,
    /// The whole statement as written, comments inside it included.
    text: String,
    /// Where the module name's string token sits in the file.
    source_range: (usize, usize),
    default_local: Option<String>,
    namespace_local: Option<String>,
    start: usize,
    end: usize,
    /// Where the statement's line ends: after a comment that follows it on the same line.
    line_end: usize,
    quote: char,
    semicolon: bool,
    type_only: bool,
    default: Option<(usize, usize)>,
    namespace: bool,
    named: Vec<Spec>,
    /// Where the braces open and close, when there are braces.
    braces: Option<(usize, usize)>,
}

fn imports(doc: &Doc) -> Result<Vec<ImportInfo>, String> {
    let root = doc.js()?;
    let mut out = Vec::new();

    for import in root.descendants().filter_map(JsImport::cast) {
        let Ok(clause) = import.import_clause() else {
            continue;
        };
        let (start, end) = trimmed(import.syntax());
        let semicolon = import.semicolon_token().is_some();

        let mut info = ImportInfo {
            module: String::new(),
            text: doc.text[start..end].to_string(),
            source_range: (0, 0),
            default_local: None,
            namespace_local: None,
            start,
            end,
            line_end: import
                .syntax()
                .last_token()
                .map(|t| usize::from(t.text_range().end()))
                .unwrap_or(end),
            quote: '"',
            semicolon,
            type_only: false,
            default: None,
            namespace: false,
            named: Vec::new(),
            braces: None,
        };

        let source = match &clause {
            AnyJsImportClause::JsImportBareClause(c) => c.source().ok(),
            AnyJsImportClause::JsImportDefaultClause(c) => {
                info.type_only = c.type_token().is_some();
                info.default = c
                    .default_specifier()
                    .ok()
                    .map(|d| bytes(d.syntax().text_trimmed_range()));
                info.default_local = c.default_specifier().ok().and_then(|d| {
                    d.local_name()
                        .ok()
                        .map(|n| n.syntax().text_trimmed().to_string())
                });
                c.source().ok()
            }
            AnyJsImportClause::JsImportNamespaceClause(c) => {
                info.type_only = c.type_token().is_some();
                info.namespace = true;
                info.namespace_local = c.namespace_specifier().ok().and_then(|n| {
                    n.local_name()
                        .ok()
                        .map(|n| n.syntax().text_trimmed().to_string())
                });
                c.source().ok()
            }
            AnyJsImportClause::JsImportNamedClause(c) => {
                info.type_only = c.type_token().is_some();
                if let Ok(specifiers) = c.named_specifiers() {
                    fill_named(&mut info, &specifiers, &doc.text);
                }
                c.source().ok()
            }
            AnyJsImportClause::JsImportCombinedClause(c) => {
                info.default = c
                    .default_specifier()
                    .ok()
                    .map(|d| bytes(d.syntax().text_trimmed_range()));
                info.default_local = c.default_specifier().ok().and_then(|d| {
                    d.local_name()
                        .ok()
                        .map(|n| n.syntax().text_trimmed().to_string())
                });
                match c.specifier() {
                    Ok(AnyJsCombinedSpecifier::JsNamedImportSpecifiers(s)) => {
                        fill_named(&mut info, &s, &doc.text)
                    }
                    Ok(AnyJsCombinedSpecifier::JsNamespaceImportSpecifier(n)) => {
                        info.namespace = true;
                        info.namespace_local = n
                            .local_name()
                            .ok()
                            .map(|n| n.syntax().text_trimmed().to_string());
                    }
                    Err(_) => {}
                }
                c.source().ok()
            }
        };

        if let Some(source) = source {
            if let Ok(token) = source.value_token() {
                info.source_range = bytes(token.text_trimmed_range());
                let text = token.text_trimmed();
                info.quote = text.chars().next().filter(|c| *c == '\'').unwrap_or('"');
                info.module = unquote(text);
            }
        }
        out.push(info);
    }
    Ok(out)
}

fn fill_named(info: &mut ImportInfo, specifiers: &JsNamedImportSpecifiers, text: &str) {
    if let (Ok(l), Ok(r)) = (specifiers.l_curly_token(), specifiers.r_curly_token()) {
        info.braces = Some((
            usize::from(l.text_trimmed_range().end()),
            usize::from(r.text_trimmed_range().start()),
        ));
    }
    for specifier in specifiers.specifiers().iter().flatten() {
        let (start, end) = bytes(specifier.syntax().text_trimmed_range());
        let name = match &specifier {
            AnyJsNamedImportSpecifier::JsNamedImportSpecifier(s) => s
                .name()
                .ok()
                .and_then(|n| n.value().ok())
                .map(|t| unquote(t.text_trimmed())),
            AnyJsNamedImportSpecifier::JsShorthandNamedImportSpecifier(s) => s
                .local_name()
                .ok()
                .map(|n| n.syntax().text_trimmed().to_string()),
            AnyJsNamedImportSpecifier::JsBogusNamedImportSpecifier(_) => None,
        };
        let local = match &specifier {
            AnyJsNamedImportSpecifier::JsNamedImportSpecifier(s) => s
                .local_name()
                .ok()
                .map(|n| n.syntax().text_trimmed().to_string()),
            _ => name.clone(),
        };
        if let (Some(name), Some(local)) = (name, local) {
            info.named.push(Spec {
                name,
                local,
                text: text[start..end].to_string(),
                start,
                end,
            });
        }
    }
}

/// Add names to the end of an import's braces, in the layout the braces already have.
///
/// At the end, because that is where a person adds one: `{ a }` plus `b` reads `{ a, b }`, two
/// names asked for in an order come out in that order, and in a diff the line only gains
/// `, b`. A list written one name per line gets the new name on its own line with the same
/// indent, and a trailing comma the developer wrote is left where it was.
fn append_names(doc: &Doc, target: &ImportInfo, texts: &[&str]) -> Edit {
    match target.named.last() {
        Some(last) => {
            let first = &target.named[0];
            let multiline = doc.text[first.start..last.end].contains('\n');
            let text = if multiline {
                let indent = doc.line_indent(last.start);
                texts
                    .iter()
                    .map(|t| format!(",\n{indent}{t}"))
                    .collect::<String>()
            } else {
                texts.iter().map(|t| format!(", {t}")).collect::<String>()
            };
            Edit {
                start: last.end,
                end: last.end,
                text,
            }
        }
        None => {
            let (open, close) = target.braces.unwrap_or((0, 0));
            Edit {
                start: open,
                end: close,
                text: format!(" {} ", texts.join(", ")),
            }
        }
    }
}

fn ensure_imports(doc: &Doc, module: &str, names: &[String]) -> Result<Vec<Edit>, String> {
    let all = imports(doc)?;
    let from_module: Vec<&ImportInfo> = all
        .iter()
        .filter(|i| i.module == module && !i.type_only)
        .collect();

    let have: Vec<&str> = from_module
        .iter()
        .flat_map(|i| i.named.iter().map(|n| n.name.as_str()))
        .collect();
    let missing: Vec<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|n| !have.contains(n))
        .collect();
    if missing.is_empty() {
        return Ok(Vec::new());
    }
    let joined = missing.join(", ");

    // Into braces: after the last name, or into empty braces.
    if let Some(target) = from_module.iter().find(|i| i.braces.is_some()) {
        return Ok(vec![append_names(doc, target, &missing)]);
    }

    // An import with only a default: `import React from "react"` → `import React, { x } from …`.
    if let Some((_, end)) = from_module
        .iter()
        .find(|i| !i.namespace)
        .and_then(|i| i.default)
    {
        return Ok(vec![Edit {
            start: end,
            end,
            text: format!(", {{ {joined} }}"),
        }]);
    }

    // A new line after the last import, in the quote and semicolon style the file already uses.
    let (quote, semi) = all
        .last()
        .map(|i| (i.quote, i.semicolon))
        .unwrap_or(('"', false));
    let line = format!(
        "import {{ {joined} }} from {quote}{module}{quote}{}",
        if semi { ";" } else { "" }
    );

    Ok(vec![match all.last() {
        Some(last) => Edit {
            start: last.line_end,
            end: last.line_end,
            text: format!("\n{line}"),
        },
        None => Edit {
            start: 0,
            end: 0,
            text: format!("{line}\n"),
        },
    }])
}

/// The edits that take `names` out of every import of `module`, and the names that were there.
///
/// An import left with no names goes whole, by `whole_line` — its line, a comment after it and the
/// line break. One that keeps others loses only the removed specifiers, each with its separator,
/// so what sits between the rest — their comments included — stays as it is.
fn remove_names(
    doc: &Doc,
    all: &[ImportInfo],
    module: &str,
    names: &[String],
) -> (Vec<Edit>, Vec<String>) {
    let mut removed: Vec<String> = Vec::new();
    let mut edits = Vec::new();

    for import in all.iter().filter(|i| i.module == module && !i.type_only) {
        let gone: Vec<&Spec> = import
            .named
            .iter()
            .filter(|s| names.contains(&s.name))
            .collect();
        if gone.is_empty() {
            continue;
        }
        removed.extend(gone.iter().map(|s| s.name.clone()));

        let everything_goes =
            gone.len() == import.named.len() && import.default.is_none() && !import.namespace;

        if everything_goes {
            edits.push(whole_line(doc, import));
            continue;
        }

        // Runs of removed specifiers, each taken out with its separators.
        let specs = &import.named;
        let mut i = 0;
        while i < specs.len() {
            if !names.contains(&specs[i].name) {
                i += 1;
                continue;
            }
            let mut j = i;
            while j + 1 < specs.len() && names.contains(&specs[j + 1].name) {
                j += 1;
            }
            let (start, end) = match specs.get(j + 1) {
                // Something follows: take the run and the comma after it.
                Some(next) => (specs[i].start, next.start),
                // The run reaches the end: take the comma before it too.
                None => (
                    i.checked_sub(1)
                        .map(|p| specs[p].end)
                        .unwrap_or(specs[i].start),
                    specs[j].end,
                ),
            };
            edits.push(Edit {
                start,
                end,
                text: String::new(),
            });
            i = j + 1;
        }
    }

    (edits, removed)
}

fn move_imports(
    doc: &Doc,
    dialect: &str,
    from: &str,
    to: &str,
    names: &[String],
) -> Result<String, String> {
    let all = imports(doc)?;
    let (edits, mut moved) = remove_names(doc, &all, from, names);

    if moved.is_empty() {
        return Ok(doc.text.clone());
    }

    moved.dedup();
    let removed = apply(&doc.text, in_line_endings_of(&doc.text, edits));
    edit(
        &removed,
        dialect,
        &Request::EnsureImports {
            module: to.to_string(),
            names: moved,
        },
    )
}
