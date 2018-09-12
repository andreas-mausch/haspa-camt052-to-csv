This is a small command line tool to convert camt.052 from the Hamburger Sparkasse (Haspa) into csv.

```
$ kscript haspa-parser.kts ./camtPacket_1xxxxxxxxx_2018-01-01-2018-01-31.zip

Date;Valuta;Amount;Currency;Creditor;Debtor;Type;Description
2018-01-15;2018-01-15;-55.6;EUR;YYYYYYYY;Andreas Mausch Mausch;Lastschrift SEPA B2C;Blah blah blah
2018-01-15;2018-01-15;40;EUR;Andreas Mausch;Andreas Mausch;SEPA Gutschrift;Überweisung
2018-01-15;2018-01-15;-200;EUR;;Andreas Mausch;Barauszahlung GA;GA-NR00000000 BLZ20050550 1 15.01/13.37UHR Haspa0000/>H
```

You can also pass a xml file directly.

```
$ kscript haspa-parser.kts ./camt_1xxxxxxxxx_15.01.2018.xml
```
