#!/usr/bin/env kscript

@file:DependsOn("org.javamoney:moneta:pom:1.4.2")
@file:DependsOn("org.apache.commons:commons-csv:1.5")
@file:DependsOn("commons-io:commons-io:2.11.0")
@file:DependsOn("org.apache.commons:commons-lang3:3.12.0")
@file:DependsOn("org.apache.tika:tika-core:2.6.0")
@file:DependsOn("org.slf4j:slf4j-nop:2.0.6")
@file:DependsOn("com.github.ajalt.clikt:clikt-jvm:3.5.0")
// fastods latest version (0.8.1) has a bug formatting floats:
// https://github.com/jferard/fastods/issues/242
@file:DependsOn("com.github.jferard:fastods:0.7.3")

import org.javamoney.moneta.Money
import org.w3c.dom.Element
import org.w3c.dom.Node
import org.w3c.dom.NodeList
import org.apache.commons.csv.CSVFormat.*
import org.apache.commons.csv.CSVPrinter
import org.apache.commons.io.input.CloseShieldInputStream
import org.apache.commons.lang3.StringUtils.normalizeSpace
import org.apache.tika.Tika
import java.io.InputStream
import java.io.FileInputStream
import java.io.OutputStream
import java.io.OutputStreamWriter
import java.util.logging.Level.*
import java.util.logging.LogManager.*
import java.util.logging.Logger.getLogger
import java.util.Locale.*
import java.util.zip.ZipInputStream
import java.nio.file.Files.*
import java.nio.file.Path
import javax.xml.parsers.DocumentBuilderFactory
import javax.xml.xpath.XPathConstants.NODE
import javax.xml.xpath.XPathConstants.NODESET
import javax.xml.xpath.XPathFactory
import java.text.DecimalFormat
import java.text.DecimalFormatSymbols
import java.time.LocalDate
import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.arguments.unique
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.enum
import com.github.ajalt.clikt.parameters.types.file
import com.github.jferard.fastods.OdsFactory
import com.github.jferard.fastods.attribute.SimpleLength
import com.github.jferard.fastods.datastyle.DataStylesBuilder
import com.github.jferard.fastods.datastyle.FloatStyleBuilder
import com.github.jferard.fastods.style.LOFonts
import com.github.jferard.fastods.style.TableCellStyle
import com.github.jferard.fastods.style.TableColumnStyle

fun NodeList.asList(): List<Node> {
    val nodes = mutableListOf<Node>()
    repeat(length) { index ->
        nodes.add(item(index))
    }
    return nodes
}

data class Party(val name: String, val iban: String)
data class Transaction(val date: LocalDate, val valuta: LocalDate, val amount: Money, val creditor: Party, val debtor: Party, val type: String, val description: String)

fun String.normalizeSpace(): String = normalizeSpace(this)

class Camt052File(inputStream: InputStream) {

    val document = DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(inputStream)
    val xpath = XPathFactory.newInstance().newXPath()

    fun parse(): List<Transaction> {
        /*
            camt looks like a great file format to me. not.
            It has a lot of unreadable, cryptically shortened names.
    
            https://www.bayernlb.de/internet/media/de/ir/downloads_1/zahlungsverkehr/formate_1/camt05X.pdf
    
            BkToCstmrAcctRpt: Bank-to-Customer Account Report message
            Rpt: Report
            Ntry: Entry (wow)
            Amt: Amount
        */

        val entries = xpath.evaluate("/Document/BkToCstmrAcctRpt/Rpt/Ntry", document, NODESET) as NodeList

        return entries.asList().map { node ->
            val entry = node as Element

            fun element(key: String): Element? {
                return xpath.evaluate(key, entry, NODE) as Element?
            }

            val debit = element("CdtDbtInd")!!.textContent == "DBIT"

            val amountElement = element("Amt")!!
            val amount = amountElement.textContent.toBigDecimal()
            val currency = amountElement.getAttribute("Ccy")
            val money = Money.of(if (debit) amount.negate() else amount, currency)

            val creditor = element("NtryDtls/TxDtls/RltdPties/Cdtr/Nm")?.textContent?.normalizeSpace() ?: ""
            val creditorIban = element("NtryDtls/TxDtls/RltdPties/CdtrAcct/Id/IBAN")?.textContent?.normalizeSpace() ?: ""
            val debtor = element("NtryDtls/TxDtls/RltdPties/Dbtr/Nm")?.textContent?.normalizeSpace() ?: ""
            val debtorIban = element("NtryDtls/TxDtls/RltdPties/DbtrAcct/Id/IBAN")?.textContent?.normalizeSpace() ?: ""

            val date = LocalDate.parse(element("BookgDt/Dt")!!.textContent)
            val valuta = LocalDate.parse(element("ValDt/Dt")!!.textContent)

            val type = element("AddtlNtryInf")?.textContent?.normalizeSpace() ?: ""
            val texts = (xpath.evaluate("NtryDtls/TxDtls/RmtInf/Ustrd", entry, NODESET) as NodeList).asList().map { it.textContent.normalizeSpace() }

            Transaction(date, valuta, money, Party(creditor, creditorIban), Party(debtor, debtorIban), type, texts.getOrElse(0, { "" }))
        }
    }
}

getLogManager().getLogger("").setLevel(WARNING)

fun isZip(path: Path): Boolean = Tika().detect(path) == "application/zip"

enum class OutputFormat {
    CSV {
        override fun print(transactions: List<Transaction>, stream: OutputStream) {
            val format = DEFAULT.withDelimiter(';').withHeader("Date", "Valuta", "Amount", "Currency", "Creditor", "Creditor IBAN", "Debtor", "Debtor IBAN", "Type", "Description")
            val printer = CSVPrinter(OutputStreamWriter(stream, "UTF-8"), format)
            transactions.forEach {
                printer.printRecord(it.date, it.valuta, DecimalFormat("#.##", DecimalFormatSymbols(US)).format(it.amount.number), it.amount.currency, it.creditor.name, it.creditor.iban, it.debtor.name, it.debtor.iban, it.type, it.description)
                printer.flush()
            }
        }
    },
    ODS {
        override fun print(transactions: List<Transaction>, stream: OutputStream) {
            val headers = listOf("Date", "Valuta", "Amount", "Currency", "Creditor", "Creditor IBAN", "Debtor", "Debtor IBAN", "Type", "Description")

            val dataStylesBuilder = DataStylesBuilder.create(GERMANY)
            dataStylesBuilder.floatStyleBuilder().decimalPlaces(2).groupThousands(true)

            val writer = OdsFactory.builder(getLogger("ods"), GERMANY).dataStyles(dataStylesBuilder.build()).build().createWriter()
            val document = writer.document()
            val sheet = document.addTable("MySheet")

            val columnDataStyle =
                TableColumnStyle.builder("col-datastyle")
                .optimalWidth()
                .build();
            sheet.setColumnStyle(0, columnDataStyle);

            val walker = sheet.getWalker()
            headers.forEach {
                walker.setStringValue(it)
                walker.next()
            }
            walker.nextRow()

            walker.setFloatValue(30000.0)
            walker.next()
            walker.setFloatValue(123456.789)

            transactions.forEach {
                // printer.printRecord(it.date, it.valuta, DecimalFormat("#.##", DecimalFormatSymbols(US)).format(it.amount.number), it.amount.currency, it.creditor.name, it.creditor.iban, it.debtor.name, it.debtor.iban, it.type, it.description)
            }

            writer.save(stream)
        }
    };

    abstract fun print(transactions: List<Transaction>, stream: OutputStream)
}

class HaspaParser : CliktCommand() {
    private val files by argument("files").file(mustExist = true).multiple(required = true).unique()
    private val outputFormat: OutputFormat by option().enum<OutputFormat>().default(OutputFormat.CSV)

    override fun run() {
        val transactions = files
            .flatMap { file ->
                if (isZip(file.toPath())) {
                    ZipInputStream(FileInputStream(file)).use { zip ->
                        generateSequence { zip.nextEntry }
                            .filterNot { it.isDirectory }
                            .flatMap { Camt052File(CloseShieldInputStream.wrap(zip)).parse() }
                            .toList()
                    }
                } else {
                    FileInputStream(file).use {
                        Camt052File(CloseShieldInputStream.wrap(it)).parse()
                    }
                }
            }
            .sortedBy { it.date }

        outputFormat.print(transactions, System.out)
    }
}

HaspaParser().main(args)
