This is a small command line tool to convert camt.052 from the Hamburger Sparkasse (Haspa) into csv.

# Requirements

- kscript >= 4.1.1
- Kotlin >= 1.8.0

# Run

## CSV

You can either run the script with a .zip containing xml files:

```shell-session
$ ./haspa-parser.kts ./camtPacket_1xxxxxxxxx_2018-01-01-2018-01-31.zip
Date;Valuta;Amount;Currency;Creditor;Debtor;Type;Description
2018-01-15;2018-01-15;-55.6;EUR;YYYYYYYY;Andreas Mausch Mausch;Lastschrift SEPA B2C;Blah blah blah
2018-01-15;2018-01-15;40;EUR;Andreas Mausch;Andreas Mausch;SEPA Gutschrift;Überweisung
2018-01-15;2018-01-15;-200;EUR;;Andreas Mausch;Barauszahlung GA;GA-NR00000000 BLZ20050550 1 15.01/13.37UHR Haspa0000/>H
```

..or with a single xml file:

```bash
./haspa-parser.kts ./camt_1xxxxxxxxx_15.01.2018.xml
```

You can also run it via `docker`, but this will run into the problem mentioned below.

```bash
docker run --rm -v $PWD:/opt/script:ro holgerbrandl/kscript:4.1.1 --silent /opt/script/haspa-parser.kts "/opt/script/*camt52Booked.ZIP" > /opt/script/output.csv
```

## ODS

You can also output the transactions as a .ods file for LibreCalc / Excel:

```bash
./haspa-parser.kts --output-format ods ./camt_1xxxxxxxxx_15.01.2018.xml > output.ods
```

### Debugging .ods

The ods libraries I've tried are flawed.

- [SODS](https://github.com/miachm/SODS) offers no way to customize date and currency formats.
- [fastods](https://github.com/jferard/fastods) is a bit buggy:
  Global float style does not work [#242](https://github.com/jferard/fastods/issues/242),
  and red negative floats are supported (there are [methods](https://github.com/jferard/fastods/blob/c8ed4ee9ba5abf190938c6506c87daf44ec016e1/fastods/src/main/java/com/github/jferard/fastods/datastyle/CurrencyStyleBuilder.java#L115) for it),
  but the `-neg` style [is not used by the cells](https://github.com/jferard/fastods/blob/c8ed4ee9ba5abf190938c6506c87daf44ec016e1/fastods/src/main/java/com/github/jferard/fastods/datastyle/NumberStyleHelper.java#L71-86).
- [odftoolkit](https://github.com/tdf/odftoolkit) has almost no documentation and some weird inheritance of styles, but seems to be the best bet, although it feels as complex as writing the XML directly.

```bash
unzip -p output.ods content.xml | xmllint --format -
unzip -p output.ods styles.xml | xmllint --format -
```

# Troubleshooting

> Caused by: java.lang.ClassNotFoundException: java.sql.SQLException

If you see this, it is [this issue](https://github.com/kscripting/kscript/issues/163).
This bug is **fixed in Kotlin 1.8.0** (see [this issue](https://youtrack.jetbrains.com/issue/KT-46312)).

For older Kotlin versions, there is an ugly workaround to run it via Java 8.
I've managed to do this on my MacBook Air M1:

```bash
brew install --cask homebrew/cask-versions/zulu8
JAVA_HOME=`/usr/libexec/java_home -v 1.8` ./haspa-parser.kts [...]
```

To build your own Docker image with the fixed Kotlin version, run this:

```bash
wget https://raw.githubusercontent.com/kscripting/kscript/eb691413c048238e58f6c5e5bc341180035f2a4c/misc/Dockerfile
docker build --build-arg KSCRIPT_VERSION=4.1.1 --build-arg KOTLIN_VERSION=1.8.0 --tag my-kscript .
```

# camt052 / camt053 File format specification

I am not exactly sure what this file format is, but I think it is used in Europe only.

See this [German Wikipedia article](https://de.wikipedia.org/wiki/Camt-Format).

I found some pdfs describing the file formats:

- [ebics.de](https://www.ebics.de/de/datenformate)
- [ing.nl](https://www.ing.nl/media/ING%20Format%20Description%20CAMT052%20CAMT053%20-%20NL%20v4.0_tcm162-110479.pdf)
