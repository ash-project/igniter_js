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
    
    # Ensure content is properly serialized before parsing
    content = rule.content
    if isinstance(content, str):
        return tinycss2.parse_declaration_list(content, skip_whitespace=False, skip_comments=False)
    else:
        # Serialize the content if it's not a string
        serialized_content = tinycss2.serialize(content)
        # Remove any surrounding braces if present
        serialized_content = serialized_content.strip('{}')
        return tinycss2.parse_declaration_list(serialized_content, skip_whitespace=False, skip_comments=False)


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


def extract_comments(css: Union[str, bytes]) -> List[str]:
    """Extract all comments from CSS code."""
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    all_comments = []
    # Use the correct tinycss2 API for tokenization
    tokens = tinycss2.parse_component_value_list(css)
    for token in tokens:
        if token.type == 'comment':
            all_comments.append(token.value)
    return all_comments


def extract_colors_and_fonts(value: str, property_name: str) -> Tuple[List[str], List[str]]:
    """Extract colors and fonts from a CSS property value."""
    colors = []
    fonts = []

    # Check for color properties
    if property_name in ["color", "background-color", "border-color"] or "#" in value:
        colors.append(value)

    # Check for font properties
    if property_name in ["font-family", "font"]:
        fonts.append(value)

    return colors, fonts


def process_declaration(declaration, properties, colors, fonts,
                       selector_properties, selector, parent_media, media_query_details):
    """Process a single CSS declaration."""
    if declaration.type != "declaration":
        return [], []

    property_name = declaration.name
    value = tinycss2.serialize(declaration.value).strip()

    # Track property usage
    if property_name not in properties:
        properties[property_name] = 0
    properties[property_name] += 1

    # Track property in media query if applicable
    if parent_media:
        if property_name not in media_query_details[parent_media]["properties"]:
            media_query_details[parent_media]["properties"][property_name] = 0
        media_query_details[parent_media]["properties"][property_name] += 1

    # Store the property value for this selector
    selector_properties[selector][property_name] = value

    # Extract colors and fonts
    new_colors, new_fonts = extract_colors_and_fonts(value, property_name)

    return new_colors, new_fonts


def process_qualified_rule(rule, parent_media, selectors, properties, colors, fonts,
                          selector_properties, media_query_details):
    """Process a qualified CSS rule (selector with declarations)."""
    selector = get_selector_text(rule)

    # Track media query relationship
    if parent_media:
        if parent_media not in media_query_details:
            media_query_details[parent_media] = {
                "selectors": [],
                "properties": {}
            }
        media_query_details[parent_media]["selectors"].append(selector)

    selectors.append(selector)

    # Initialize selector_properties entry
    if selector not in selector_properties:
        selector_properties[selector] = {}

    declarations = get_rule_declarations(rule)
    for decl in declarations:
        new_colors, new_fonts = process_declaration(
            decl, properties, colors, fonts,
            selector_properties, selector, parent_media, media_query_details
        )
        colors.extend(new_colors)
        fonts.extend(new_fonts)


def process_media_rule(rule, media_query_list, media_query_details,
                      selectors, properties, colors, fonts, selector_properties):
    """Process a media query rule."""
    if not rule.content:
        return

    media_query = tinycss2.serialize(rule.prelude).strip()
    media_query_list.append(media_query)

    # Process rules inside the media query
    inner_rules = parse_stylesheet(tinycss2.serialize(rule.content))
    process_rule_block(
        inner_rules, media_query, selectors, properties,
        colors, fonts, media_query_list, media_query_details, selector_properties
    )


def process_rule_block(rule_block, parent_media, selectors, properties,
                      colors, fonts, media_query_list, media_query_details, selector_properties):
    """Process a block of CSS rules, handling nested structures."""
    for rule in rule_block:
        if rule.type == "qualified-rule":
            process_qualified_rule(
                rule, parent_media, selectors, properties, colors, fonts,
                selector_properties, media_query_details
            )
        elif rule.type == "at-rule" and rule.at_keyword.lower() == "media":
            process_media_rule(
                rule, media_query_list, media_query_details,
                selectors, properties, colors, fonts, selector_properties
            )


def extract_imports(rules):
    """
    Extract @import rules from CSS using AST approach.

    Args:
        rules: List of CSS rules from tinycss2 parser

    Returns:
        Tuple of (imports list, import_media_queries dict)
    """
    imports = []
    import_media_queries = {}

    for rule in rules:
        if rule.type == "at-rule" and rule.lower_at_keyword == "import":
            # Parse the import URL
            import_url = None
            media_tokens = []

            # Process prelude tokens to extract URL and media query
            prelude_tokens = rule.prelude

            # Process each token to find URL (in either format) and media query
            i = 0
            while i < len(prelude_tokens):
                token = prelude_tokens[i]

                # Handle url token
                if token.type == "function" and token.lower_name == "url":
                    # Extract URL from inside the url() function
                    url_content = tinycss2.parse_component_value_list(tinycss2.serialize(token.arguments))
                    for arg in url_content:
                        if arg.type in ["string", "ident"]:
                            import_url = arg.value
                            break

                    # Media query would be remaining tokens
                    media_tokens = prelude_tokens[i+1:]
                    break

                # Handle string token
                elif token.type == "string":
                    import_url = token.value
                    # Media query would be remaining tokens
                    media_tokens = prelude_tokens[i+1:]
                    break

                i += 1

            # If we found a URL, process it
            if import_url:
                imports.append(import_url)

                # Extract media query if present
                if media_tokens:
                    media_query_text = tinycss2.serialize(media_tokens).strip()
                    if media_query_text:
                        import_media_queries[import_url] = media_query_text

    return imports, import_media_queries


def analyze_stylesheet(css: Union[str, bytes]) -> Dict[str, Any]:
    """
    Analyze a CSS stylesheet and return various statistics.

    Args:
        css: The CSS code as string or bytes

    Returns:
        Dictionary with statistics and detailed information about the stylesheet

    Raises:
        Exception: If the CSS cannot be properly parsed
    """
    if isinstance(css, bytes):
        css = css.decode('utf-8')

    # Validate CSS syntax before proceeding
    # Check for unbalanced braces - a common CSS error
    if css.count('{') != css.count('}'):
        raise Exception("CSS syntax error: Unbalanced braces")

    # Extract all comments using the existing function
    all_comments = extract_comments(css)

    try:
        # Parse CSS for further analysis
        rules = parse_stylesheet(css)

        # Check for parse errors
        for rule in rules:
            if hasattr(rule, 'type') and rule.type == 'error':
                raise Exception(f"CSS parse error: {getattr(rule, 'message', 'Unknown error')}")

        # Extract imports using AST approach
        imports, import_media_queries = extract_imports(rules)

    except Exception as e:
        # Re-raise any parsing exceptions with a clear message
        raise Exception(f"Failed to parse CSS: {str(e)}")

    # Initialize data structures
    selectors = []
    properties = {}
    colors = []
    fonts = []
    media_query_list = []
    media_query_details = {}
    selector_properties = {}

    # Process CSS rules
    try:
        process_rule_block(
            rules, None, selectors, properties, colors, fonts,
            media_query_list, media_query_details, selector_properties
        )
    except Exception as e:
        raise Exception(f"Error analyzing CSS structure: {str(e)}")

    # Return the analysis results
    return {
        "selectors": selectors,
        "selectors_count": len(selectors),
        "unique_selectors": len(set(selectors)),
        "properties_count": sum(properties.values()),
        "unique_properties": len(properties),
        "most_used_properties": sorted(properties.items(), key=lambda x: x[1], reverse=True)[:10],
        "colors_used": len(set(colors)),
        "colors": list(set(colors)),
        "fonts_used": len(set(fonts)),
        "fonts": list(set(fonts)),
        "media_queries_count": len(media_query_list),
        "media_queries": media_query_list,
        "media_query_details": media_query_details,
        "comments_count": len(all_comments),
        "comments": all_comments,
        "file_size_bytes": len(css.encode('utf-8')),
        "selector_properties": selector_properties,
        "imports": imports,
        "imports_count": len(imports),
        "import_media_queries": import_media_queries
    }
