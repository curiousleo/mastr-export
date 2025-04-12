from __future__ import annotations

from datetime import date, datetime
from typing import Iterator, Optional
import polars as pl
import os.path
import yaml

XSD_TO_POLARS = {
    # Date and time
    "date": pl.Date,
    "dateTime": pl.Datetime(time_unit="us", time_zone=None),
    # Float
    "float": pl.Float32,
    "double": pl.Float64,
    "decimal": pl.Float64,
    # Int
    "byte": pl.Int8,
    "short": pl.Int16,
    "int": pl.Int32,
    "nonNegativeInteger": pl.UInt64,
    # Other
    "boolean": pl.Boolean,
    "string": pl.Utf8,
}

XSD_TO_PYTHON = {
    # Date and time
    "date": date.fromisoformat,
    "dateTime": datetime.fromisoformat,
    # Float
    "float": float,
    "double": float,
    "decimal": float,
    # Int
    "byte": int,
    "short": int,
    "int": int,
    "nonNegativeInteger": int,
    # Other
    "boolean": bool,
    "string": lambda s: s,
}

XSD_TO_SQLITE = {
    "boolean": "integer",
    "date": "text",
    "dateTime": "text",
    "decimal": "real",
    "nonNegativeInteger": "integer",
    "string": "text",
}


XSD_TO_DUCKDB = {
    # Date and time
    "date": "date",
    "dateTime": "timestamp",
    # Float
    "float": "float",
    "double": "double",
    "decimal": "double",
    # Int
    "byte": "tinyint",
    "short": "smallint",
    "int": "integer",
    "nonNegativeInteger": "ubigint",
    # Other
    "boolean": "boolean",
    "string": "text",
}


class Reference:
    table: str
    column: str

    def __init__(self, table, column):
        self.table = table
        self.column = column

    def to_object(self):
        return {
            "table": self.table,
            "column": self.column,
        }

    def sqlite_schema(self):
        return f"""references "{self.table}"("{self.column}")"""


class Field:
    name: str
    index: bool
    xsd: str
    references: Optional[Reference]

    def __init__(self, name, index=False, xsd="string", references=None):
        self.name = name
        self.index = index
        self.xsd = xsd
        self.references = Reference(**references) if references is not None else None
        self.polars_type = XSD_TO_POLARS[self.xsd]
        self.python_type = XSD_TO_PYTHON[self.xsd]

    def to_object(self):
        object = {
            "name": self.name,
        }
        if self.xsd != "string":
            object["xsd"] = self.xsd
        if self.index:
            object["index"] = self.index
        if self.references is not None:
            object["references"] = self.references.to_object()
        return object

    def convert(self, s):
        return self.python_type(s) if s is not None else None

    def sqlite_schema(self):
        references = (
            f" {self.references.sqlite_schema()}" if self.references is not None else ""
        )
        return f""""{self.name}" {XSD_TO_SQLITE[self.xsd]}{references}"""

    def sqlite_index(self, element):
        return f"""create index if not exists idx_{element}_{self.name} on "{element}"("{self.name}");"""

    def duckdb_schema(self):
        return f""""{self.name}" {XSD_TO_DUCKDB[self.xsd]}"""


class Spec:
    root: str
    element: str
    without_rowid: bool
    primary: Optional[str]
    fields: dict[str, Field]

    def __init__(self, root, element, fields, primary=None, without_rowid=False):
        self.root = root
        self.element = element

        fields = [Field(**field) for field in fields]
        self.fields = dict((field.name, field) for field in fields)

        self.primary = primary
        self.without_rowid = without_rowid

    def to_object(self):
        object = {
            "root": self.root,
            "element": self.element,
            "fields": [field.to_object() for field in self.fields.values()],
        }
        if self.without_rowid:
            object["without_rowid"] = self.without_rowid
        if self.primary:
            object["primary"] = self.primary
        return object

    def sqlite_schema(self) -> str:
        columns = ",\n    ".join(
            field.sqlite_schema() for field in self.fields.values()
        )
        primary = (
            f""",
    primary key ("{self.primary}")"""
            if self.primary is not None
            else ""
        )
        return f"""create table if not exists "{self.element}" (
    {columns}{primary}
) strict{", without rowid" if self.without_rowid else ""};
"""

    def sqlite_indices(self) -> list[str]:
        return [
            field.sqlite_index(self.element)
            for field in self.fields.values()
            if field.index
        ]

    def duckdb_schema(self) -> str:
        columns = ",\n    ".join(
            field.duckdb_schema() for field in self.fields.values()
        )
        primary = (
            f""",
    primary key ("{self.primary}")"""
            if self.primary is not None
            else ""
        )
        return f"""create table if not exists "{self.element}" (
    {columns}{primary}
);
"""


class Specs:
    specs: list[Spec]

    def __init__(self, specs):
        self.specs = specs

    def __iter__(self) -> Iterator[Spec]:
        for d in self.specs:
            yield d

    @staticmethod
    def load(spec_file) -> Specs:
        spec_path = os.path.dirname(spec_file)
        spec_items = yaml.safe_load(open(spec_file))
        specs = [
            Spec(
                primary=descr.get("primary", None),
                **yaml.safe_load(open(os.path.join(spec_path, descr["spec"]))),
            )
            for descr in spec_items
        ]
        return Specs(specs)

    def save(self, spec_file):
        # Sanity check `spec_file`
        failed = False

        spec_items = yaml.safe_load(open(spec_file))
        spec_items = {spec["spec"]: spec.get("primary", None) for spec in spec_items}
        for spec in self.specs:
            f = f"{spec.root}.yaml"
            if f not in spec_items:
                print(f"Add {f} to {spec_file}")
                failed = True
                continue
            if spec_items[f] is not None and spec_items[f] not in spec.fields:
                print(
                    f"The primary key configured for {spec.element} is {spec_items[f]}, but it is not a field of the spec"
                )
                failed = True

        for spec in spec_items.keys():
            found = False
            for s in self.specs:
                if spec == f"{s.root}.yaml":
                    found = True
                    break
            if not found:
                print(f"Remove {spec} from {spec_file}")
                failed = True

        assert not failed

        # Actually save the specs
        spec_path = os.path.dirname(spec_file)
        os.makedirs(spec_path, exist_ok=True)
        p = lambda spec: os.path.join(spec_path, spec.root) + ".yaml"
        for spec in self.specs:
            yaml.safe_dump(spec.to_object(), open(p(spec), "w"), sort_keys=False)

    def for_file(self, filename) -> Spec:
        for descr in self.specs:
            if filename.startswith(descr.root):
                return descr
        raise Exception(f"No spec for {filename}")
