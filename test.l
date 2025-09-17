@Engine("duckdb");
@AttachDatabase("mastr", "20250223.duckdb1");

SolarEinheitInOrt(id:, ort:) :-
  mastr.EinheitSolar(EinheitMastrNummer:id, Ort:ort);

InGaggenau(id:) :-
  SolarEinheitInOrt(id:, ort:"Gaggenau");

Nettonennleistung(id:, leistung:) :-
  mastr.EinheitBiomasse(EinheitMastrNummer:id, NettoNennleistung:leistung) |
  mastr.EinheitGeothermieGrubengasDruckentspannung(EinheitMastrNummer:id, NettoNennleistung:leistung) |
  mastr.EinheitKernkraft(EinheitMastrNummer:id, NettoNennleistung:leistung) |
  mastr.EinheitSolar(EinheitMastrNummer:id, NettoNennleistung:leistung) |
  mastr.EinheitStromSpeicher(EinheitMastrNummer:id, NettoNennleistung:leistung) |
  mastr.EinheitVerbrennung(EinheitMastrNummer:id, NettoNennleistung:leistung) |
  mastr.EinheitWasser(EinheitMastrNummer:id, NettoNennleistung:leistung) |
  mastr.EinheitWind(EinheitMastrNummer:id, NettoNennleistung:leistung);

L(leistung:) :- Nettonennleistung(id:"SEE927519373619", leistung:);

# EinheitenInLokation(lokation:, einheit:) :-
#   mastr.Lokation(MastrNummer:lokation, VerknuepfteEinheitenMaStRNummern:einheiten),
#   einheit in Split(einheiten, ", ");

EinheitenInLokation(lokation:, einheit:) :-
  mastr.EinheitBiomasse(EinheitMastrNummer:einheit, LokationMaStRNummer:lokation) |
  mastr.EinheitGeothermieGrubengasDruckentspannung(EinheitMastrNummer:einheit, LokationMaStRNummer:lokation) |
  mastr.EinheitKernkraft(EinheitMastrNummer:einheit, LokationMaStRNummer:lokation) |
  mastr.EinheitSolar(EinheitMastrNummer:einheit, LokationMaStRNummer:lokation) |
  mastr.EinheitStromSpeicher(EinheitMastrNummer:einheit, LokationMaStRNummer:lokation) |
  mastr.EinheitVerbrennung(EinheitMastrNummer:einheit, LokationMaStRNummer:lokation) |
  mastr.EinheitWasser(EinheitMastrNummer:einheit, LokationMaStRNummer:lokation) |
  mastr.EinheitWind(EinheitMastrNummer:einheit, LokationMaStRNummer:lokation);

Kategorie(id, kategorie) :-
  mastr.Katalogwert(Id:id, Wert:kategorie);

@Limit(E, 10);
@OrderBy(E, "leistung desc");
E(einheit:, leistung:) :-
  EinheitenInLokation(lokation:"SEL983901223443", einheit:),
  Nettonennleistung(id:einheit, leistung:);

@Limit(G, 10);
@OrderBy(G, "leistung desc");
G(einheit:, leistung:) :-
  mastr.EinheitVerbrennung(EinheitMastrNummer:einheit, WeitererHauptbrennstoff:brennstoff, NettoNennleistung:leistung),
  Kategorie(brennstoff, "Biomethan (Bioerdgas)");
