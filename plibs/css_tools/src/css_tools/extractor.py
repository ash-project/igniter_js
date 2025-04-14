"""CSS extraction utilities using tinycss2."""

import tinycss2
import re
from typing import Dict, List, Any, Tuple, Optional, Union, Set
from .parser import parse_stylesheet, get_selector_text, get_rule_declarations


def extract_colors(css: Union[str, bytes]) -> Dict[str, List[str]]:
    """
    Extract all color values from CSS, including those in nested selectors.

    Args:
        css: The CSS code as string or bytes

    Returns:
        Dictionary mapping selectors to their color properties

    Raises:
        Exception: If the CSS cannot be properly parsed
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    # Validate CSS syntax before proceeding
    if css.count('{') != css.count('}'):
        raise Exception("CSS syntax error: Unbalanced braces")

    # Parse CSS for analysis
    rules = parse_stylesheet(css)

    # Check for parse errors
    for rule in rules:
        if hasattr(rule, 'type') and rule.type == 'error':
            raise Exception(f"CSS parse error: {getattr(rule, 'message', 'Unknown error')}")

    colors = {}

    # Regular expressions for different color formats
    hex_pattern = r'#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})'
    rgb_pattern = r'rgb\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)'
    rgba_pattern = r'rgba\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*,\s*[0-9.]+\s*\)'
    hsl_pattern = r'hsl\(\s*\d+\s*,\s*\d+%\s*,\s*\d+%\s*\)'
    hsla_pattern = r'hsla\(\s*\d+\s*,\s*\d+%\s*,\s*\d+%\s*,\s*[0-9.]+\s*\)'

    color_properties = [
        'color', 'background-color', 'border-color', 'border-top-color',
        'border-right-color', 'border-bottom-color', 'border-left-color',
        'outline-color', 'text-decoration-color', 'box-shadow', 'text-shadow'
    ]

    # Recursive function to process rules
    def process_rules(rule_list):
        for rule in rule_list:
            if rule.type == "qualified-rule":
                selector = get_selector_text(rule)
                declarations = get_rule_declarations(rule)
                for decl in declarations:
                    if decl.type == "declaration":
                        value = tinycss2.serialize(decl.value).strip()
                        # Check if it's a color property or has a color value
                        is_color_property = decl.name in color_properties
                        has_color_value = (
                            re.search(hex_pattern, value) or
                            re.search(rgb_pattern, value) or
                            re.search(rgba_pattern, value) or
                            re.search(hsl_pattern, value) or
                            re.search(hsla_pattern, value) or
                            value in ['black', 'white', 'red', 'green', 'blue', 'yellow',
                                     'purple', 'orange', 'brown', 'gray', 'transparent']
                        )
                        if is_color_property or has_color_value:
                            if selector not in colors:
                                colors[selector] = []
                            colors[selector].append(f"{decl.name}: {value}")

            # Process media queries and other at-rules with nested content
            elif rule.type == "at-rule" and rule.content is not None:
                # Parse nested rules
                nested_rules = parse_stylesheet(tinycss2.serialize(rule.content))
                # Recursively process nested rules
                process_rules(nested_rules)

    # Start processing rules
    process_rules(rules)

    return colors

def extract_media_queries(css: Union[str, bytes]) -> Dict[str, List[Dict[str, Any]]]:
    """
    Extract all media queries and their contents.

    Args:
        css: The CSS code as string or bytes

    Returns:
        Dictionary mapping media query conditions to their rules
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    media_queries = {}

    for rule in rules:
        if rule.type == "at-rule" and rule.lower_at_keyword == "media":
            condition = tinycss2.serialize(rule.prelude).strip()

            if condition not in media_queries:
                media_queries[condition] = []

            # Parse the content of the media query
            if hasattr(rule, 'content') and rule.content:
                inner_rules = tinycss2.parse_stylesheet(
                    rule.content, skip_whitespace=False, skip_comments=False
                )

                for inner_rule in inner_rules:
                    if inner_rule.type == "qualified-rule":
                        selector = get_selector_text(inner_rule)
                        declarations = get_rule_declarations(inner_rule)

                        props = {}
                        for decl in declarations:
                            if decl.type == "declaration":
                                props[decl.name] = tinycss2.serialize(decl.value).strip()

                        media_queries[condition].append({
                            "selector": selector,
                            "properties": props
                        })

    return media_queries


def extract_animations(css: Union[str, bytes]) -> Dict[str, Dict[str, Any]]:
    """
    Extract all CSS animations and keyframes.

    Args:
        css: The CSS code as string or bytes

    Returns:
        Dictionary mapping animation names to their keyframes
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    animations = {}
    animation_usage = {}

    # First pass: Find all @keyframes rules
    for rule in rules:
        if rule.type == "at-rule" and rule.lower_at_keyword == "keyframes":
            animation_name = tinycss2.serialize(rule.prelude).strip()

            keyframes = {}
            if hasattr(rule, 'content') and rule.content:
                keyframe_rules = tinycss2.parse_stylesheet(
                    rule.content, skip_whitespace=False, skip_comments=False
                )

                for keyframe_rule in keyframe_rules:
                    if keyframe_rule.type == "qualified-rule":
                        # The "selector" for keyframes is the percentage or keywords (from/to)
                        percentage = get_selector_text(keyframe_rule)
                        declarations = get_rule_declarations(keyframe_rule)

                        props = {}
                        for decl in declarations:
                            if decl.type == "declaration":
                                props[decl.name] = tinycss2.serialize(decl.value).strip()

                        keyframes[percentage] = props

            animations[animation_name] = keyframes

    # Second pass: Find all elements using animations
    for rule in rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            for decl in declarations:
                if decl.type == "declaration" and decl.name in ["animation", "animation-name"]:
                    value = tinycss2.serialize(decl.value).strip()
                    # Simple extraction, might need more complex parsing for multiple animations
                    animation_name = value.split()[0]

                    if animation_name not in animation_usage:
                        animation_usage[animation_name] = []
                    animation_usage[animation_name].append(selector)

    # Combine the results
    result = {}
    for name, keyframes in animations.items():
        result[name] = {
            "keyframes": keyframes,
            "used_by": animation_usage.get(name, [])
        }

    return result


def extract_selectors_by_property(css: Union[str, bytes], property_name: str) -> Dict[str, str]:
    """
    Extract all selectors that use a specific CSS property.

    Args:
        css: The CSS code as string or bytes
        property_name: The property name to search for

    Returns:
        Dictionary mapping selectors to their property values
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    selectors = {}

    for rule in rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            for decl in declarations:
                if decl.type == "declaration" and decl.name == property_name:
                    value = tinycss2.serialize(decl.value).strip()
                    selectors[selector] = value

    return selectors


def extract_comments(css: Union[str, bytes]) -> Dict[str, List[str]]:
    """
    Extract all comments from CSS and associate them with nearby rules when possible.

    Args:
        css: The CSS code as string or bytes

    Returns:
        Dictionary with comments categorized
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    result = {
        "standalone_comments": [],
        "rule_comments": {},
        "declaration_comments": {}
    }

    current_comments = []

    for i, rule in enumerate(rules):
        if rule.type == "comment":
            current_comments.append(rule.value)

            # Check if this is a standalone comment (not followed by a rule)
            if i == len(rules) - 1 or rules[i+1].type == "comment":
                result["standalone_comments"].extend(current_comments)
                current_comments = []

        elif rule.type == "qualified-rule" and current_comments:
            selector = get_selector_text(rule)

            if selector not in result["rule_comments"]:
                result["rule_comments"][selector] = []

            result["rule_comments"][selector].extend(current_comments)
            current_comments = []

            # Look for comments inside declarations
            declarations = get_rule_declarations(rule)
            inside_comments = []

            for decl in declarations:
                if decl.type == "comment":
                    inside_comments.append(decl.value)
                elif decl.type == "declaration" and inside_comments:
                    if selector not in result["declaration_comments"]:
                        result["declaration_comments"][selector] = {}

                    if decl.name not in result["declaration_comments"][selector]:
                        result["declaration_comments"][selector][decl.name] = []

                    result["declaration_comments"][selector][decl.name].extend(inside_comments)
                    inside_comments = []

        elif current_comments:
            # For other rule types like at-rules
            result["standalone_comments"].extend(current_comments)
            current_comments = []

    return result


def extract_unused_selectors(css: Union[str, bytes], html_content: str) -> List[str]:
    """
    Extract CSS selectors that are not used in the given HTML content.

    Args:
        css: The CSS code as string or bytes
        html_content: The HTML content to check against

    Returns:
        List of unused selectors
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    all_selectors = []
    unused_selectors = []

    for rule in rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            # Skip pseudo-elements and pseudo-classes for simplicity
            base_selector = re.sub(r'::?[a-zA-Z-]+(\([^)]*\))?', '', selector)

            # Process complex selectors
            parts = re.split(r'\s*[,>+~]\s*', base_selector)
            for part in parts:
                part = part.strip()
                if part and part not in all_selectors:
                    all_selectors.append(part)

    # Basic check for unused selectors
    for selector in all_selectors:
        # Extract class and ID selectors
        if selector.startswith('.'):
            # Class selector
            class_name = selector[1:]
            if f'class="{class_name}"' not in html_content and f"class='{class_name}'" not in html_content:
                unused_selectors.append(selector)
        elif selector.startswith('#'):
            # ID selector
            id_name = selector[1:]
            if f'id="{id_name}"' not in html_content and f"id='{id_name}'" not in html_content:
                unused_selectors.append(selector)
        else:
            # Element selector - more complex, would need proper HTML parsing
            pass

    return unused_selectors


def extract_fonts(css: Union[str, bytes]) -> Dict[str, List[Dict[str, Any]]]:
    """
    Extract all font-related properties.

    Args:
        css: The CSS code as string or bytes

    Returns:
        Dictionary mapping selectors to their font properties
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    rules = parse_stylesheet(css)
    fonts = {}

    font_properties = [
        'font', 'font-family', 'font-size', 'font-weight', 'font-style',
        'font-variant', 'line-height', 'text-transform', 'letter-spacing'
    ]

    for rule in rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            declarations = get_rule_declarations(rule)

            font_decls = []
            for decl in declarations:
                if decl.type == "declaration" and decl.name in font_properties:
                    value = tinycss2.serialize(decl.value).strip()
                    font_decls.append({
                        "property": decl.name,
                        "value": value
                    })

            if font_decls:
                if selector not in fonts:
                    fonts[selector] = []
                fonts[selector].extend(font_decls)

    return fonts
