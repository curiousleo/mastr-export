create or replace view Einheit as
with JedeEinheit as (
    select 'Biomasse' as Typ, * from EinheitBiomasse
    union all by name
    select 'GeothermieGrubengasDruckentspannung' as Typ, * from EinheitGeothermieGrubengasDruckentspannung
    union all by name
    select 'Kernkraft' as Typ, * from EinheitKernkraft
    union all by name
    select 'Solar' as Typ, * from EinheitSolar
    union all by name
    select 'StromSpeicher' as Typ, * from EinheitStromSpeicher
    union all by name
    select 'Verbrennung' as Typ, * from EinheitVerbrennung
    union all by name
    select 'Wasser' as Typ, * from EinheitWasser
    union all by name
    select 'Wind' as Typ, * from EinheitWind
)
select
    (select Wert from Katalogwert where Id = NetzbetreiberpruefungStatus) as NetzbetreiberpruefungStatus,
    (select Wert from Katalogwert where Id = Land) as Land,
    (select Wert from Katalogwert where Id = Bundesland) as Bundesland,
    (select Wert from Katalogwert where Id = EinheitSystemstatus) as EinheitSystemstatus,
    (select Wert from Katalogwert where Id = EinheitBetriebsstatus) as EinheitBetriebsstatus,
    (select Wert from Katalogwert where Id = Energietraeger) as Energietraeger,
    (select Wert from Katalogwert where Id = Einspeisungsart) as Einspeisungsart,
    (select Wert from Katalogwert where Id = Hauptbrennstoff) as Hauptbrennstoff,
    (select Wert from Katalogwert where Id = Biomasseart) as Biomasseart,
    (select Wert from Katalogwert where Id = Technologie) as Technologie,
    (select Wert from Katalogwert where Id = ReserveartNachDemEnWG) as ReserveartNachDemEnWG,
    (select Wert from Katalogwert where Id = AnzahlModule) as AnzahlModule,
    (select Wert from Katalogwert where Id = Lage) as Lage,
    (select Wert from Katalogwert where Id = Leistungsbegrenzung) as Leistungsbegrenzung,
    (select Wert from Katalogwert where Id = Hauptausrichtung) as Hauptausrichtung,
    (select Wert from Katalogwert where Id = HauptausrichtungNeigungswinkel) as HauptausrichtungNeigungswinkel,
    (select Wert from Katalogwert where Id = Nebenausrichtung) as Nebenausrichtung,
    (select Wert from Katalogwert where Id = NebenausrichtungNeigungswinkel) as NebenausrichtungNeigungswinkel,
    (select Wert from Katalogwert where Id = Nutzungsbereich) as Nutzungsbereich,
    (select Wert from Katalogwert where Id = Einsatzort) as Einsatzort,
    (select Wert from Katalogwert where Id = AcDcKoppelung) as AcDcKoppelung,
    (select Wert from Katalogwert where Id = Batterietechnologie) as Batterietechnologie,
    (select Wert from Katalogwert where Id = Pumpspeichertechnologie) as Pumpspeichertechnologie,
    (select Wert from Katalogwert where Id = EegAnlagentyp) as EegAnlagentyp,
    (select Wert from Katalogwert where Id = WeitererHauptbrennstoff) as WeitererHauptbrennstoff,
    (select Wert from Katalogwert where Id = ArtDerWasserkraftanlage) as ArtDerWasserkraftanlage,
    (select Wert from Katalogwert where Id = ArtDesZuflusses) as ArtDesZuflusses,
    (select Wert from Katalogwert where Id = Seelage) as Seelage,
    (select Wert from Katalogwert where Id = ClusterOstsee) as ClusterOstsee,
    (select Wert from Katalogwert where Id = ClusterNordsee) as ClusterNordsee,
    (select Wert from Katalogwert where Id = Hersteller) as Hersteller,
    * exclude (
         NetzbetreiberpruefungStatus
       , Land
       , Bundesland
       , EinheitSystemstatus
       , EinheitBetriebsstatus
       , Energietraeger
       , Einspeisungsart
       , Hauptbrennstoff
       , Biomasseart
       , Technologie
       , ReserveartNachDemEnWG
       , AnzahlModule
       , Lage
       , Leistungsbegrenzung
       , Hauptausrichtung
       , HauptausrichtungNeigungswinkel
       , Nebenausrichtung
       , NebenausrichtungNeigungswinkel
       , Nutzungsbereich
       , Einsatzort
       , AcDcKoppelung
       , Batterietechnologie
       , Pumpspeichertechnologie
       , EegAnlagentyp
       , WeitererHauptbrennstoff
       , ArtDerWasserkraftanlage
       , ArtDesZuflusses
       , Seelage
       , ClusterOstsee
       , ClusterNordsee
       , Hersteller)
from JedeEinheit;
