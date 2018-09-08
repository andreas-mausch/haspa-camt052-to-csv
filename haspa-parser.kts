#!/usr/bin/env kscript

@file:DependsOn("org.javamoney:moneta:1.3@pom")
@file:DependsOn("org.apache.commons:commons-csv:1.5")
@file:DependsOn("commons-io:commons-io:2.6")

import org.javamoney.moneta.Money
import org.w3c.dom.Element
import org.w3c.dom.Node
import org.w3c.dom.NodeList
import org.apache.commons.csv.CSVFormat.*
import org.apache.commons.csv.CSVPrinter
import org.apache.commons.io.input.CloseShieldInputStream
import java.io.File
import java.io.InputStream
import java.io.FileInputStream
import java.io.OutputStreamWriter
import java.util.logging.Level.*
import java.util.logging.Logger
import java.util.logging.LogManager.*
import java.util.Locale.*
import java.util.zip.ZipEntry
import java.util.zip.ZipInputStream
import javax.xml.parsers.DocumentBuilderFactory
import javax.xml.xpath.XPathConstants.NODE
import javax.xml.xpath.XPathConstants.NODESET
import javax.xml.xpath.XPathFactory
import java.text.DecimalFormat
import java.text.DecimalFormatSymbols
import java.time.LocalDate

fun NodeList.asList(): List<Node> {
    val nodes = mutableListOf<Node>()
    repeat(length) { index ->
        nodes.add(item(index))
    }
    return nodes
}

data class Party(val name: String)
data class Transaction(val date: LocalDate, val valuta: LocalDate, val amount: Money, val creditor: Party, val debtor: Party, val type: String, val description: String)

class Camt052File(val inputStream: InputStream) {

    val document = DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(inputStream)
    val xpath = XPathFactory.newInstance().newXPath()

    fun parse(): List<Transaction> {
        /*
            camt looks like a great file format to me. not.
            It has a lot of unreadable, crypticly shortened names.
    
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

            val creditor = element("NtryDtls/TxDtls/RltdPties/Cdtr/Nm")?.textContent ?: ""
            val debtor = element("NtryDtls/TxDtls/RltdPties/Dbtr/Nm")?.textContent ?: ""

            val date = LocalDate.parse(element("BookgDt/Dt")!!.textContent)
            val valuta = LocalDate.parse(element("ValDt/Dt")!!.textContent)

            val texts = (xpath.evaluate("NtryDtls/TxDtls/RmtInf/Ustrd", entry, NODESET) as NodeList).asList().map { it.textContent }

            Transaction(date, valuta, money, Party(creditor), Party(debtor), texts.getOrElse(0, { "" }), texts.getOrElse(1, { "" }))
        }
    }
}

Thread.currentThread().contextClassLoader = Camt052File::class.java.classLoader

getLogManager().getLogger("").setLevel(WARNING)

val transactions = mutableListOf<Transaction>()

for (arg in args) {
    val file = File(arg)
    ZipInputStream(FileInputStream(file)).use { zip ->

        var entry = zip.nextEntry
        while (entry != null) {
            val fileTransactions = Camt052File(CloseShieldInputStream(zip)).parse()
            transactions.addAll(fileTransactions)
            entry = zip.nextEntry
        }
    }
}

transactions.sortBy { it.date }

val format = DEFAULT.withDelimiter(';').withHeader("Date", "Valuta", "Amount", "Currency", "Creditor", "Debtor", "Type", "Description")
val printer = CSVPrinter(OutputStreamWriter(System.out, "UTF-8"), format)
transactions.forEach {
    printer.printRecord(it.date, it.valuta, DecimalFormat("#.##", DecimalFormatSymbols(US)).format(it.amount.number), it.amount.currency, it.creditor.name, it.debtor.name, it.type, it.description)
    printer.flush()
}
