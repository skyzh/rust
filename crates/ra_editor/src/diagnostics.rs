use itertools::Itertools;

use ra_syntax::{
    ast::{self, AstNode},
    Location,
    SourceFileNode,
    SyntaxKind,
    TextRange,
};
use ra_text_edit::{
    TextEdit,
    TextEditBuilder,
};

use crate::{
    Diagnostic,
    LocalEdit,
    Severity,
};

pub fn diagnostics(file: &SourceFileNode) -> Vec<Diagnostic> {
    fn location_to_range(location: Location) -> TextRange {
        match location {
            Location::Offset(offset) => TextRange::offset_len(offset, 1.into()),
            Location::Range(range) => range,
        }
    }

    let mut errors: Vec<Diagnostic> = file
        .errors()
        .into_iter()
        .map(|err| Diagnostic {
            range: location_to_range(err.location()),
            msg: format!("Syntax Error: {}", err),
            severity: Severity::Error,
            fix: None,
        })
        .collect();

    errors.extend(check_unnecessary_braces_in_use_statement(file));
    errors.extend(check_struct_shorthand_initialization(file));
    errors
}

fn check_unnecessary_braces_in_use_statement(file: &SourceFileNode) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for use_tree_list in file
        .syntax()
        .descendants()
        .filter_map(ast::UseTreeList::cast)
    {
        if let Some((single_use_tree,)) = use_tree_list.use_trees().collect_tuple() {
            let range = use_tree_list.syntax().range();
            let edit =
                text_edit_for_remove_unnecessary_braces_with_self_in_use_statement(single_use_tree)
                    .unwrap_or_else(|| {
                        let to_replace = single_use_tree.syntax().text().to_string();
                        let mut edit_builder = TextEditBuilder::new();
                        edit_builder.delete(range);
                        edit_builder.insert(range.start(), to_replace);
                        edit_builder.finish()
                    });

            diagnostics.push(Diagnostic {
                range,
                msg: format!("Unnecessary braces in use statement"),
                severity: Severity::WeakWarning,
                fix: Some(LocalEdit {
                    label: "Remove unnecessary braces".to_string(),
                    edit,
                    cursor_position: None,
                }),
            })
        }
    }

    diagnostics
}

fn text_edit_for_remove_unnecessary_braces_with_self_in_use_statement(
    single_use_tree: ast::UseTree,
) -> Option<TextEdit> {
    let use_tree_list_node = single_use_tree.syntax().parent()?;
    if single_use_tree
        .path()?
        .segment()?
        .syntax()
        .first_child()?
        .kind()
        == SyntaxKind::SELF_KW
    {
        let start = use_tree_list_node.prev_sibling()?.range().start();
        let end = use_tree_list_node.range().end();
        let range = TextRange::from_to(start, end);
        let mut edit_builder = TextEditBuilder::new();
        edit_builder.delete(range);
        return Some(edit_builder.finish());
    }
    None
}

fn check_struct_shorthand_initialization(file: &SourceFileNode) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for struct_lit in file.syntax().descendants().filter_map(ast::StructLit::cast) {
        if let Some(named_field_list) = struct_lit.named_field_list() {
            for named_field in named_field_list.fields() {
                if let (Some(name_ref), Some(expr)) = (named_field.name_ref(), named_field.expr()) {
                    let field_name = name_ref.syntax().text().to_string();
                    let field_expr = expr.syntax().text().to_string();
                    if field_name == field_expr {
                        let mut edit_builder = TextEditBuilder::new();
                        edit_builder.delete(named_field.syntax().range());
                        edit_builder.insert(named_field.syntax().range().start(), field_name);
                        let edit = edit_builder.finish();

                        diagnostics.push(Diagnostic {
                            range: named_field.syntax().range(),
                            msg: format!("Shorthand struct initialization"),
                            severity: Severity::WeakWarning,
                            fix: Some(LocalEdit {
                                label: "use struct shorthand initialization".to_string(),
                                edit,
                                cursor_position: None,
                            }),
                        });
                    }
                }
            }
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use crate::test_utils::assert_eq_text;

    use super::*;

    fn check_not_applicable(code: &str, func: fn(file: &SourceFileNode) -> Vec<Diagnostic>) {
        let file = SourceFileNode::parse(code);
        let diagnostics = func(&file);
        assert!(diagnostics.is_empty());
    }

    fn check_apply(before: &str, after: &str, func: fn(file: &SourceFileNode) -> Vec<Diagnostic>) {
        let file = SourceFileNode::parse(before);
        let diagnostic = func(&file)
            .pop()
            .unwrap_or_else(|| panic!("no diagnostics for:\n{}\n", before));
        let fix = diagnostic.fix.unwrap();
        let actual = fix.edit.apply(&before);
        assert_eq_text!(after, &actual);
    }

    #[test]
    fn test_check_unnecessary_braces_in_use_statement() {
        check_not_applicable(
            "
            use a;
            use a::{c, d::e};
        ",
            check_unnecessary_braces_in_use_statement,
        );
        check_apply(
            "use {b};",
            "use b;",
            check_unnecessary_braces_in_use_statement,
        );
        check_apply(
            "use a::{c};",
            "use a::c;",
            check_unnecessary_braces_in_use_statement,
        );
        check_apply(
            "use a::{self};",
            "use a;",
            check_unnecessary_braces_in_use_statement,
        );
        check_apply(
            "use a::{c, d::{e}};",
            "use a::{c, d::e};",
            check_unnecessary_braces_in_use_statement,
        );
    }

    #[test]
    fn test_check_struct_shorthand_initialization() {
        check_not_applicable(
            r#"
            struct A {
                a: &'static str
            }

            fn main() {
                A {
                    a: "hello"
                }
            }
        "#,
            check_struct_shorthand_initialization,
        );

        check_apply(
            r#"
struct A {
    a: &'static str
}

fn main() {
    let a = "haha";
    A {
        a: a
    }
}
        "#,
            r#"
struct A {
    a: &'static str
}

fn main() {
    let a = "haha";
    A {
        a
    }
}
        "#,
            check_struct_shorthand_initialization,
        );

        check_apply(
            r#"
struct A {
    a: &'static str,
    b: &'static str
}

fn main() {
    let a = "haha";
    let b = "bb";
    A {
        a: a,
        b
    }
}
        "#,
            r#"
struct A {
    a: &'static str,
    b: &'static str
}

fn main() {
    let a = "haha";
    let b = "bb";
    A {
        a,
        b
    }
}
        "#,
            check_struct_shorthand_initialization,
        );
    }
}
