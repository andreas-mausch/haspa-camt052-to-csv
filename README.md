This is a small command line tool to convert camt.052 from the Hamburger Sparkasse (Haspa) into csv.

# Requirements

kscript (4.1.1) must be installed.

# Run

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
docker run -it --rm -v $PWD:/opt/script:ro holgerbrandl/kscript:4.1.1 /opt/script/haspa-parser.kts "/opt/script/*camt52Booked.ZIP" > /opt/script/output.csv
```

# Troubleshooting

> Caused by: java.lang.ClassNotFoundException: java.sql.SQLException

If you see this, it is [this issue](https://github.com/kscripting/kscript/issues/163).

I've managed to do this on my MacBook Air M1 (ugly!):

```bash
brew install --cask homebrew/cask-versions/zulu8
JAVA_HOME=`/usr/libexec/java_home -v 1.8` ./haspa-parser.kts [...]
```
