einheiten = {
    "EinheitBiomasse": "Biomasse",
    "EinheitGeothermieGrubengasDruckentspannung": "GeothermieGrubengasDruckentspannung",
    "EinheitKernkraft": "Kernkraft",
    "EinheitSolar": "Solar",
    "EinheitStromSpeicher": "StromSpeicher",
    "EinheitVerbrennung": "Verbrennung",
    "EinheitWasser": "Wasser",
    "EinheitWind": "Wind",
}

katalog_spalten = [
    "NetzbetreiberpruefungStatus",
    "Land",
    "Bundesland",
    "EinheitSystemstatus",
    "EinheitBetriebsstatus",
    "Energietraeger",
    "Einspeisungsart",
    "Hauptbrennstoff",
    # "Biomasseart",
    "Technologie",
    "ReserveartNachDemEnWG",
    # "Lage",
    # "Leistungsbegrenzung",
    # "Hauptausrichtung",
    # "HauptausrichtungNeigungswinkel",
    # "Nebenausrichtung",
    # "NebenausrichtungNeigungswinkel",
    # "Nutzungsbereich",
    "Einsatzort",
    # "AcDcKoppelung",
    # "Batterietechnologie",
    # "Pumpspeichertechnologie",
    # "EegAnlagentyp",
    "WeitererHauptbrennstoff",
    # "ArtDerWasserkraftanlage",
    # "ArtDesZuflusses",
    # "Seelage",
    # "ClusterOstsee",
    # "ClusterNordsee",
    # "Hersteller",
]

jede_einheit = "\nunion all by name\n".join(
    f"""select '{typ}' as Typ, * from {tabelle}""" for tabelle, typ in einheiten.items()
)
katalog_select = ",\n".join(
    f"""k_{spalte}.Wert as {spalte}""" for spalte in katalog_spalten
)
katalog_exclude = (
    "JedeEinheit.* exclude ("
    + ",\n".join(f"""{spalte}""" for spalte in katalog_spalten)
    + ")"
)
katalog_joins = "\n".join(
    f"""left join Katalogwert k_{spalte} on k_{spalte}.Id = JedeEinheit.{spalte}"""
    for spalte in katalog_spalten
)

with open("Einheit2.sql", "w") as f:
    f.write(
        f"""
create or replace view Einheit as
with JedeEinheit as ({jede_einheit})
select
{katalog_select},
{katalog_exclude}
from
JedeEinheit
{katalog_joins};
"""
    )
