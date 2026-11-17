package com.hickorydocs.simple
import org.json.JSONObject
internal data class NativeAttachProfile(val port:Int) {
 val notice get()="Attach on loopback port $port · external source revision unverified"
}
internal fun nativeAttachProfile(profile:JSONObject):NativeAttachProfile? {
 if(!profile.has("attach"))return null
 val value=profile.get("attach") as? JSONObject ?: error("Malformed Attach profile; no debugger request was sent.")
 val raw=value.get("port")
 val number=(raw as? Number)?.toString()?.toIntOrNull() ?: error("Attach profile has no canonical loopback port.")
 check(number in 1..65535){"Attach profile loopback port is out of range."}
 check(value.get("arguments") is JSONObject){"Attach profile arguments are unavailable."}
 return NativeAttachProfile(number)
}
internal fun nativeDebugProfileLabel(profile:JSONObject):String =
 (if(nativeAttachProfile(profile)!=null)"Attach " else "Debug ")+profile.getString("name")
