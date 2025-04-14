"""CSS minification utilities using tinycss2."""

import tinycss2
import re
from typing import Dict, List, Any, Tuple, Optional, Union
from .parser import parse_stylesheet, get_selector_text, get_rule_declarations


def minify_css(css: Union[str, bytes]) -> str:
    """
    Minify CSS by removing comments, whitespace, and unnecessary characters.

    Args:
        css: The CSS code as string or bytes

    Returns:
        Minified CSS as a string
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    minified_css = ""

    for rule in rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            # Remove whitespace in selectors
            selector = re.sub(r'\s*([,>+~])\s*', r'\1', selector)

            declarations = get_rule_declarations(rule)
            # Filter out comments and whitespace
            declarations = [decl for decl in declarations if decl.type == "declaration"]

            # Serialize declarations without whitespace
            content = ""
            for decl in declarations:
                value = tinycss2.serialize(decl.value).strip()
                # Minimize color values
                if value.startswith('#'):
                    # Convert #RRGGBB to #RGB when possible
                    if len(value) == 7 and value[1] == value[2] and value[3] == value[4] and value[5] == value[6]:
                        value = '#' + value[1] + value[3] + value[5]

                important = "!important" if decl.important else ""
                content += f"{decl.name}:{value}{important};"

            if content:
                minified_css += f"{selector}{{{content}}}"

        elif rule.type == "at-rule":
            if rule.lower_at_keyword == "media" or rule.lower_at_keyword == "keyframes":
                prelude = tinycss2.serialize(rule.prelude).strip()

                # Recursively minify the content of the at-rule
                inner_css = tinycss2.serialize(rule.content)
                minified_inner = minify_css(inner_css)

                minified_css += f"@{rule.lower_at_keyword} {prelude}{{{minified_inner}}}"
            else:
                # For other at-rules like @charset, @import, etc.
                prelude = tinycss2.serialize(rule.prelude).strip()
                minified_css += f"@{rule.lower_at_keyword} {prelude};"

    return minified_css


def beautify_css(css: Union[str, bytes]) -> str:
    """
    Beautify CSS by adding proper indentation and formatting.

    Args:
        css: The CSS code as string or bytes

    Returns:
        Beautified CSS as a string
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    # First parse the CSS
    rules = parse_stylesheet(css)
    beautified_css = ""

    for rule in rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            # Format selector nicely (one selector per line for multiple selectors)
            if ',' in selector:
                selector = ',\n'.join([s.strip() for s in selector.split(',')])

            declarations = get_rule_declarations(rule)
            formatted_content = ""

            for decl in declarations:
                if decl.type == "declaration":
                    value = tinycss2.serialize(decl.value).strip()
                    important = " !important" if decl.important else ""
                    formatted_content += f"    {decl.name}: {value}{important};\n"
                elif decl.type == "comment":
                    formatted_content += f"    /* {decl.value} */\n"

            beautified_css += f"{selector} {{\n{formatted_content}}}\n\n"

        elif rule.type == "at-rule":
            if rule.lower_at_keyword == "media" or rule.lower_at_keyword == "keyframes":
                prelude = tinycss2.serialize(rule.prelude).strip()

                # Recursively beautify the content of the at-rule
                inner_css = tinycss2.serialize(rule.content)
                inner_rules = parse_stylesheet(inner_css)

                formatted_inner = ""
                for inner_rule in inner_rules:
                    if inner_rule.type == "qualified-rule":
                        inner_selector = get_selector_text(inner_rule)
                        inner_declarations = get_rule_declarations(inner_rule)

                        inner_content = ""
                        for inner_decl in inner_declarations:
                            if inner_decl.type == "declaration":
                                inner_value = tinycss2.serialize(inner_decl.value).strip()
                                inner_important = " !important" if inner_decl.important else ""
                                inner_content += f"        {inner_decl.name}: {inner_value}{inner_important};\n"
                            elif inner_decl.type == "comment":
                                inner_content += f"        /* {inner_decl.value} */\n"

                        formatted_inner += f"    {inner_selector} {{\n{inner_content}    }}\n\n"

                beautified_css += f"@{rule.lower_at_keyword} {prelude} {{\n{formatted_inner}}}\n\n"
            else:
                # For other at-rules like @charset, @import, etc.
                prelude = tinycss2.serialize(rule.prelude).strip()
                beautified_css += f"@{rule.lower_at_keyword} {prelude};\n\n"

        elif rule.type == "comment":
            beautified_css += f"/* {rule.value} */\n\n"

    return beautified_css


def sort_properties(css: Union[str, bytes]) -> str:
    """
    Sort CSS properties alphabetically within each rule.

    Args:
        css: The CSS code as string or bytes

    Returns:
        CSS with properties sorted alphabetically
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    sorted_css = ""

    for rule in rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            # Separate declarations and comments
            decls = []
            comments = []

            for item in declarations:
                if item.type == "declaration":
                    decls.append(item)
                elif item.type == "comment":
                    comments.append(item)

            # Sort declarations by property name
            sorted_decls = sorted(decls, key=lambda d: d.name)

            # Combine sorted declarations with comments
            sorted_content = ""
            for decl in sorted_decls:
                value = tinycss2.serialize(decl.value).strip()
                important = " !important" if decl.important else ""
                sorted_content += f"    {decl.name}: {value}{important};\n"

            # Add comments at the end
            for comment in comments:
                sorted_content += f"    /* {comment.value} */\n"

            sorted_css += f"{selector} {{\n{sorted_content}}}\n\n"

        else:
            # Keep other rules as they are
            sorted_css += tinycss2.serialize([rule]) + "\n"

    return sorted_css


def remove_duplicates(css: Union[str, bytes]) -> str:
    """
    Remove duplicate selectors and properties from CSS.

    Args:
        css: The CSS code as string or bytes

    Returns:
        CSS with duplicates removed
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    selectors_map = {}  # Maps selectors to rule index
    media_queries_map = {}  # Maps media query conditions to their rules

    # First pass: identify duplicates
    for i, rule in enumerate(rules):
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)

            if selector in selectors_map:
                # Duplicate selector found
                existing_idx = selectors_map[selector]
                existing_rule = rules[existing_idx]

                # Merge declarations
                existing_decls = get_rule_declarations(existing_rule)
                new_decls = get_rule_declarations(rule)

                # Track existing properties to avoid duplicates
                existing_props = {}
                for j, decl in enumerate(existing_decls):
                    if decl.type == "declaration":
                        existing_props[decl.name] = j

                # Add non-duplicate declarations
                for decl in new_decls:
                    if decl.type == "declaration":
                        if decl.name in existing_props:
                            # Replace the existing declaration (newer takes precedence)
                            existing_decls[existing_props[decl.name]] = decl
                        else:
                            # Add new declaration
                            existing_decls.append(decl)

                # Mark as deleted by setting to None
                rules[i] = None
            else:
                # First occurrence of this selector
                selectors_map[selector] = i

        elif rule.type == "at-rule" and rule.at_keyword.lower() == "media":
            # Handle media queries
            media_condition = tinycss2.serialize(rule.prelude).strip()
            
            if media_condition in media_queries_map:
                # Duplicate media query found
                existing_media_rule = media_queries_map[media_condition]
                
                # Parse the content of both media queries
                existing_content = parse_stylesheet(tinycss2.serialize(existing_media_rule.content))
                new_content = parse_stylesheet(tinycss2.serialize(rule.content))
                
                # Create a map of selectors to their rules in existing content
                existing_selectors = {}
                for existing_rule in existing_content:
                    if existing_rule.type == "qualified-rule":
                        selector = get_selector_text(existing_rule)
                        existing_selectors[selector] = existing_rule
                
                # Merge the content rules
                for content_rule in new_content:
                    if content_rule.type == "qualified-rule":
                        selector = get_selector_text(content_rule)
                        
                        if selector in existing_selectors:
                            # Merge declarations with existing rule
                            existing_rule = existing_selectors[selector]
                            existing_decls = get_rule_declarations(existing_rule)
                            new_decls = get_rule_declarations(content_rule)
                            
                            # Track existing properties
                            existing_props = {}
                            for j, decl in enumerate(existing_decls):
                                if decl.type == "declaration":
                                    existing_props[decl.name] = j
                            
                            # Add non-duplicate declarations
                            for decl in new_decls:
                                if decl.type == "declaration":
                                    if decl.name in existing_props:
                                        # Replace existing declaration
                                        existing_decls[existing_props[decl.name]] = decl
                                    else:
                                        # Add new declaration
                                        existing_decls.append(decl)
                        else:
                            # Add new selector rule
                            existing_content.append(content_rule)
                
                # Update the existing media rule's content
                existing_media_rule.content = existing_content
                # Mark current rule as deleted
                rules[i] = None
            else:
                # First occurrence of this media query
                media_queries_map[media_condition] = rule

    # Second pass: build result with duplicates removed
    cleaned_css = ""
    for rule in rules:
        if rule is None:
            continue  # Skip deleted rules

        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            # Remove duplicate properties
            unique_props = {}
            unique_decls = []

            for decl in declarations:
                if decl.type == "declaration":
                    # Newer declarations override older ones
                    unique_props[decl.name] = decl
                else:
                    # Keep non-declaration nodes (like comments)
                    unique_decls.append(decl)

            # Add unique declarations
            for decl in unique_props.values():
                unique_decls.append(decl)

            # Sort declarations for consistency
            declaration_nodes = [d for d in unique_decls if d.type == "declaration"]
            sorted_decls = sorted(declaration_nodes, key=lambda d: d.name)
            comment_nodes = [d for d in unique_decls if d.type == "comment"]

            # Format the content
            content = ""
            for decl in sorted_decls:
                value = tinycss2.serialize(decl.value).strip()
                important = " !important" if decl.important else ""
                content += f"    {decl.name}: {value}{important};\n"

            # Add comments at the end
            for comment in comment_nodes:
                content += f"    /* {comment.value} */\n"

            cleaned_css += f"{selector} {{\n{content}}}\n\n"

        elif rule.type == "at-rule" and rule.at_keyword.lower() == "media":
            # Format media query
            media_condition = tinycss2.serialize(rule.prelude).strip()
            
            # Format the content
            content = ""
            for content_rule in rule.content:
                if content_rule.type == "qualified-rule":
                    selector = get_selector_text(content_rule)
                    declarations = get_rule_declarations(content_rule)
                    
                    # Remove duplicate properties
                    unique_props = {}
                    unique_decls = []
                    
                    for decl in declarations:
                        if decl.type == "declaration":
                            # Newer declarations override older ones
                            unique_props[decl.name] = decl
                        else:
                            # Keep non-declaration nodes (like comments)
                            unique_decls.append(decl)
                    
                    # Add unique declarations
                    for decl in unique_props.values():
                        unique_decls.append(decl)
                    
                    # Sort declarations for consistency
                    declaration_nodes = [d for d in unique_decls if d.type == "declaration"]
                    sorted_decls = sorted(declaration_nodes, key=lambda d: d.name)
                    comment_nodes = [d for d in unique_decls if d.type == "comment"]
                    
                    # Format declarations
                    rule_content = ""
                    for decl in sorted_decls:
                        value = tinycss2.serialize(decl.value).strip()
                        important = " !important" if decl.important else ""
                        rule_content += f"        {decl.name}: {value}{important};\n"
                    
                    # Add comments at the end
                    for comment in comment_nodes:
                        rule_content += f"        /* {comment.value} */\n"
                    
                    content += f"    {selector} {{\n{rule_content}    }}\n\n"
                elif content_rule.type == "comment":
                    content += f"    /* {content_rule.value} */\n"
            
            cleaned_css += f"@media {media_condition} {{\n{content}}}\n\n"
        else:
            # Keep other rules as they are
            cleaned_css += tinycss2.serialize([rule]) + "\n"

    return cleaned_css
