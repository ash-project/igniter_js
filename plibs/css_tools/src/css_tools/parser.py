"""CSS parsing utilities using tinycss2."""

import tinycss2
from typing import Dict, List, Any, Tuple, Optional, Union


def parse_stylesheet(css: Union[str, bytes]) -> List[Any]:
    """
    Parse a CSS stylesheet into a list of rules.

    Args:
        css: The CSS code as string or bytes

    Returns:
        List of tinycss2 nodes representing the stylesheet
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')
    return tinycss2.parse_stylesheet(css, skip_whitespace=False, skip_comments=False)


def parse_declarations(declarations_str: str) -> List[Any]:
    """
    Parse a CSS declaration list into a list of declarations.

    Args:
        declarations_str: The CSS declarations as a string

    Returns:
        List of tinycss2 declarations
    """
    return tinycss2.parse_declaration_list(
        declarations_str, skip_whitespace=False, skip_comments=False
    )


def serialize_stylesheet(rules: List[Any]) -> str:
    """
    Serialize a list of CSS rules back to a CSS string.

    Args:
        rules: List of tinycss2 nodes

    Returns:
        CSS code as string
    """
    return tinycss2.serialize(rules)


def serialize_declarations(declarations: List[Any]) -> str:
    """
    Serialize a list of CSS declarations back to a CSS string.

    Args:
        declarations: List of tinycss2 declarations

    Returns:
        CSS declarations as string
    """
    return tinycss2.serialize(declarations)


def get_rule_declarations(rule: Any) -> List[Any]:
    """
    Extract the declarations from a CSS rule.

    Args:
        rule: A tinycss2.ast.QualifiedRule object

    Returns:
        List of declarations
    """
    if not hasattr(rule, 'content'):
        return []
    return tinycss2.parse_declaration_list(
        rule.content, skip_whitespace=False, skip_comments=False
    )


def get_selector_text(rule: Any) -> str:
    """
    Extract the selector text from a CSS rule.

    Args:
        rule: A tinycss2.ast.QualifiedRule object

    Returns:
        Selector text as string
    """
    if not hasattr(rule, 'prelude'):
        return ""
    return tinycss2.serialize(rule.prelude).strip()


def extract_rules_by_selector(css: Union[str, bytes], selector_pattern: str) -> List[Any]:
    """
    Extract all rules that match a given selector pattern.

    Args:
        css: The CSS code as string or bytes
        selector_pattern: The selector pattern to match (can be partial)

    Returns:
        List of matching rules
    """
    rules = parse_stylesheet(css)
    matching_rules = []

    for rule in rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            if selector_pattern in selector:
                matching_rules.append(rule)

    return matching_rules


def analyze_stylesheet(css: Union[str, bytes]) -> Dict[str, Any]:
    """
    Analyze a CSS stylesheet and return various statistics.

    Args:
        css: The CSS code as string or bytes

    Returns:
        Dictionary with statistics about the stylesheet
    """
    rules = parse_stylesheet(css)

    selectors = []
    properties = {}
    colors = []
    fonts = []
    media_queries = []
    comments = []

    for rule in rules:
        if rule.type == "qualified-rule":
            selector = get_selector_text(rule)
            selectors.append(selector)

            declarations = get_rule_declarations(rule)
            for decl in declarations:
                if decl.type == "declaration":
                    if decl.name not in properties:
                        properties[decl.name] = 0
                    properties[decl.name] += 1

                    value = tinycss2.serialize(decl.value).strip()
                    if decl.name in ["color", "background-color", "border-color"] or "#" in value:
                        colors.append(value)
                    if decl.name in ["font-family", "font"]:
                        fonts.append(value)

        elif rule.type == "at-rule" and rule.lower_at_keyword == "media":
            media_queries.append(tinycss2.serialize(rule.prelude).strip())

        elif rule.type == "comment":
            comments.append(rule.value)

    return {
        "selectors_count": len(selectors),
        "unique_selectors": len(set(selectors)),
        "properties_count": sum(properties.values()),
        "unique_properties": len(properties),
        "most_used_properties": sorted(properties.items(), key=lambda x: x[1], reverse=True)[:10],
        "colors_used": len(set(colors)),
        "fonts_used": len(set(fonts)),
        "media_queries": len(media_queries),
        "comments_count": len(comments),
        "file_size_bytes": len(css) if isinstance(css, bytes) else len(css.encode('utf-8')),
    }
