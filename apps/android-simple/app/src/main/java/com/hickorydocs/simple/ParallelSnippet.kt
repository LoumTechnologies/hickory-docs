package com.hickorydocs.simple

internal data class ParallelSnippetExpansion(val edit: NativeSourceEdit, val fields: List<SnippetField>)
internal data class ParallelSnippetChange(val edits: List<NativeSourceEdit>, val state: ParallelSnippet)

internal data class ParallelSnippet(
    val source: String,
    val peers: List<Peer>,
    val primary: Int,
) {
    data class Peer(
        val id: Int, val from: Int, val to: Int, val text: String,
        val selection: CompletionSelection, val session: SnippetSession?,
    )

    val selections: List<CompletionSelection>
        get() = peers.map { CompletionSelection(it.from + it.selection.from, it.from + it.selection.to) }
    val primaryIndex: Int get() = peers.indexOfFirst { it.id == primary }

    companion object {
        const val maxCarets = 256
        private const val maxFields = 4096
        private const val maxBytes = 1024 * 1024

        private fun utf8Size(text: String): Int = text.toByteArray(Charsets.UTF_8).size

        private fun scalarText(text: String): Boolean {
            var at = 0
            while (at < text.length) {
                val unit = text[at++]
                if (unit.isHighSurrogate()) {
                    if (at >= text.length || !text[at++].isLowSurrogate()) return false
                } else if (unit.isLowSurrogate()) return false
            }
            return true
        }

        private fun boundary(text: String, at: Int): Boolean {
            if (at !in 0..text.length) return false
            if (at == 0 || at == text.length) return true
            val left = text[at - 1]
            val right = text[at]
            return !(left == '\r' && right == '\n' || left.isHighSurrogate() && right.isLowSurrogate())
        }

        private fun valid(text: String, from: Int, to: Int): Boolean =
            from <= to && boundary(text, from) && boundary(text, to)

        private fun replace(source: String, edits: List<NativeSourceEdit>): String = buildString {
            var start = 0
            for (edit in edits) {
                append(source, start, edit.from)
                append(edit.text)
                start = edit.to
            }
            append(source, start, source.length)
        }

        fun begin(source: String, expansions: List<ParallelSnippetExpansion>, primary: Int): ParallelSnippetChange? {
            if (expansions.size !in 2..maxCarets || primary !in expansions.indices ||
                source.length > 8 * maxBytes || !scalarText(source) || utf8Size(source) > 8 * maxBytes) return null
            val ordered = expansions.withIndex().sortedWith(compareBy({ it.value.edit.from }, { it.value.edit.to }))
            val peers = mutableListOf<Peer>()
            val edits = mutableListOf<NativeSourceEdit>()
            val budget = SnippetTransformBudget()
            var shift = 0
            var bytes = 0
            var metadata = 0L
            var fields = 0
            for ((id, expansion) in ordered) {
                val edit = expansion.edit
                val previous = edits.lastOrNull()
                if (!valid(source, edit.from, edit.to) ||
                    previous != null && (previous.to > edit.from || previous.from == edit.from && previous.to == edit.to) ||
                    edit.text.length > maxBytes - bytes || !scalarText(edit.text)) return null
                bytes += utf8Size(edit.text)
                fields += expansion.fields.size
                if (bytes > maxBytes || fields > maxFields) return null
                metadata += snippetMetadataCost(expansion.fields, maxBytes - metadata) ?: return null

                val session = beginSnippet(edit.text, expansion.fields, budget)
                val exit = expansion.fields.firstOrNull { it.index == 0 } ?: return null
                if (exit.from != exit.to || !valid(edit.text, exit.from, exit.to)) return null
                if (session == null && (expansion.fields.size != 1 || exit.transform != null ||
                    exit.parent != null || exit.id == null || exit.id !in 0 until maxFields)) return null
                val field = session?.fields?.firstOrNull { it.id == session.current } ?: exit
                val start = edit.from + shift
                peers.add(Peer(id, start, start + edit.text.length, edit.text,
                    CompletionSelection(field.from, field.to), session))
                edits.add(edit)
                shift += edit.text.length - (edit.to - edit.from)
            }
            val after = replace(source, edits)
            val state = ParallelSnippet(after, peers, primary)
            return if (state.matches(after)) ParallelSnippetChange(edits, state) else null
        }
    }

    private fun matches(text: String): Boolean {
        if (text != source) return false
        var previous = 0
        for (peer in peers) {
            if (peer.from < previous || !valid(text, peer.from, peer.to) ||
                text.substring(peer.from, peer.to) != peer.text ||
                !valid(peer.text, peer.selection.from, peer.selection.to) ||
                peer.session != null && peer.session.source != peer.text) return false
            previous = peer.to
        }
        return true
    }

    private fun localSelection(peer: Peer, absolute: CompletionSelection): CompletionSelection? {
        if (absolute.from < peer.from || absolute.to > peer.to) return null
        val range = CompletionSelection(absolute.from - peer.from, absolute.to - peer.from)
        if (!valid(peer.text, range.from, range.to)) return null
        val session = peer.session
        if (session != null && session.active != 0) {
            if (session.fields.none { it.id == session.current && it.from <= range.from && range.to <= it.to }) return null
            if (session.fields.any { it.transform != null && it.from < it.to &&
                    it.from <= range.from && range.to <= it.to &&
                    (range.from < range.to || it.from < range.from && range.from < it.to) }) return null
        }
        return range
    }

    fun selecting(source: String, ranges: List<CompletionSelection>, primary: Int): ParallelSnippet? {
        if (!matches(source) || ranges.size != peers.size || primary !in peers.indices) return null
        val selected = peers.mapIndexed { index, peer ->
            peer.copy(selection = localSelection(peer, ranges[index]) ?: return null)
        }
        return copy(peers = selected, primary = peers[primary].id)
    }

    fun editing(source: String, input: List<NativeSourceEdit>): ParallelSnippetChange? {
        if (!matches(source) || input.size != peers.size) return null
        val budget = SnippetTransformBudget()
        val edits = mutableListOf<NativeSourceEdit>()
        val updated = mutableListOf<Peer>()
        var shift = 0
        var bytes = 0
        for ((index, peer) in peers.withIndex()) {
            val replacement = input[index]
            val range = localSelection(peer, CompletionSelection(replacement.from, replacement.to)) ?: return null
            if (replacement.text.length > maxBytes - bytes || !scalarText(replacement.text)) return null
            val session = peer.session
            val plan = if (session != null && session.active != 0) {
                editSnippet(session, peer.text, range.from, range.to, replacement.text, budget) ?: return null
            } else null
            val local = plan?.edits ?: listOf(NativeSourceEdit(range.from, range.to, replacement.text))
            val text = plan?.session?.source ?: replace(peer.text, local)
            bytes += utf8Size(text)
            if (bytes > maxBytes) return null
            val start = peer.from + shift
            val caret = range.from + replacement.text.length
            updated.add(Peer(peer.id, start, start + text.length, text,
                plan?.selection ?: CompletionSelection(caret, caret), plan?.session))
            shift += text.length - peer.text.length

            for (localEdit in local) {
                val edit = NativeSourceEdit(peer.from + localEdit.from, peer.from + localEdit.to, localEdit.text)
                val previous = edits.lastOrNull()
                if (previous != null && previous.to > edit.from) return null
                if (previous != null && previous.to == edit.from) {
                    edits[edits.lastIndex] = NativeSourceEdit(previous.from, edit.to, previous.text + edit.text)
                } else edits.add(edit)
            }
        }
        val after = replace(source, edits)
        val next = copy(source = after, peers = updated)
        return if (next.matches(after)) ParallelSnippetChange(edits, next) else null
    }

    fun moving(source: String, direction: Int): ParallelSnippet? {
        if (!matches(source) || direction != 1 && direction != -1) return null
        val updated = peers.map { peer ->
            val session = peer.session ?: return@map peer
            val advance = advanceSnippet(session, peer.text, peer.selection.from, peer.selection.to, direction)
            if (advance == null) {
                val first = session.scopes.lastOrNull()?.order?.firstOrNull() ?:
                    session.fields.filter { it.transform == null && it.index > 0 }.minOfOrNull { it.index }
                if (direction == 1 && session.active == 0 || direction == -1 && session.active == first) return@map peer
                return null
            }
            val next = advance.session ?: session.copy(active = 0, current = session.fields.first { it.index == 0 }.id!!)
            peer.copy(selection = advance.selection, session = next)
        }
        return copy(peers = updated)
    }
}
