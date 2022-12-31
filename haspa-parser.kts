#!/usr/bin/env kscript

@file:DependsOn("org.javamoney:moneta:pom:1.4.2")
@file:DependsOn("org.apache.commons:commons-csv:1.5")
@file:DependsOn("commons-io:commons-io:2.11.0")
@file:DependsOn("org.apache.commons:commons-lang3:3.12.0")
@file:DependsOn("org.apache.tika:tika-core:2.6.0")
@file:DependsOn("org.slf4j:slf4j-nop:2.0.6")
@file:DependsOn("com.github.ajalt.clikt:clikt-jvm:3.5.0")
@file:DependsOn("org.odftoolkit:odfdom-java:0.11.0")

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.arguments.unique
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.enum
import com.github.ajalt.clikt.parameters.types.file
import org.apache.commons.csv.CSVFormat.*
import org.apache.commons.csv.CSVPrinter
import org.apache.commons.io.input.CloseShieldInputStream
import org.apache.commons.lang3.StringUtils.normalizeSpace
import org.apache.tika.Tika
import org.javamoney.moneta.Money
import org.odftoolkit.odfdom.doc.OdfSpreadsheetDocument.newSpreadsheetDocument
import org.odftoolkit.odfdom.doc.table.OdfTableCell
import org.odftoolkit.odfdom.doc.table.OdfTableRow
import org.odftoolkit.odfdom.dom.OdfDocumentNamespace.TABLE
import org.odftoolkit.odfdom.dom.style.OdfStyleFamily.TableCell
import org.odftoolkit.odfdom.dom.style.props.OdfTextProperties.*
import org.odftoolkit.odfdom.incubator.doc.style.OdfStyle
import org.odftoolkit.odfdom.pkg.OdfName.newName
import org.w3c.dom.Element
import org.w3c.dom.Node
import org.w3c.dom.NodeList
import java.io.FileInputStream
import java.io.InputStream
import java.io.OutputStream
import java.io.OutputStreamWriter
import java.nio.file.Files.*
import java.nio.file.Path
import java.text.DecimalFormat
import java.text.DecimalFormatSymbols
import java.time.LocalDate
import java.util.*
import java.util.Locale.*
import java.util.logging.Level.*
import java.util.logging.LogManager.*
import java.util.zip.ZipInputStream
import javax.xml.parsers.DocumentBuilderFactory
import javax.xml.xpath.XPathConstants.NODE
import javax.xml.xpath.XPathConstants.NODESET
import javax.xml.xpath.XPathFactory


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

getLogManager().getLogger("").level = WARNING

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

            val document = newSpreadsheetDocument()
            val sheet = document.spreadsheetTables.first()
            sheet.tableName = "MySheet"

            val styles = document.contentDom.orCreateAutomaticStyles

            val defaultStyle = styles.getStyle("Default", TableCell)

            val headingStyle = styles.newStyle(TableCell)
            headingStyle.setFontWeight("bold")

            val headRow = sheet.getRowByIndex(0)
            headRow.defaultCellStyle = headingStyle

            headers.forEachIndexed { index, header ->
                val cell = headRow.getCellByIndex(index)
                cell.stringValue = header
                cell.setStyle(headingStyle)
            }

            transactions.take(5).forEach {
                val row = sheet.appendRow()
                row.withCell(0) { stringValue = it.date.toString() }
                row.withCell(1) { stringValue = it.valuta.toString() }
                row.withCell(2) { stringValue = it.amount.toString() }
                row.withCell(3) { stringValue = it.amount.currency.toString() }
                row.withCell(4) { stringValue = it.creditor.name }
                row.withCell(5) { stringValue = it.creditor.iban }
                row.withCell(6) { stringValue = it.debtor.name }
                row.withCell(7) { stringValue = it.debtor.iban }
                row.withCell(8) { stringValue = it.type }
                row.withCell(9) { stringValue = it.description }
            }

            document.save(stream)
        }

        private fun OdfStyle.setFontWeight(value: String) {
            setProperty(FontWeight, value)
            setProperty(FontWeightAsian, value)
            setProperty(FontWeightComplex, value)
        }

        private fun OdfTableCell.setStyle(style: OdfStyle?) {
            when (style) {
                null -> {
                    // For some reason, the style from the previous row is automatically applied to a cell.
                    // Therefore, we reset the set style.
                    val name = newName(TABLE, "style-name")
                    odfElement.removeAttributeNS(name.uri, name.localName)
                    return
                }
                else -> odfElement.styleName = style.styleNameAttribute
            }
        }

        private fun <R> OdfTableRow.withCell(index: Int, style: OdfStyle? = null, block: OdfTableCell.() -> R): R =
            with(getCellByIndex(index)) {
                setStyle(style)
                block()
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
