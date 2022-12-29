#!/usr/bin/env kscript

@file:DependsOn("org.javamoney:moneta:pom:1.4.2")
@file:DependsOn("org.apache.commons:commons-csv:1.5")
@file:DependsOn("commons-io:commons-io:2.6")
@file:DependsOn("org.apache.commons:commons-lang3:3.12.0")
@file:DependsOn("org.apache.tika:tika-core:2.6.0")
@file:DependsOn("org.slf4j:slf4j-nop:2.0.6")
@file:DependsOn("com.github.ajalt.clikt:clikt-jvm:3.5.0")

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
import java.io.OutputStreamWriter
import java.util.logging.Level.*
import java.util.logging.LogManager.*
import java.util.Locale.*
import java.util.zip.ZipInputStream
import java.nio.file.Files.*
import java.nio.file.Path
import java.nio.file.Paths
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
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.file


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

class HaspaParser : CliktCommand() {
    private val files by argument().file(mustExist = true).multiple(required=true).unique()

    override fun run() {
        val transactions = mutableListOf<Transaction>()

        for (file in files) {
            if (isZip(file.toPath())) {
                ZipInputStream(FileInputStream(file)).use { zip ->
                    var entry = zip.nextEntry
                    while (entry != null) {
                        val fileTransactions = Camt052File(CloseShieldInputStream(zip)).parse()
                        transactions.addAll(fileTransactions)
                        entry = zip.nextEntry
                    }
                }
            } else {
                FileInputStream(file).use { xml ->
                    val fileTransactions = Camt052File(CloseShieldInputStream(xml)).parse()
                    transactions.addAll(fileTransactions)
                }
            }
        }

        transactions.sortBy { it.date }

        val format = DEFAULT.withDelimiter(';').withHeader("Date", "Valuta", "Amount", "Currency", "Creditor", "Creditor IBAN", "Debtor", "Debtor IBAN", "Type", "Description")
        val printer = CSVPrinter(OutputStreamWriter(System.out, "UTF-8"), format)
        transactions.forEach {
            printer.printRecord(it.date, it.valuta, DecimalFormat("#.##", DecimalFormatSymbols(US)).format(it.amount.number), it.amount.currency, it.creditor.name, it.creditor.iban, it.debtor.name, it.debtor.iban, it.type, it.description)
            printer.flush()
        }
    }
}

HaspaParser().main(args)
