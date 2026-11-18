package com.hickorydocs.simple

import java.security.MessageDigest

internal data class MarkedSnippetResult(
    val source: String,
    val selection: CompletionSelection,
    val session: SnippetSession? = null,
    val parallel: ParallelSnippet? = null,
)

/** Owned by one native input connection. Native marked text remains authoritative until commit. */
internal class MarkedSnippetInput(
    val path: String,
    val source: String,
    val selection: CompletionSelection,
    val range: CompletionSelection,
    val history: SourceUndoSnapshot<AndroidSnippetHistory?>,
    val intent: Int,
) {
    private val prefix = source.substring(0, range.from)
    private val suffix = source.substring(range.to)
    private val trail = mutableListOf(sourceIdentity(source))
    private var overflow = false
    private var splitSource = false

    private fun confined(text: String): Boolean =
        text.length >= prefix.length + suffix.length && text.startsWith(prefix) && text.endsWith(suffix)

    fun observe(text: String) {
        if (!confined(text)) {
            splitSource = true
            return
        }
        append(text)
    }

    private fun append(text: String) {
        if (overflow || splitSource) return
        val proof = sourceIdentity(text)
        if (trail.last() == proof) return
        if (trail.size >= 64) {
            overflow = true
            return
        }
        trail.add(proof)
    }

    fun replay(finalSource: String, mirrored: Boolean): MarkedSourceReplay? {
        if (mirrored) append(finalSource) else observe(finalSource)
        if (finalSource == source || overflow || splitSource || trail.size < 2 || trail.last() != sourceIdentity(finalSource)) return null
        return MarkedSourceReplay(trail.toList())
    }

    fun finish(observed: String, selected: CompletionSelection): MarkedSnippetResult? {
        if (overflow || splitSource || !confined(observed)) return null
        val end = observed.length - suffix.length
        val inserted = observed.substring(prefix.length, end)
        if (inserted.length > 1024 * 1024 || inserted.toByteArray(Charsets.UTF_8).size > 1024 * 1024 ||
            selected.from < range.from || selected.to > end || selected.from > selected.to) return null
        val offset = selected.from - end
        val length = selected.to - selected.from
        val context = history.context ?: return null
        val multiple = context.group
        if (multiple != null) {
            if (multiple.selections[multiple.primaryIndex] != range) return null
            val plan = multiple.editing(source, multiple.selections.map { NativeSourceEdit(it.from, it.to, inserted) }) ?: return null
            val primary = plan.state.primaryIndex
            val ranges = plan.state.selections.toMutableList()
            val start = ranges[primary].from + offset
            ranges[primary] = CompletionSelection(start, start + length)
            val next = plan.state.selecting(plan.state.source, ranges, primary) ?: return null
            return MarkedSnippetResult(next.source, ranges[primary], parallel = next)
        }
        val current = context.session ?: return null
        val plan = editSnippet(current, source, range.from, range.to, inserted) ?: return null
        val start = plan.selection.from + offset
        return MarkedSnippetResult(plan.session.source, CompletionSelection(start, start + length), session = plan.session)
    }
}

private fun sourceIdentity(source: String): String {
    val digest = MessageDigest.getInstance("SHA-256")
    val bytes = ByteArray(4096)
    var at = 0
    for (unit in source) {
        bytes[at++] = (unit.code ushr 8).toByte()
        bytes[at++] = unit.code.toByte()
        if (at == bytes.size) {
            digest.update(bytes)
            at = 0
        }
    }
    digest.update(bytes, 0, at)
    return digest.digest().joinToString("") { "%02x".format(it) }
}

/** Native Compose may merge proposals. Accept a monotone subsequence of the
 * observed revisions. If a replay steps into unrelated history, reverse just
 * the performed steps before handing ordinary Undo or Redo back to the caller.
 */
internal class MarkedSourceReplay(private val revisions: List<String>) {
    enum class Result { Finished, Ordinary, Interrupted }
    val retainedBytes: Long get() = 128L + 160L * revisions.size

    fun run(
        backwards: Boolean,
        read: () -> String,
        move: (Boolean) -> Boolean,
    ): Result {
        val order = if (backwards) revisions.asReversed() else revisions
        if (sourceIdentity(read()) != order.first()) return Result.Ordinary
        var at = 0
        var performed = 0
        while (at < order.lastIndex && performed < order.lastIndex) {
            if (!move(backwards)) break
            performed++
            val key = sourceIdentity(read())
            val next = (at + 1..order.lastIndex).firstOrNull { order[it] == key } ?: break
            at = next
        }
        if (at == order.lastIndex) return Result.Finished
        repeat(performed) {
            if (!move(!backwards)) return Result.Interrupted
        }
        return if (sourceIdentity(read()) == order.first()) Result.Ordinary else Result.Interrupted
    }
}

internal fun changedMarkedSource(before: String, after: String): NativeSourceEdit? {
    if (before == after) return null
    fun boundary(text: String, offset: Int): Boolean {
        if (offset <= 0 || offset >= text.length) return true
        val left = text[offset - 1]
        val right = text[offset]
        return !(left == '\r' && right == '\n' || left.isHighSurrogate() && right.isLowSurrogate())
    }
    val limit = minOf(before.length, after.length)
    var first = 0
    while (first < limit && before[first] == after[first]) first++
    while (!boundary(before, first) || !boundary(after, first)) first--
    var tail = 0
    while (tail < limit - first && before[before.lastIndex - tail] == after[after.lastIndex - tail]) tail++
    while (!boundary(before, before.length - tail) || !boundary(after, after.length - tail)) tail--
    return NativeSourceEdit(first, before.length - tail, after.substring(first, after.length - tail))
}
