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


def remove_selector(css: Union[str, bytes], selector: Union[str, bytes]) -> str:
    """
    Remove a CSS selector and all its properties.

    Args:
        css: The CSS code as string or bytes
        selector: The CSS selector to remove

    Returns:
        Modified CSS as a string

    Raises:
        Exception: If the CSS cannot be properly parsed
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    # Ensure selector is also a string, not bytes
    if isinstance(selector, bytes):
        selector = selector.decode('utf-8')

    # Validate CSS syntax before proceeding
    # Check for unbalanced braces - a common CSS error
    if css.count('{') != css.count('}'):
        raise Exception("CSS syntax error: Unbalanced braces")

    # Parse CSS for further analysis
    rules = parse_stylesheet(css)

    # Check for parse errors
    for rule in rules:
        if hasattr(rule, 'type') and rule.type == 'error':
            raise Exception(f"CSS parse error: {getattr(rule, 'message', 'Unknown error')}")

    # Function to process rule blocks with potential nesting
    def process_rule_block(rules):
        result = ""
        for rule in rules:
            if rule.type == "qualified-rule":
                rule_selector = get_selector_text(rule)
                if rule_selector != selector:
                    # Keep rules that don't match the selector to be removed
                    declarations = get_rule_declarations(rule)
                    serialized_content = tinycss2.serialize(declarations).strip()
                    serialized_content = "\n".join("    " + line.strip() for line in serialized_content.splitlines() if line.strip())
                    formatted_rule = f"{rule_selector} {{\n{serialized_content}\n}}\n"
                    result += formatted_rule
            elif rule.type == "at-rule" and rule.content is not None:
                # Handle at-rules with blocks (e.g., media queries)
                at_keyword = rule.at_keyword
                prelude = tinycss2.serialize(rule.prelude).strip()

                # Parse the content of the at-rule
                inner_rules = parse_stylesheet(tinycss2.serialize(rule.content))

                # Process the inner rules recursively
                inner_content = process_rule_block(inner_rules)

                # Only include the at-rule if it has content after processing
                if inner_content.strip():
                    result += f"@{at_keyword} {prelude} {{\n{inner_content}\n}}\n"
            else:
                # Keep other rules as they are
                result += tinycss2.serialize([rule])

        return result

    # Start processing from the top level
    modified_css = process_rule_block(rules)

    return modified_css.strip()

def modify_property_value(
    css: Union[str, bytes],
    selector: Union[str, bytes],
    property_name: Union[str, bytes],
    new_value: Union[str, bytes],
    important: Optional[bool] = None
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
    # Convert all byte parameters to strings
    if isinstance(css, bytes):
        css = css.decode('utf-8')
    if isinstance(selector, bytes):
        selector = selector.decode('utf-8')
    if isinstance(property_name, bytes):
        property_name = property_name.decode('utf-8')
    if isinstance(new_value, bytes):
        new_value = new_value.decode('utf-8')

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
    property_name: Union[str, bytes],
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
    if isinstance(property_name, bytes):
        property_name = property_name.decode('utf-8')

    rules = parse_stylesheet(css)
    modified_css = ""

    def process_declarations(declarations):
        """Helper function to process declarations and add prefixes"""
        new_declarations = []
        for decl in declarations:
            new_declarations.append(decl)
            if decl.type == "declaration" and decl.name == property_name:
                # Add prefixed versions before the standard property
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
        return new_declarations

    def process_rules(rules_list, indent_level=0):
        """Recursively process rules, handling nested at-rules"""
        result = ""
        indent = "    " * indent_level

        for rule in rules_list:
            if rule.type == "qualified-rule":
                # Regular CSS rule
                rule_selector = get_selector_text(rule)
                declarations = get_rule_declarations(rule)

                # Process declarations to add prefixes
                new_declarations = process_declarations(declarations)

                # Format and add the rule to the result
                serialized_content = tinycss2.serialize(new_declarations).strip()
                serialized_content = "\n".join(indent + "    " + line.strip()
                                              for line in serialized_content.splitlines() if line.strip())

                formatted_rule = f"{indent}{rule_selector} {{\n{serialized_content}\n{indent}}}\n"
                result += formatted_rule

            elif rule.type == "at-rule" and rule.content is not None:
                # Handle at-rules with blocks like @media
                at_keyword = rule.at_keyword
                prelude = tinycss2.serialize(rule.prelude).strip()

                # Parse the content of the at-rule
                content_rules = parse_stylesheet(tinycss2.serialize(rule.content))

                # Process the nested rules
                inner_content = process_rules(content_rules, indent_level + 1)

                # Format the at-rule
                formatted_at_rule = f"{indent}@{at_keyword} {prelude} {{\n{inner_content}{indent}}}\n"
                result += formatted_at_rule

            else:
                # Keep other rules as they are (at-rules without blocks, comments, etc.)
                result += indent + tinycss2.serialize([rule])

        return result

    # Start processing from the top level
    modified_css = process_rules(rules)

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

def replace_selector_rule(css: Union[str, bytes], selector: Union[str, bytes], new_declarations: Union[str, bytes]) -> str:
    """
    Replace an entire CSS rule for a specific selector with new declarations.

    Args:
        css: The CSS code as string or bytes
        selector: The CSS selector to replace
        new_declarations: The new CSS declarations as a string (without curly braces)

    Returns:
        Modified CSS as a string

    Raises:
        Exception: If the CSS cannot be properly parsed or new declarations are invalid
    """
    # Ensure input types are correct
    if isinstance(css, bytes):
        css = css.decode('utf-8')
    if isinstance(selector, bytes):
        selector = selector.decode('utf-8')
    if isinstance(new_declarations, bytes):
        new_declarations = new_declarations.decode('utf-8')

    # Validate CSS syntax before proceeding
    if css.count('{') != css.count('}'):
        raise Exception("CSS syntax error: Unbalanced braces")

    # Basic validation of the original CSS by parsing it
    try:
        rules = parse_stylesheet(css)

        # Check for parse errors in the original CSS
        for rule in rules:
            if hasattr(rule, 'type') and rule.type == 'error':
                raise Exception(f"CSS parse error: {getattr(rule, 'message', 'Unknown error')}")
    except Exception as e:
        raise Exception(f"Failed to parse CSS: {str(e)}")

    # Validate new declarations syntax - ensure each declaration ends with a semicolon
    declarations_list = [d.strip() for d in new_declarations.split(';') if d.strip()]
    for decl in declarations_list:
        if ':' not in decl:
            raise Exception(f"Invalid declaration syntax: Missing colon in '{decl}'")

    # Reconstruct new_declarations with proper formatting and ensure semicolons
    new_declarations = '; '.join(declarations_list) + ';'

    try:
        # Simple validation by trying to parse a test rule
        test_css = f".test{{ {new_declarations} }}"
        test_rules = parse_stylesheet(test_css)
        for rule in test_rules:
            if hasattr(rule, 'type') and rule.type == 'error':
                raise Exception(f"Invalid declaration syntax: {getattr(rule, 'message', 'Unknown error')}")
    except Exception as e:
        raise Exception(f"Invalid declaration syntax: {str(e)}")

    # Flatten nested selectors if present in the CSS
    flattened_css = ""
    selector_found = False

    def flatten_nested_css(rules, parent_selector=None):
        nonlocal flattened_css, selector_found

        for rule in rules:
            if rule.type == "qualified-rule":
                current_selector = get_selector_text(rule)
                combined_selector = current_selector

                if parent_selector:
                    combined_selector = f"{parent_selector} {current_selector}"

                if combined_selector == selector:
                    # Found the selector to replace
                    selector_found = True
                    formatted_declarations = "\n".join(f"    {decl};" for decl in new_declarations.split(';') if decl.strip())
                    flattened_css += f"{selector} {{\n{formatted_declarations}\n}}\n"
                else:
                    # Keep other rules
                    declarations = get_rule_declarations(rule)

                    # Check if this rule contains more nested rules
                    nested_rules = []
                    for item in declarations:
                        if item.type == "qualified-rule":
                            nested_rules.append(item)

                    if nested_rules:
                        # Process nested rules
                        flatten_nested_css(nested_rules, combined_selector)
                    else:
                        # Regular rule - add it to output
                        serialized_content = tinycss2.serialize(declarations).strip()
                        if serialized_content:  # Only add if there's content
                            serialized_content = "\n".join(f"    {line.strip()}" for line in serialized_content.splitlines() if line.strip())
                            flattened_css += f"{combined_selector} {{\n{serialized_content}\n}}\n"

            elif rule.type == "at-rule" and rule.content:
                # Handle at-rules like media queries
                at_keyword = rule.at_keyword
                prelude = tinycss2.serialize(rule.prelude).strip()

                # Store the current CSS position
                current_css_length = len(flattened_css)

                # Process nested rules in the at-rule
                inner_rules = parse_stylesheet(tinycss2.serialize(rule.content))
                flatten_nested_css(inner_rules)

                # If content was added, wrap it in the at-rule
                if len(flattened_css) > current_css_length:
                    at_rule_content = flattened_css[current_css_length:]
                    flattened_css = flattened_css[:current_css_length]
                    flattened_css += f"@{at_keyword} {prelude} {{\n{at_rule_content}}}\n"
            else:
                # Other rules like comments
                flattened_css += tinycss2.serialize([rule])

    # Process the CSS
    flatten_nested_css(rules)

    # Add the selector if not found
    if not selector_found:
        formatted_declarations = "\n".join(f"    {decl};" for decl in new_declarations.split(';') if decl.strip())
        flattened_css += f"\n{selector} {{\n{formatted_declarations}\n}}\n"

    return flattened_css.strip()
