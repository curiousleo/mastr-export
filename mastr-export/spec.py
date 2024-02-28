from datetime import date, datetime
import polars as pl
import os.path
import yaml

XSD_TO_POLARS = {
    "boolean": pl.Boolean,
    "date": pl.Date,
    "dateTime": pl.Datetime(time_unit="us", time_zone=None),
    "decimal": pl.Float64,
    "nonNegativeInteger": pl.UInt64,
    "string": pl.Utf8,
}

XSD_TO_PYTHON = {
    "boolean": bool,
    "date": date.fromisoformat,
    "dateTime": datetime.fromisoformat,
    "decimal": float,
    "nonNegativeInteger": int,
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
    "boolean": "boolean",
    "date": "date",
    "dateTime": "timestamp",
    "decimal": "double",
    "nonNegativeInteger": "bigint",
    "string": "text",
}


class Reference:
    table: str
    column: str

    def __init__(self, table, column):
        self.table = table
        self.column = column

    def sqlite_schema(self):
        return f"""references "{self.table}"("{self.column}")"""


class Field:
    name: str
    index: bool
    xsd: str
    references: Reference

    def __init__(self, name, index=None, xsd="string", references=None):
        self.name = name
        self.index = index
        self.xsd = xsd
        self.references = Reference(**references) if references is not None else None
        self.polars_type = XSD_TO_POLARS[self.xsd]
        self.python_type = XSD_TO_PYTHON[self.xsd]

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
    primary: str
    without_rowid: bool
    fields: dict[str, Field]

    def __init__(self, root, element, primary, fields, without_rowid=False):
        self.root = root
        self.element = element
        self.primary = primary
        self.without_rowid = without_rowid
        fields = [Field(**field) for field in fields]
        self.fields = dict((field.name, field) for field in fields)

    def sqlite_schema(self):
        columns = ",\n    ".join(
            field.sqlite_schema() for field in self.fields.values()
        )
        return f"""create table if not exists "{self.element}" (
    {columns},
    primary key ("{self.primary}")
) {" without rowid" if self.without_rowid else ""};
"""

    def sqlite_indices(self):
        return [
            field.sqlite_index(self.element)
            for field in self.fields.values()
            if field.index
        ]

    def duckdb_schema(self):
        columns = ",\n    ".join(
            field.duckdb_schema() for field in self.fields.values()
        )
        return f"""create table if not exists "{self.element}" (
    {columns},
    primary key ("{self.primary}")
);
"""


class Specs:
    specs: list[Spec]

    def __init__(self, specs):
        self.specs = specs

    def __iter__(self):
        for d in self.specs:
            yield d

    @staticmethod
    def load(spec_file):
        spec_path = os.path.dirname(spec_file)
        specs = [
            Spec(**yaml.safe_load(open(os.path.join(spec_path, descr))))
            for descr in yaml.safe_load(open(spec_file))
        ]
        return Specs(specs)

    def for_file(self, filename):
        for descr in self.specs:
            if filename.startswith(descr.root):
                return descr
        raise Exception(f"No spec for {filename}")
