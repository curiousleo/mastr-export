import certifi
import re
import ssl
import urllib.request


def print_export_url():
    context = ssl.create_default_context(
        ssl.Purpose.SERVER_AUTH, cafile=certifi.where()
    )
    with urllib.request.urlopen(
        "https://www.marktstammdatenregister.de/MaStR/Datendownload", context=context
    ) as f:
        html = f.read().decode("utf-8")
        match = re.search(
            r"(https://download\.marktstammdatenregister\.de/Gesamtdatenexport_[0-9_.]+\.zip)",
            html,
        )
        return match.group()


if __name__ == "__main__":
    print(print_export_url())
