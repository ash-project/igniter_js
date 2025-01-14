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
    // Resource Atoms
}
