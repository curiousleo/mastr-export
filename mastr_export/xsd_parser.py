from typing import Any, Dict
from .spec import Field, Spec
import xml.etree.ElementTree as ET


def parse_xsd(xsd_file: str) -> Spec:
    """
    Parse an XSD file and create a Spec instance from it.

    Example structure:

    ```
    <?xml version="1.0" encoding="UTF-8"?>
    <xs:schema attributeFormDefault="unqualified" elementFormDefault="qualified" xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="EinheitenWind">
        <xs:complexType>
        <xs:sequence>
            <xs:element name="EinheitWind" maxOccurs="unbounded" minOccurs="0">
            <xs:complexType>
                <xs:sequence>
                <xs:element type="xs:string" name="EinheitMastrNummer"/>
                <xs:element type="xs:dateTime" name="DatumLetzteAktualisierung"/>
                <xs:element type="xs:string" name="LokationMaStRNummer" minOccurs="0"/>
                <xs:element name="NetzbetreiberpruefungStatus">
                    <xs:simpleType>
                    <xs:restriction base="xs:short">
                        <xs:enumeration value="2954"/>
                        <xs:enumeration value="2955"/>
                        <xs:enumeration value="3075"/>
                    </xs:restriction>
                    </xs:simpleType>
                </xs:element>
                <xs:element type="xs:date" name="NetzbetreiberpruefungDatum" minOccurs="0"/>
                ...
    ```

    Args:
        xsd_file_path: Path to the XSD file

    Returns:
        A Spec instance representing the structure defined in the XSD
    """
    # Define the XML namespace
    ns = {"xs": "http://www.w3.org/2001/XMLSchema"}
    tree = ET.parse(xsd_file)

    root_name = tree.find("./xs:element", ns).get("name")
    element_name = tree.find(
        "./xs:element/xs:complexType/xs:sequence/xs:element", ns
    ).get("name")

    sequence = tree.find(
        "./xs:element/xs:complexType/xs:sequence/xs:element/xs:complexType/xs:sequence",
        ns,
    )
    # E.g. for EinheitenGeothermieGrubengasDruckentspannung.xsd
    if sequence is None:
        sequence = tree.find(
            "./xs:element/xs:complexType/xs:sequence/xs:element/xs:complexType/xs:choice",
            ns,
        )

    # Extract field information
    fields = []
    for field_elem in sequence.findall("./xs:element", ns):
        field_name = field_elem.get("name")
        field_type = field_elem.get("type")
        #required = field_elem.get("minOccurs") != "0"

        # If the field has a simple type defined inline
        if field_type is None:
            simple_type = field_elem.find("./xs:simpleType/xs:restriction", ns)
            if simple_type is not None:
                # Check if it's a "0"/"1" restriction that should be treated as boolean
                enumerations = simple_type.findall("./xs:enumeration", ns)
                values = set(e.get("value") for e in enumerations)
                if values == set(("0", "1")):
                    field_type = "xs:boolean"
                else:
                    field_type = simple_type.get("base")

        field_type = field_type.removeprefix("xs:")

        # For now, we're not handling references or indexes
        fields.append({
            "name": field_name,
            #"required": required,
            "xsd": field_type,
            #"references": None,
        })

    assert len(fields) > 0
    return Spec(
        root=root_name,
        element=element_name,
        fields=fields,
        primary=None,  # No primary key info in the XSD
        without_rowid=False,  # Default value
    )
