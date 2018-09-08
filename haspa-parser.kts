#!/usr/bin/env kscript

@file:DependsOn("org.javamoney:moneta:1.3@pom")

import org.javamoney.moneta.Money
import org.w3c.dom.Element
import org.w3c.dom.Node
import org.w3c.dom.NodeList
import java.io.File
import java.io.InputStream
import java.io.FileInputStream
import java.util.logging.Level.*
import java.util.logging.Logger
import java.util.logging.LogManager.*
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

class Camt052File(val inputStream: InputStream) {

    val document = DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(inputStream)
    val xpath = XPathFactory.newInstance().newXPath()

    fun parse() {
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

        repeat(entries.length) { index ->
            val entry = entries.item(index) as Element

            val debit = (xpath.evaluate("CdtDbtInd", entry, NODE) as Element).textContent == "DBIT"

            val amountElement = xpath.evaluate("Amt", entry, NODE) as Element
            val amount = amountElement.textContent.toBigDecimal()
            val currency = amountElement.getAttribute("Ccy")
            val money = Money.of(if (debit) amount.negate() else amount, currency)

            val creditor = (xpath.evaluate("NtryDtls/TxDtls/RltdPties/Cdtr/Nm", entry, NODE) as Element).textContent
            val debtor = (xpath.evaluate("NtryDtls/TxDtls/RltdPties/Dbtr/Nm", entry, NODE) as Element).textContent

            val texts = (xpath.evaluate("NtryDtls/TxDtls/RmtInf/Ustrd", entry, NODESET) as NodeList).asList().map { it.textContent }

            val transaction = Transaction(money, Party(creditor), Party(debtor), texts[0], texts[1])
            println("Transaction: $transaction")
        }
    }

    data class Party(val name: String)
    data class Transaction(val amount: Money, val creditor: Party, val debtor: Party, val type: String, val description: String)
}

Thread.currentThread().contextClassLoader = Camt052File::class.java.classLoader

getLogManager().getLogger("").setLevel(WARNING)

val file = File("./input.xml")
FileInputStream(file).use {
    Camt052File(it).parse()
}
