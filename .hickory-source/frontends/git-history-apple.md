<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read immutable repository history without replacing the editor

For maintainers changing workspace services independently of the editor renderer.

History is a read-only capability over saved Git facts. Capture the requested workspace/path and request intent independently of the active editor. Retain full commit identities from the log, require an exact identity match when fetching immutable before/after blobs, and retire older or closed replies. Local paging and a bounded log window keep the floating reading compact. No checkout, save, index or source mutation occurs.

<ui:copy id="simple-git-history-apple">
import SwiftUI
import Observation
enum GitHistoryRequest {
 case log(String?)
 case commit(String,String)
 var title:String{switch self{case .log(let path):return path.map{"Saved history · \($0)"} ?? "Repository history";case .commit(_,let title):return title}}
 var commitID:String?{if case .commit(let sha,_)=self{return sha};return nil}
 var path:String?{if case .log(let path)=self{return path};return nil}
 var isLog:Bool{if case .log=self{return true};return false}
}
struct HistoryCommit:Decodable,Identifiable {var sha:String;var short:String;var author:String;var subject:String;var id:String{sha}}
struct HistoryLog:Decodable {var repository:Bool;var commits:[HistoryCommit]}
struct HistoryFile:Decodable {var path:String;var from:String;var status:String;var before:String;var after:String;var binary:Bool}
struct HistoryReading:Decodable {var sha:String;var message:String;var files:[HistoryFile]}
@MainActor @Observable final class GitHistoryModel {
 let request:GitHistoryRequest
 init(request:GitHistoryRequest = .log(nil)){self.request=request}
 var commits:[HistoryCommit]=[];var reading:HistoryReading?;var file=0;var before=false;var page=0;var limit=120;var notice="";var loading=false;private var ticket=UUID()
 func close(){ticket=UUID();loading=false;reading=nil;notice=""}
 func refresh(limit:Int=120,load:(Int,String?)async throws->HistoryLog)async {
  guard request.isLog else{notice="Choose a log request before reading commit choices.";return};let path=request.path
  let id=UUID();ticket=id;self.limit=max(1,min(1000,limit));commits=[];reading=nil;page=0;loading=true;notice="Reading saved history…";defer{if ticket==id{loading=false}}
  do{let value=try await load(self.limit,path);guard ticket==id else{return};guard value.commits.allSatisfy({$0.sha.range(of:"^([0-9a-f]{40}|[0-9a-f]{64})$",options:.regularExpression) != nil}) else{throw EngineError(message:"Malformed commit identity. Refresh repository history.")};commits=value.commits;notice=value.repository ? "\(commits.count) reported commits · saved history only.":"This workspace is not a Git repository."}catch{if ticket==id{notice=error.localizedDescription}}
 }
 func select(_ sha:String,load:(String)async throws->HistoryReading)async {
  guard commits.contains(where:{$0.sha==sha}) else{return};await readCommit(sha,load:load)
 }
 func readCommit(_ sha:String,load:(String)async throws->HistoryReading)async {
  guard sha.range(of:"^([0-9a-f]{40}|[0-9a-f]{64})$",options:.regularExpression) != nil,(request.commitID==nil || request.commitID==sha) else{ticket=UUID();reading=nil;loading=false;notice="Commit reference is missing or changed. Refresh its source and choose the commit again.";return};let id=UUID();ticket=id;reading=nil;loading=true;notice="Reading immutable commit…";defer{if ticket==id{loading=false}}
  do{let value=try await load(sha);guard ticket==id else{return};guard value.sha==sha else{throw EngineError(message:"Commit identity changed. Refresh repository history.")};reading=value;file=0;before=false;notice="Immutable commit reading · current editor draft is retained."}catch{if ticket==id{notice=error.localizedDescription}}
 }
}
</ui:copy>

## Admit logs and observed commits as distinct reading requests

A typed request owns either a saved-history scope or one full observed commit identity. Attribution can therefore reuse immutable before/after reading without inventing a log result or resolving a display abbreviation. The commit request must match every read and response; a rejected new reference retires older work.

## Present immutable file sides without replacing live source

The existing floating surface pages commit choices and reads exact before/after blobs. Historical file selection is local to the reading; it does not activate, reload or edit the source buffer. Render text literally, retain binary and empty-side distinctions, and report display limits. Closing returns focus and retires outstanding reads. A fresh request context keeps its title and data together: browser models are keyed by workspace/path, and native menu adapters replace the read session before presenting a different request. Closed readings clear their visible content before reuse.

<ui:copy id="simple-git-history-apple-view">
struct GitHistoryView:View {
 @State private var comparing = false
 var workspace:Workspace
 var model:GitHistoryModel
 private func refresh(_ count:Int=120)async {guard let engine=workspace.engine else{model.notice="Connect to a workspace server first.";return};if case .commit(let sha,_)=model.request{await model.readCommit(sha){try await engine.request("GET","/api/git/reading",query:[URLQueryItem(name:"sha",value:$0)])};return};await model.refresh(limit:count){count,path in var query=[URLQueryItem(name:"limit",value:String(count))];if let path{query.append(URLQueryItem(name:"path",value:path))};return try await engine.request("GET","/api/git/log",query:query)}}
 private func select(_ sha:String)async {guard let engine=workspace.engine else{return};await model.select(sha){try await engine.request("GET","/api/git/reading",query:[URLQueryItem(name:"sha",value:$0)])}}
 var body:some View {
  VStack(alignment:.leading,spacing:8){
   HStack{Text(model.request.title).font(.headline);Spacer();Button("Refresh"){Task{await refresh(model.limit)}};Button("Close"){workspace.showHistory=false;workspace.currentBuffer?.focusSource?()}}
   Text(model.notice).font(.caption).textSelection(.enabled)
   if model.request.isLog {
    Toggle("Compare two commits", isOn: $comparing).toggleStyle(.button)
   }
   if comparing {
    GitCommitComparisonView(workspace: workspace)
   } else if model.request.isLog{
   ScrollView{VStack(alignment:.leading){ForEach(Array(model.commits.dropFirst(model.page*20).prefix(20))){commit in Button("\(commit.short) · \(commit.subject) · \(commit.author)"){Task{await select(commit.sha)}}.accessibilityLabel("Read commit \(commit.sha)")}}}
   HStack{Button("Previous"){model.page-=1}.disabled(model.page==0);Text("Page \(model.page+1)");Button("Next"){model.page+=1}.disabled((model.page+1)*20>=model.commits.count);Button("Load more commits"){Task{await refresh(min(1000,model.limit+120))}}.disabled(model.limit>=1000 || model.commits.count<model.limit)}.font(.caption)
   }
   if !comparing,let reading=model.reading{Text(reading.sha).font(.caption).textSelection(.enabled);Text(String(reading.message.prefix(20000))).font(.caption).textSelection(.enabled);if reading.message.count>20000{Text("First 20,000 message characters shown.").font(.caption)}
    if reading.files.indices.contains(model.file){Picker("Historical file",selection:Binding(get:{model.file},set:{model.file=$0})){ForEach(reading.files.indices,id:\.self){index in Text("\(reading.files[index].status) · \(reading.files[index].path)").tag(index)}}
     Picker("Historical side",selection:Binding(get:{model.before},set:{model.before=$0})){Text("Before commit").tag(true);Text("After commit").tag(false)}
     let file=reading.files[model.file],content=model.before ? file.before:file.after
     Text(file.binary ? "Binary content · text reading unavailable.":content.count>200000 ? "First 200,000 characters shown.":content.isEmpty ? "No text on this side.":"Immutable file content.").font(.caption)
     ScrollView([.horizontal,.vertical]){Text(file.binary ? "":String(content.prefix(200000))).font(.system(.caption,design:.monospaced)).textSelection(.enabled).frame(maxWidth:.infinity,alignment:.leading)}
    }else{Text("No files under this workspace in this commit.")}
   }
  }.padding().frame(idealWidth:680,idealHeight:600).task(id:ObjectIdentifier(model)){await refresh()}.onDisappear{model.close()}
 }
}
</ui:copy>

## Compare a commit with an editable working file

The picker pins the two commit IDs before it requests a file patch. Opening
the comparison attaches its baseline to the existing buffer. Recompute line
markers from the current draft off the main actor and retire them on every
source change until an exact-source result arrives. Neither the left line
numbers nor the right source actions change their coordinate system.

<ui:copy id="apple-git-commit-comparison">
struct GitComparisonFiles: Decodable {
    struct File: Decodable {
        var path: String
        var status: String
    }
    var sha: String
    var parent: String
    var files: [File]
}

struct GitComparisonFile: Decodable {
    struct File: Decodable, Sendable {
        var path: String
        var status: String
        var before: String
        var after: String
        var patch: String
        var patch_truncated: Bool
        var binary: Bool
    }
    var sha: String
    var parent: String
    var files: [File]
}

struct GitEditorComparison: Sendable {
    let id = UUID()
    let path: String
    let base: String
    let target: String
    let file: GitComparisonFile.File

    struct Line: Sendable {
        var added: String?
        var removed: [String] = []
    }

    static func changes(from before: String, to draft: String) -> [Int: Line]? {
        guard before.utf16.count <= 200_000, draft.utf16.count <= 200_000 else { return nil }
        func rows(_ source: String) -> [Data] {
            guard !source.isEmpty else { return [] }
            let lines = source.components(separatedBy: "\n")
            return lines.enumerated().compactMap { index, line in
                if index == lines.count - 1 { return line.isEmpty ? nil : Data(line.utf8) }
                return Data((line + "\n").utf8)
            }
        }
        let old = rows(before), new = rows(draft)
        guard old.count <= 4000, new.count <= 4000 else { return nil }
        let difference = new.difference(from: old)
        func display(_ data: Data) -> String {
            data.last == 10 ? String(decoding: data.dropLast(), as: UTF8.self) : String(decoding: data, as: UTF8.self)
        }
        var removals = Set<Int>(), insertions = Set<Int>()
        for edit in difference {
            switch edit {
            case .remove(let offset, _, _): removals.insert(offset)
            case .insert(let offset, _, _): insertions.insert(offset)
            }
        }
        var result: [Int: Line] = [:]
        var o = 0, n = 0
        while o < old.count || n < new.count {
            if removals.contains(o) {
                // A deletion at EOF belongs to the final addressable editor line.
                let anchor = min(n, max(0, new.count - 1 + (draft.utf8.last == 10 ? 1 : 0)))
                result[anchor, default: Line()].removed.append(display(old[o]))
                o += 1
            } else if insertions.contains(n) {
                result[n, default: Line()].added = display(new[n])
                n += 1
            } else {
                o += 1; n += 1
            }
        }
        return result
    }
}

struct GitCommitComparisonView: View {
    let workspace: Workspace
    @State private var base = ""
    @State private var target = ""
    @State private var listing: GitComparisonFiles?
    @State private var selection: Data?
    @State private var reading: GitEditorComparison?
    @State private var notice = ""
    @State private var busy = false
    @State private var ticket = UUID()

    private var refs: [URLQueryItem] {
        [URLQueryItem(name: "base", value: base.trimmingCharacters(in: .whitespacesAndNewlines)),
         URLQueryItem(name: "sha", value: target.trimmingCharacters(in: .whitespacesAndNewlines))]
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            TextField("Base commit", text: $base).autocorrectionDisabled()
            TextField("Target commit", text: $target).autocorrectionDisabled()
            Button("Read comparison") { Task { await list() } }
                .disabled(busy || refs.contains { $0.value?.isEmpty != false })
            if let listing {
                Text("\(listing.parent.prefix(12)) → \(listing.sha.prefix(12))").font(.caption.monospaced())
                if listing.files.isEmpty {
                    Text("The selected commits have no changes in this workspace.").font(.caption)
                } else {
                    Picker("Changed file", selection: $selection) {
                        Text("Choose a file").tag(Data?.none)
                        ForEach(listing.files.indices, id: \.self) { index in
                            let file = listing.files[index]
                            Text("\(file.status) · \(file.path)").tag(Optional(Data(file.path.utf8)))
                        }
                    }
                    .onChange(of: selection) { _, selected in Task { await read(selected, listing: listing) } }
                }
            }
            if let reading {
                if reading.file.binary {
                    Text("Binary change").font(.caption)
                } else {
                    if reading.file.patch_truncated { Text("First 200,000 patch characters shown.").font(.caption) }
                    ScrollView([.vertical, .horizontal]) {
                        Text(reading.file.patch).font(.caption.monospaced()).textSelection(.enabled)
                    }.frame(minHeight: 160, maxHeight: 320)
                    if workspace.workspaceFile(reading.path) != nil {
                        Button("Open working file with comparison") { Task { await open(reading) } }.disabled(busy)
                    } else {
                        Text("This path has no working file to open.").font(.caption)
                    }
                }
            }
            if !notice.isEmpty { Text(notice).font(.caption).textSelection(.enabled) }
        }
        .onChange(of: refs) { _, _ in retire() }
        .onDisappear { retire() }
    }

    private func retire() {
        ticket = UUID(); listing = nil; selection = nil; reading = nil; busy = false; notice = ""
    }

    private func list() async {
        guard let engine = workspace.engine else { notice = "Connect to a workspace first."; return }
        retire()
        let id = ticket, query = refs
        busy = true
        defer { if ticket == id { busy = false } }
        do {
            let value: GitComparisonFiles = try await engine.request("GET", "/api/git/reading", query: query)
            guard ticket == id, workspace.engine?.session === engine.session else { return }
            guard [value.parent, value.sha].allSatisfy({ $0.range(of: "^([0-9a-f]{40}|[0-9a-f]{64})$", options: .regularExpression) != nil }) else {
                throw EngineError(message: "The workspace returned an invalid commit identity.")
            }
            listing = value
            if let path = workspace.currentBuffer?.file.path,
               value.files.contains(where: { exactUTF8($0.path, path) }) { selection = Data(path.utf8) }
        } catch { if ticket == id { notice = error.localizedDescription } }
    }

    private func read(_ selected: Data?, listing: GitComparisonFiles) async {
        guard selected == selection else { return }
        let id = UUID(); ticket = id; reading = nil; notice = ""; busy = false
        guard let selected, let path = String(data: selected, encoding: .utf8), let engine = workspace.engine else { return }
        busy = true
        defer { if ticket == id { busy = false } }
        do {
            let result: GitComparisonFile = try await engine.request("GET", "/api/git/reading", query: [
                URLQueryItem(name: "base", value: listing.parent), URLQueryItem(name: "sha", value: listing.sha),
                URLQueryItem(name: "path", value: path),
            ])
            guard ticket == id, workspace.engine?.session === engine.session, selected == selection else { return }
            guard result.parent == listing.parent, result.sha == listing.sha, result.files.count == 1,
                  let file = result.files.first, exactUTF8(file.path, path) else {
                throw EngineError(message: "The patch does not match the selected commits and file.")
            }
            reading = GitEditorComparison(path: path, base: result.parent, target: result.sha, file: file)
        } catch { if ticket == id { notice = error.localizedDescription } }
    }

    private func open(_ comparison: GitEditorComparison) async {
        let id = ticket
        busy = true
        defer { if ticket == id { busy = false } }
        if await workspace.openGitComparison(comparison, current: { ticket == id && reading?.id == comparison.id }) {
            workspace.showHistory = false
        } else if ticket == id {
            notice = "The working file could not be opened. Check Files and try again."
        }
    }
}

struct GitEditorComparisonTask: ViewModifier {
    var buffer: DocumentBuffer

    func body(content: Content) -> some View {
        content.task(id: buffer.gitComparison?.id.uuidString.appending(buffer.sourceEditIdentity.uuidString)) {
            guard let comparison = buffer.gitComparison else { buffer.gitComparisonChanges = nil; return }
            let source = buffer.text, revision = buffer.sourceEditIdentity
            do { try await Task.sleep(for: .milliseconds(100)) } catch { return }
            let job = Task.detached(priority: .userInitiated) {
                GitEditorComparison.changes(from: comparison.file.before, to: source)
            }
            let changes = await job.value
            guard !Task.isCancelled, buffer.gitComparison?.id == comparison.id, buffer.sourceEditIdentity == revision else { return }
            buffer.gitComparisonChanges = (comparison.id, revision, changes)
        }
    }
}

struct GitEditorComparisonLine: View {
    var buffer: DocumentBuffer
    var comparison: GitEditorComparison
    var line: Int
    @State private var notice = ""

    var body: some View {
        let change = buffer.gitChanges?[line]
        VStack(alignment: .leading, spacing: 8) {
            Text("Base \(comparison.base.prefix(12)) → working draft").font(.headline)
            if exactUTF8(buffer.text, comparison.file.after) {
                Text("Draft matches target \(comparison.target.prefix(12)).").font(.caption)
            } else if !buffer.dirty && !buffer.readOnly && !comparison.file.status.hasPrefix("D") {
                Button("Use target \(comparison.target.prefix(12)) as draft") {
                    let source = buffer.text
                    guard buffer.gitComparison?.id == comparison.id, exactUTF8(buffer.file.path, comparison.path),
                          !buffer.dirty, !buffer.readOnly else { return }
                    let edit = NativeSourceEdit(range: NSRange(location: 0, length: source.utf16.count), text: comparison.file.after)
                    if buffer.applySourceEdits?([edit], source) == true {
                        notice = ""
                    } else {
                        notice = "The editor could not start this draft. Return to the file and retry."
                    }
                }
            }
            if buffer.gitChanges == nil {
                Text("Line comparison is loading or exceeds the editor limit.").font(.caption)
            }
            if let change {
                ScrollView {
                    VStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(change.removed.enumerated()), id: \.offset) { entry in
                            Text("− \(entry.element)").foregroundStyle(.red)
                        }
                        if let added = change.added { Text("+ \(added)").foregroundStyle(.green) }
                    }.font(.caption.monospaced()).textSelection(.enabled)
                }.frame(maxHeight: 230)
            }
            DisclosureGroup("Commit patch") {
                Text("\(comparison.base.prefix(12)) → \(comparison.target.prefix(12))").font(.caption.monospaced())
                if comparison.file.patch_truncated { Text("First 200,000 patch characters shown.").font(.caption) }
                ScrollView([.horizontal, .vertical]) {
                    Text(comparison.file.patch).font(.caption.monospaced()).textSelection(.enabled)
                }.frame(height: 200)
            }
            if !notice.isEmpty { Text(notice).font(.caption) }
            Button("Close comparison") { buffer.gitComparison = nil }
        }.padding(12).frame(idealWidth: 360)
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/GitHistory.swift"><ui:paste select="#simple-git-history-apple"/><ui:paste select="#simple-git-history-apple-view"/><ui:paste select="#apple-git-commit-comparison"/></ui:file>
## Keep immutable readings bound to exact commit identities

Hold an immutable reading while selecting another commit, then release it after the newer reply. Exact full identities, closed-view retirement and malformed log identities must prevent a foreign or obsolete reading from replacing the selected commit. These service witnesses do not assert native pointer or accessibility acceptance.

<ui:copy id="simple-git-history-apple-tests">
import XCTest
@testable import HickorySimple
@MainActor final class GitHistoryTests:XCTestCase {
 func testHeldAndClosedReadingsRetainTheSelectedCommitIdentity()async {
  let model=GitHistoryModel(),a=String(repeating:"a",count:40),b=String(repeating:"b",count:40)
  await model.refresh(){_,_ in HistoryLog(repository:true,commits:[a,b].map{HistoryCommit(sha:$0,short:String($0.prefix(7)),author:"fixture",subject:"saved commit")})}
  var pending:CheckedContinuation<HistoryReading,Never>?
  let first=Task{await model.select(a){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()}
  await model.select(b){HistoryReading(sha:$0,message:"new",files:[])};pending?.resume(returning:HistoryReading(sha:a,message:"obsolete",files:[]));await first.value;XCTAssertEqual(model.reading?.sha,b);XCTAssertEqual(model.reading?.message,"new")
  pending=nil;let closed=Task{await model.select(a){_ in await withCheckedContinuation{pending=$0}}};while pending==nil{await Task.yield()};model.close();pending?.resume(returning:HistoryReading(sha:a,message:"closed",files:[]));await closed.value;XCTAssertNil(model.reading)
  await model.select(a){_ in HistoryReading(sha:b,message:"foreign",files:[])};XCTAssertNil(model.reading);XCTAssertTrue(model.notice.contains("identity changed"))
 }
 func testMalformedLogIdentityNeverBecomesAReadingChoice()async {
  let model=GitHistoryModel();await model.refresh(limit:2000){_,_ in HistoryLog(repository:true,commits:[HistoryCommit(sha:"HEAD",short:"HEAD",author:"fixture",subject:"ambiguous")])};XCTAssertEqual(model.limit,1000);XCTAssertTrue(model.commits.isEmpty);XCTAssertTrue(model.notice.contains("Malformed commit identity"))
 }
 func testObservedFullCommitRequestDoesNotManufactureALogChoice()async {
  let sha=String(repeating:"c",count:40),model=GitHistoryModel(request:.commit(String(repeating:"c",count:40),"Saved attribution"));var calls:[String]=[]
  func load(_ value:String)async throws->HistoryReading{calls.append(value);return HistoryReading(sha:value,message:"observed",files:[])}
  await model.readCommit(sha,load:load);XCTAssertEqual(calls,[sha]);XCTAssertEqual(model.reading?.sha,sha)
  await model.readCommit("12345678",load:load);XCTAssertEqual(calls,[sha]);XCTAssertNil(model.reading);await model.readCommit(String(repeating:"d",count:40),load:load);XCTAssertEqual(calls,[sha]);XCTAssertNil(model.reading)
 }

}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/GitHistoryTests.swift"><ui:paste select="#simple-git-history-apple-tests"/></ui:file>
</ui:doc>
