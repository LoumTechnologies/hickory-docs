package com.hickorydocs.simple
import org.junit.Assert.*
import org.junit.Test
import org.json.JSONObject
class CoverageProjectionTest {
    @Test fun bundledCoverageOwnershipAndUnknowns(){
        val fixture=JSONObject(javaClass.classLoader!!.getResourceAsStream("coverage-projections.json")!!.bufferedReader().use {it.readText()});assertEquals(1,fixture.getInt("version"));val cases=fixture.getJSONArray("cases");assertEquals(5,cases.length())
        for(index in 0 until cases.length()){
            val test=cases.getJSONObject(index);val raw=test.getJSONObject("expected")
            val expected=raw.keys().asSequence().associateWith {path->val rows=raw.getJSONArray(path);(0 until rows.length()).map {i->val row=rows.getJSONObject(i);val origins=row.getJSONArray("origins");CoverageProjectedLine(row.getInt("line"),row.getString("state"),if(row.isNull("count"))null else row.getLong("count"),(0 until origins.length()).map {origins.getString(it)})}}
            assertEquals(test.getString("name"),expected,coverageProjection(test.getJSONObject("record")))
        }
    }
}
