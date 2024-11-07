from .spec import Spec

import xml.parsers.expat as sax

START_ELEMENT = 0
END_ELEMENT = 1
CDATA = 2


class Parser:
    def __init__(self, spec: Spec):
        self.spec = spec
        self.columns = dict((name, []) for name in self.spec.fields.keys())
        self.current_column = None

    def start_root(self, event, data):
        if event == START_ELEMENT:
            if data == self.spec.root:
                return self.start_element_or_end_root
            else:
                raise Exception(
                    f"{self.filename}: Expected START_ELEMENT for {self.spec.root}, got {data}"
                )
        else:
            raise Exception(
                f"{self.filename}: Expected START_ELEMENT for {self.spec.root}, got {event}"
            )

    def start_element_or_end_root(self, event, data):
        if event == START_ELEMENT:
            if data == self.spec.element:
                for column in self.columns.values():
                    column.append(None)
                return self.start_attr_or_end_element
            else:
                raise Exception(
                    f"{self.filename}: Expected START_ELEMENT for {self.spec.element}, got {data}"
                )
        elif event == END_ELEMENT:
            if data == self.spec.root:
                return self.done
        else:
            raise Exception(f"{self.filename}: Expected START_ELEMENT, got {event}")

    def start_attr_or_end_element(self, event, data):
        if event == START_ELEMENT:
            if data in self.columns:
                self.current_column = data
                return self.attr_cdata_or_end_attr
            else:
                raise Exception(
                    f"{self.filename}: Element {data} not in {self.columns.keys()}"
                )
        elif event == END_ELEMENT:
            if data == self.spec.element:
                return self.start_element_or_end_root
            else:
                raise Exception(
                    f"{self.filename}: Expected END_ELEMENT for {self.spec.element}, got {data}"
                )
        else:
            raise Exception(f"{self.filename}: Got {event} for {data}")

    def attr_cdata_or_end_attr(self, event, data):
        if event == CDATA:
            current = self.columns[self.current_column][-1]
            current = "" if current is None else current
            self.columns[self.current_column][-1] = current + data
            return self.attr_cdata_or_end_attr
        elif event == END_ELEMENT:
            if data == self.current_column:
                convert = self.spec.fields[self.current_column].convert
                string_value = self.columns[self.current_column][-1]
                self.columns[self.current_column][-1] = convert(string_value)
                self.current_column = None
                return self.start_attr_or_end_element
            else:
                raise Exception(
                    f"{self.filename}: Expected END_ELEMENT for {self.current_column}, got {data}"
                )
        else:
            raise Exception(
                f"{self.filename}: Expected END_ELEMENT for {self.current_column}, got {event} for {data}"
            )

    def done(self, event=None, data=None):
        raise Exception(
            f"{self.filename}: Did not expect further events, but got {event} for {data} in {self.spec.root}"
        )

    def parse(self, f, filename):
        self.parser = self.start_root
        self.filename = filename

        def start_element_handler(name, _attrs):
            self.parser = self.parser(START_ELEMENT, name)

        def end_element_handler(name):
            self.parser = self.parser(END_ELEMENT, name)

        def cdata_handler(cdata):
            self.parser = self.parser(CDATA, cdata)

        p = sax.ParserCreate()
        p.StartElementHandler = start_element_handler
        p.EndElementHandler = end_element_handler
        p.CharacterDataHandler = cdata_handler

        p.ParseFile(f)
        return self.columns
