"""CSS modification utilities using tinycss2."""

import tinycss2
from typing import Dict, List, Any, Tuple, Optional, Union
from .parser import parse_stylesheet, get_selector_text, get_rule_declarations


def add_property_to_selector(
    css: Union[str, bytes],
    selector: str,
    property_name: str,
    property_value: str,
    important: bool = False
) -> str:
    """
    Add a CSS property to a specific selector, or create the selector if it doesn't exist.

    Args:
        css: The CSS code as string or bytes
        selector: The CSS selector to modify
        property_name: The property name to add
        property_value: The property value to add
        important: Whether to mark the property as !important

    Returns:
        Modified CSS as a string
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    found_selector = False
    modified_css = ""

    for rule in rules:
        if rule.type == "qualified-rule":
            rule_selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            if rule_selector == selector:
                found_selector = True

                # Check if the property already exists
                property_exists = False
                for i, decl in enumerate(declarations):
                    if decl.type == "declaration" and decl.name == property_name:
                        property_exists = True
                        # Replace the existing property
                        declarations[i] = tinycss2.ast.Declaration(
                            name=property_name,
                            value=[
                                tinycss2.ast.WhitespaceToken(value=" ", line=0, column=0),
                                tinycss2.ast.IdentToken(value=property_value, line=0, column=0)
                            ],
                            important=important,
                            line=0,
                            column=0,
                            lower_name=property_name.lower(),
                        )
                        break

                # Add the property if it doesn't exist
                if not property_exists:
                    if declarations and declarations[-1].type != "whitespace":
                        declarations.append(tinycss2.ast.WhitespaceToken(line=0, column=0, value='\n    '))
                    declarations.append(
                        tinycss2.ast.Declaration(
                            name=property_name,
                            value=[
                                tinycss2.ast.WhitespaceToken(value=" ", line=0, column=0),
                                tinycss2.ast.IdentToken(value=property_value, line=0, column=0)
                            ],
                            important=important,
                            line=0,
                            column=0,
                            lower_name=property_name.lower(),
                        )
                    )

            # Format and add the rule to the result
            serialized_content = tinycss2.serialize(declarations).strip()
            serialized_content = "\n".join("    " + line.strip() for line in serialized_content.splitlines() if line.strip())
            formatted_rule = f"{rule_selector} {{\n{serialized_content}\n}}\n"
            modified_css += formatted_rule

        else:
            # Keep other rules as they are
            modified_css += tinycss2.serialize([rule])

    # Create the selector if it doesn't exist
    if not found_selector:
        new_rule = f"\n{selector} {{\n    {property_name}: {property_value}{' !important' if important else ''};\n}}\n"
        modified_css += new_rule

    return modified_css.strip()


def remove_property_from_selector(
    css: Union[str, bytes],
    selector: str,
    property_name: str
) -> str:
    """
    Remove a CSS property from a specific selector.

    Args:
        css: The CSS code as string or bytes
        selector: The CSS selector to modify
        property_name: The property name to remove

    Returns:
        Modified CSS as a string
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    modified_css = ""

    for rule in rules:
        if rule.type == "qualified-rule":
            rule_selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            if rule_selector == selector:
                # Filter out the property to remove
                filtered_declarations = []
                for decl in declarations:
                    if not (decl.type == "declaration" and decl.name == property_name):
                        filtered_declarations.append(decl)

                declarations = filtered_declarations

            # Only add the rule if it has declarations
            if declarations:
                serialized_content = tinycss2.serialize(declarations).strip()
                if serialized_content:  # Only include if there are actual declarations
                    serialized_content = "\n".join("    " + line.strip() for line in serialized_content.splitlines() if line.strip())
                    formatted_rule = f"{rule_selector} {{\n{serialized_content}\n}}\n"
                    modified_css += formatted_rule

        else:
            # Keep other rules as they are
            modified_css += tinycss2.serialize([rule])

    return modified_css.strip()


def remove_selector(css: Union[str, bytes], selector: str) -> str:
    """
    Remove a CSS selector and all its properties.

    Args:
        css: The CSS code as string or bytes
        selector: The CSS selector to remove

    Returns:
        Modified CSS as a string
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    modified_css = ""

    for rule in rules:
        if rule.type == "qualified-rule":
            rule_selector = get_selector_text(rule)

            if rule_selector != selector:
                # Keep rules that don't match the selector to be removed
                declarations = get_rule_declarations(rule)
                serialized_content = tinycss2.serialize(declarations).strip()
                serialized_content = "\n".join("    " + line.strip() for line in serialized_content.splitlines() if line.strip())
                formatted_rule = f"{rule_selector} {{\n{serialized_content}\n}}\n"
                modified_css += formatted_rule

        else:
            # Keep other rules as they are
            modified_css += tinycss2.serialize([rule])

    return modified_css.strip()


def modify_property_value(
    css: Union[str, bytes],
    selector: str,
    property_name: str,
    new_value: str,
    important: bool = None
) -> str:
    """
    Modify the value of a CSS property for a specific selector.

    Args:
        css: The CSS code as string or bytes
        selector: The CSS selector to modify
        property_name: The property name to modify
        new_value: The new property value
        important: Whether to mark the property as !important (None = keep current setting)

    Returns:
        Modified CSS as a string
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    modified_css = ""
    property_found = False

    for rule in rules:
        if rule.type == "qualified-rule":
            rule_selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            if rule_selector == selector:
                # Modify the property value
                for i, decl in enumerate(declarations):
                    if decl.type == "declaration" and decl.name == property_name:
                        property_found = True
                        # Use existing important flag if not specified
                        is_important = important if important is not None else decl.important
                        declarations[i] = tinycss2.ast.Declaration(
                            name=property_name,
                            value=[
                                tinycss2.ast.WhitespaceToken(value=" ", line=0, column=0),
                                tinycss2.ast.IdentToken(value=new_value, line=0, column=0)
                            ],
                            important=is_important,
                            line=0,
                            column=0,
                            lower_name=property_name.lower(),
                        )

            # Format and add the rule to the result
            serialized_content = tinycss2.serialize(declarations).strip()
            serialized_content = "\n".join("    " + line.strip() for line in serialized_content.splitlines() if line.strip())
            formatted_rule = f"{rule_selector} {{\n{serialized_content}\n}}\n"
            modified_css += formatted_rule

        else:
            # Keep other rules as they are
            modified_css += tinycss2.serialize([rule])

    # If the property wasn't found, add it
    if not property_found and selector:
        # Check if selector already exists in the result
        if selector not in modified_css:
            # Add new rule with the property
            new_rule = f"\n{selector} {{\n    {property_name}: {new_value}{' !important' if important else ''};\n}}\n"
            modified_css += new_rule
        else:
            # Property wasn't found but selector exists, so add_property would be needed
            # This is a bit tricky since we've already formatted the CSS
            # For simplicity, we'll call add_property on our current result
            return add_property_to_selector(modified_css, selector, property_name, new_value, important or False)

    return modified_css.strip()


def add_prefix_to_property(
    css: Union[str, bytes],
    property_name: str,
    prefixes: List[str]
) -> str:
    """
    Add vendor prefixes to a CSS property throughout the stylesheet.

    Args:
        css: The CSS code as string or bytes
        property_name: The property name to prefix
        prefixes: List of prefixes to add (e.g., ['-webkit-', '-moz-'])

    Returns:
        Modified CSS as a string
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    modified_css = ""

    for rule in rules:
        if rule.type == "qualified-rule":
            rule_selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            # Look for the property to prefix
            new_declarations = []
            for decl in declarations:
                new_declarations.append(decl)
                if decl.type == "declaration" and decl.name == property_name:
                    # Add prefixed versions before the standard property
                    value = tinycss2.serialize(decl.value)
                    for prefix in prefixes:
                        prefixed_prop = tinycss2.ast.Declaration(
                            name=f"{prefix}{property_name}",
                            value=decl.value,  # Use the same value as the original property
                            important=decl.important,
                            line=0,
                            column=0,
                            lower_name=f"{prefix}{property_name}".lower(),
                        )
                        # Insert prefixed property before the original
                        new_declarations.insert(len(new_declarations) - 1, prefixed_prop)
                        # Add whitespace between properties
                        new_declarations.insert(
                            len(new_declarations) - 1,
                            tinycss2.ast.WhitespaceToken(line=0, column=0, value='\n    ')
                        )

            # Format and add the rule to the result
            serialized_content = tinycss2.serialize(new_declarations).strip()
            serialized_content = "\n".join("    " + line.strip() for line in serialized_content.splitlines() if line.strip())
            formatted_rule = f"{rule_selector} {{\n{serialized_content}\n}}\n"
            modified_css += formatted_rule

        else:
            # Keep other rules as they are
            modified_css += tinycss2.serialize([rule])

    return modified_css.strip()


def merge_stylesheets(css_list: List[Union[str, bytes]]) -> str:
    """
    Merge multiple CSS stylesheets into one, removing duplicates.

    Args:
        css_list: List of CSS stylesheets as strings or bytes

    Returns:
        Merged CSS as a string
    """
    all_rules = []
    selector_map = {}  # Maps selectors to their rule index in all_rules

    for css in css_list:
        if isinstance(css, bytes):
            css = css.decode('utf-8')

        rules = parse_stylesheet(css)

        for rule in rules:
            if rule.type == "qualified-rule":
                selector = get_selector_text(rule)

                if selector in selector_map:
                    # Merge declarations with existing rule
                    existing_rule_idx = selector_map[selector]
                    existing_rule = all_rules[existing_rule_idx]

                    # Get declarations from both rules
                    existing_decls = get_rule_declarations(existing_rule)
                    new_decls = get_rule_declarations(rule)

                    # Create a map of existing declarations to avoid duplicates
                    existing_props = {
                        decl.name: i
                        for i, decl in enumerate(existing_decls)
                        if decl.type == "declaration"
                    }

                    # Add new declarations if they don't exist
                    for decl in new_decls:
                        if decl.type == "declaration":
                            if decl.name in existing_props:
                                # Replace existing declaration (newer takes precedence)
                                existing_decls[existing_props[decl.name]] = decl
                            else:
                                # Add new declaration
                                existing_decls.append(decl)

                    # Update the rule content
                    existing_rule.content = tinycss2.serialize(existing_decls)
                else:
                    # Add new rule
                    all_rules.append(rule)
                    selector_map[selector] = len(all_rules) - 1
            else:
                # For at-rules and comments, just add them
                all_rules.append(rule)

    # Serialize the merged rules
    merged_css = ""
    for rule in all_rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            serialized_content = tinycss2.serialize(declarations).strip()
            serialized_content = "\n".join("    " + line.strip() for line in serialized_content.splitlines() if line.strip())
            formatted_rule = f"{selector} {{\n{serialized_content}\n}}\n"
            merged_css += formatted_rule
        else:
            merged_css += tinycss2.serialize([rule])

    return merged_css.strip()
