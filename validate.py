#!/usr/bin/env python3
"""
Validate XML files in a zip archive against their JSON schemas.

Verifies:
1. Root element naming matches schema
2. Child record element naming matches schema
3. Property completeness across schema groups
"""

import json
import re
import sys
import zipfile
from collections import defaultdict
from pathlib import Path
from typing import Dict, Set, Tuple


def get_schema_name(xml_filename: str) -> str:
    """Map XML filename to schema filename by removing _N suffix."""
    # Remove .xml extension
    base = xml_filename.rsplit('.', 1)[0]
    # Remove _N suffix (e.g., AnlagenEegSolar_1 -> AnlagenEegSolar)
    base = re.sub(r'_\d+$', '', base)
    return f"{base}.json"


def extract_xml_structure_streaming(xml_content: bytes) -> Tuple[str, str, Set[str]]:
    """
    Extract root element, child element, and all property names from XML.

    Uses fast string processing instead of full XML parser.
    Assumes:
    - XML is well-formed
    - Root element is at the start
    - Child elements are direct children of root
    - Properties are direct children of child elements

    Returns:
        (root_element, child_element, set of property names)
    """
    # Decode from UTF-16 if needed (check BOM or encoding declaration)
    text = xml_content.decode('utf-16', errors='ignore')

    # Find root element - match first opening tag after <?xml...?>
    root_match = re.search(r'<\?xml[^>]*\?>\s*<(\w+)', text)
    if not root_match:
        # Try without XML declaration
        root_match = re.search(r'^\s*<(\w+)', text)

    if not root_match:
        raise ValueError("Could not find root element")

    root_element = root_match.group(1)

    # Find child record element - first child of root
    # Pattern: <RootElement><ChildElement>
    child_pattern = re.compile(f'<{re.escape(root_element)}[^>]*>\\s*<(\\w+)[^>]*>')
    child_match = child_pattern.search(text)

    if not child_match:
        raise ValueError(f"Could not find child element under root {root_element}")

    child_element = child_match.group(1)

    # Extract all property names
    # Find all opening tags that are NOT the root or child element
    # We want tags like <PropertyName>value</PropertyName>
    property_names = set()

    # Simple approach: find all opening tags
    tag_pattern = re.compile(r'<(\w+)[^>]*>')
    for match in tag_pattern.finditer(text):
        tag_name = match.group(1)
        # Skip root and child elements
        if tag_name not in (root_element, child_element):
            property_names.add(tag_name)

    return root_element, child_element, property_names


def load_schema(schema_path: Path) -> Tuple[str, str, Set[str]]:
    """Load schema and extract root, element, and field names."""
    with open(schema_path, 'r', encoding='utf-8') as f:
        schema = json.load(f)

    root = schema['root']
    element = schema['element']
    fields = {field['name'] for field in schema['fields']}

    return root, element, fields


def validate_zip(zip_path: Path, schema_dir: Path) -> bool:
    """
    Validate all XML files in zip against their schemas.

    Returns True if validation passes, False otherwise.
    """
    # Group XML files by schema
    schema_to_xmls: Dict[str, list] = defaultdict(list)

    # Track properties found in each XML file (grouped by schema)
    schema_properties: Dict[str, Set[str]] = defaultdict(set)

    errors = []
    success = True

    with zipfile.ZipFile(zip_path, 'r') as zf:
        xml_files = [name for name in zf.namelist() if name.endswith('.xml')]

        print(f"Validating {len(xml_files)} XML files from {zip_path.name}...")

        for xml_filename in sorted(xml_files):
            schema_filename = get_schema_name(xml_filename)
            schema_path = schema_dir / schema_filename

            # Check if schema exists
            if not schema_path.exists():
                errors.append(f"❌ {xml_filename}: Schema not found: {schema_filename}")
                success = False
                continue

            # Load schema
            try:
                schema_root, schema_element, schema_fields = load_schema(schema_path)
            except Exception as e:
                errors.append(f"❌ {xml_filename}: Failed to load schema {schema_filename}: {e}")
                success = False
                continue

            # Read and parse XML
            try:
                xml_content = zf.read(xml_filename)
                xml_root, xml_element, xml_properties = extract_xml_structure_streaming(xml_content)
            except Exception as e:
                errors.append(f"❌ {xml_filename}: Failed to parse XML: {e}")
                success = False
                continue

            # Validate root element
            if xml_root != schema_root:
                errors.append(
                    f"❌ {xml_filename}: Root element mismatch: "
                    f"expected '{schema_root}', found '{xml_root}'"
                )
                success = False

            # Validate child element
            if xml_element != schema_element:
                errors.append(
                    f"❌ {xml_filename}: Child element mismatch: "
                    f"expected '{schema_element}', found '{xml_element}'"
                )
                success = False

            # Track properties for schema group validation
            schema_to_xmls[schema_filename].append(xml_filename)
            schema_properties[schema_filename].update(xml_properties)

            print(f"✓ {xml_filename}: {len(xml_properties)} properties found")

    # Validate property completeness across schema groups
    print("\nValidating property completeness across schema groups...")

    for schema_filename in sorted(schema_properties.keys()):
        schema_path = schema_dir / schema_filename

        try:
            _, _, schema_fields = load_schema(schema_path)
        except Exception as e:
            errors.append(f"❌ Schema {schema_filename}: Failed to load: {e}")
            success = False
            continue

        xml_files_for_schema = schema_to_xmls[schema_filename]
        found_properties = schema_properties[schema_filename]

        # Check for unexpected properties (in XML but not in schema)
        unexpected = found_properties - schema_fields
        if unexpected:
            errors.append(
                f"❌ Schema {schema_filename} (used by {len(xml_files_for_schema)} file(s)): "
                f"Unexpected properties found: {sorted(unexpected)}"
            )
            success = False

        # Check for unused schema properties (in schema but not in any XML)
        unused = schema_fields - found_properties
        if unused:
            errors.append(
                f"❌ Schema {schema_filename} (used by {len(xml_files_for_schema)} file(s)): "
                f"Unused schema properties: {sorted(unused)}"
            )
            success = False

        if not unexpected and not unused:
            print(f"✓ Schema {schema_filename}: All {len(schema_fields)} properties validated")

    # Print errors
    if errors:
        print("\n" + "="*80)
        print("VALIDATION ERRORS:")
        print("="*80)
        for error in errors:
            print(error)
        print()

    return success


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <zip_file> [schema_dir]")
        print()
        print("Arguments:")
        print("  zip_file    Path to the zip archive containing XML files")
        print("  schema_dir  Path to directory containing JSON schema files (default: ./schema)")
        sys.exit(1)

    zip_path = Path(sys.argv[1])
    schema_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else Path("schema")

    if not zip_path.exists():
        print(f"Error: Zip file not found: {zip_path}", file=sys.stderr)
        sys.exit(1)

    if not schema_dir.exists():
        print(f"Error: Schema directory not found: {schema_dir}", file=sys.stderr)
        sys.exit(1)

    success = validate_zip(zip_path, schema_dir)

    if success:
        print("\n✅ Validation PASSED")
        sys.exit(0)
    else:
        print("\n❌ Validation FAILED")
        sys.exit(1)


if __name__ == "__main__":
    main()
