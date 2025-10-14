// SPDX-FileCopyrightText: 2024 Shahryar Tavakkoli
// SPDX-FileCopyrightText: 2024 igniter_js contributors <https://github.com/ash-project/igniter_js/graphs.contributors>
//
// SPDX-License-Identifier: MIT

rustler::atoms! {
    // Success Atoms
    ok,

    // Error Atoms
    error,

    // Nif Functions Atoms
    source_to_ast_nif,
    is_module_imported_from_ast_nif,
    insert_import_to_ast_nif,
    remove_import_from_ast_nif,
    find_live_socket_node_from_ast,
    extend_hook_object_to_ast_nif,
    remove_objects_of_hooks_from_ast_nif,
    statistics_from_ast_nif,
    extend_var_object_property_by_names_to_ast_nif,
    contains_variable_from_ast_nif,
    format_css_nif,
    is_css_formatted_nif,
    format_js_nif,
    is_js_formatted_nif,
    convert_ast_to_estree_nif,
    insert_ast_at_index_nif,
    replace_ast_at_index_nif,
    // Resource Atoms
}
