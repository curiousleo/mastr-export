Set up the testing environment.
  $ . "$TESTDIR"/setup.sh

Show the help message.
  $ mastr-export extract --help | sed -E 's/ +$//'
  Usage: extract [options]...
  Options:
        --zip-file ZIP_FILE     (input) Markstammdatenregister zip file
        --parquet-dir PARQUET_DIR
                                (output) Directory for Parquet files
        --help

Extract files.
  $ ZIP=Gesamtdatenexport_20251130_25.2.zip
  $ cp "$TESTDIR/$ZIP" ./work/
  $ mastr-export extract --zip-file "$ZIP" --parquet-dir ./data | sort
  Extracting files from Gesamtdatenexport_20251130_25.2.zip to ./data using schemas from /opt/mastr-export/schema ...
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data AnlagenEegBiomasse.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data AnlagenEegGeothermieGrubengasDruckentspannung.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data AnlagenEegSolar_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data AnlagenEegSpeicher_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data AnlagenEegWasser.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data AnlagenEegWind.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data AnlagenGasSpeicher.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data AnlagenKwk.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data AnlagenStromSpeicher_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Bilanzierungsgebiete.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenAenderungNetzbetreiberzuordnungen_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenBiomasse.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenGasErzeuger.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenGasSpeicher.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenGasverbraucher.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenGenehmigung.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenGeothermieGrubengasDruckentspannung.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenKernkraft.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenSolar_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenStromSpeicher_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenStromVerbraucher.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenVerbrennung.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenWasser.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data EinheitenWind.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Einheitentypen.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Ertuechtigungen.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data GeloeschteUndDeaktivierteEinheiten_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data GeloeschteUndDeaktivierteMarktakteure_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Katalogkategorien.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Katalogwerte.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Lokationen_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Lokationstypen.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data MarktakteureUndRollen.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Marktakteure_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Marktfunktionen.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Marktrollen.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Netzanschlusspunkte_1.xml
  process_xml_file Gesamtdatenexport_20251130_25.2.zip /opt/mastr-export/schema ./data Netze.xml

Show parquet files.
  $ ls ./work/data
  AnlagenEegBiomasse.parquet
  AnlagenEegGeothermieGrubengasDruckentspannung.parquet
  AnlagenEegSolar_1.parquet
  AnlagenEegSpeicher_1.parquet
  AnlagenEegWasser.parquet
  AnlagenEegWind.parquet
  AnlagenGasSpeicher.parquet
  AnlagenKwk.parquet
  AnlagenStromSpeicher_1.parquet
  Bilanzierungsgebiete.parquet
  EinheitenAenderungNetzbetreiberzuordnungen_1.parquet
  EinheitenBiomasse.parquet
  EinheitenGasErzeuger.parquet
  EinheitenGasSpeicher.parquet
  EinheitenGasverbraucher.parquet
  EinheitenGenehmigung.parquet
  EinheitenGeothermieGrubengasDruckentspannung.parquet
  EinheitenKernkraft.parquet
  EinheitenSolar_1.parquet
  EinheitenStromSpeicher_1.parquet
  EinheitenStromVerbraucher.parquet
  EinheitenVerbrennung.parquet
  EinheitenWasser.parquet
  EinheitenWind.parquet
  Einheitentypen.parquet
  Ertuechtigungen.parquet
  GeloeschteUndDeaktivierteEinheiten_1.parquet
  GeloeschteUndDeaktivierteMarktakteure_1.parquet
  Katalogkategorien.parquet
  Katalogwerte.parquet
  Lokationen_1.parquet
  Lokationstypen.parquet
  MarktakteureUndRollen.parquet
  Marktakteure_1.parquet
  Marktfunktionen.parquet
  Marktrollen.parquet
  Netzanschlusspunkte_1.parquet
  Netze.parquet
