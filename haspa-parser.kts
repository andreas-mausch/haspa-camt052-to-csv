#!/usr/bin/env kscript

@file:DependsOn("org.javamoney:moneta:1.3@pom")

import org.javamoney.moneta.Money
import org.w3c.dom.Element
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

            println("Amount: $money")
        }
    }

    data class Entry(val amount: Money)
}

Thread.currentThread().contextClassLoader = Camt052File::class.java.classLoader

getLogManager().getLogger("").setLevel(WARNING)

val file = File("./input.xml")
FileInputStream(file).use {
    Camt052File(it).parse()
}
