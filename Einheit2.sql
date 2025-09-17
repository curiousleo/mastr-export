select
k_NetzbetreiberpruefungStatus.Wert as NetzbetreiberpruefungStatus,
k_Land.Wert as Land,
k_Bundesland.Wert as Bundesland,
k_EinheitSystemstatus.Wert as EinheitSystemstatus,
k_EinheitBetriebsstatus.Wert as EinheitBetriebsstatus,
k_Energietraeger.Wert as Energietraeger,
k_Einspeisungsart.Wert as Einspeisungsart,
k_Hauptbrennstoff.Wert as Hauptbrennstoff,
k_Technologie.Wert as Technologie,
k_ReserveartNachDemEnWG.Wert as ReserveartNachDemEnWG,
k_Einsatzort.Wert as Einsatzort,
k_WeitererHauptbrennstoff.Wert as WeitererHauptbrennstoff,
e.* exclude (NetzbetreiberpruefungStatus,
Land,
Bundesland,
EinheitSystemstatus,
EinheitBetriebsstatus,
Energietraeger,
Einspeisungsart,
Hauptbrennstoff,
Technologie,
ReserveartNachDemEnWG,
Einsatzort,
WeitererHauptbrennstoff)
from
EinheitVerbrennung e
left join Katalogwert k_NetzbetreiberpruefungStatus on k_NetzbetreiberpruefungStatus.Id = e.NetzbetreiberpruefungStatus
left join Katalogwert k_Land on k_Land.Id = e.Land
left join Katalogwert k_Bundesland on k_Bundesland.Id = e.Bundesland
left join Katalogwert k_EinheitSystemstatus on k_EinheitSystemstatus.Id = e.EinheitSystemstatus
left join Katalogwert k_EinheitBetriebsstatus on k_EinheitBetriebsstatus.Id = e.EinheitBetriebsstatus
left join Katalogwert k_Energietraeger on k_Energietraeger.Id = e.Energietraeger
left join Katalogwert k_Einspeisungsart on k_Einspeisungsart.Id = e.Einspeisungsart
left join Katalogwert k_Hauptbrennstoff on k_Hauptbrennstoff.Id = e.Hauptbrennstoff
left join Katalogwert k_Technologie on k_Technologie.Id = e.Technologie
left join Katalogwert k_ReserveartNachDemEnWG on k_ReserveartNachDemEnWG.Id = e.ReserveartNachDemEnWG
left join Katalogwert k_Einsatzort on k_Einsatzort.Id = e.Einsatzort
left join Katalogwert k_WeitererHauptbrennstoff on k_WeitererHauptbrennstoff.Id = e.WeitererHauptbrennstoff
