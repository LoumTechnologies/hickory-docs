#!/usr/bin/env python3
"""Focused real-browser witness. Run against a disposable workspace URL.

uv run --with playwright==1.63.0 python scripts/test-quiet-workspace.py URL
Uses the existing Playwright Chromium installation; no provider requests.
"""
import json
import sys
import time
from playwright.sync_api import sync_playwright, expect

# Contract: .hickory-source/guarantees/shell/quiet-inline-workspace/contract.md

def command(page, text):
    page.keyboard.press("ControlOrMeta+Shift+P")
    field = page.get_by_role("textbox", name="Commands")
    field.fill(">" + text)
    option = page.get_by_role("option").filter(has_text=text).first
    expect(option).to_be_visible()
    field.press("Enter")


# Contract: .hickory-source/guarantees/authoring/a-tree-row-opens-in-the-platform/contract.md
def check_links(page, origin):
    """Drive only public HTTP and visible UI, independent of editor internals."""
    source = ('<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">\n'
              '# Link reading\n\nSee [related note](notes/other.md) and '
              '[the web](https://example.com).\n\n'
              '@notes/other.md\n\n<hick:include file="notes/other.md" />\n</hick:doc>\n')
    response = page.request.post(origin + "/api/projects/local/docs", data={"path": "links.md", "source": source})
    assert response.ok, response.text()
    address = origin + "/#/docs/" + response.json()["id"]
    page.goto(address)
    editor = page.locator('[data-spatial-key="document:links.md"] .cm-content').first
    label = editor.locator('.cm-md-link').filter(has_text="related note")
    expect(label).to_be_visible(timeout=45000)
    expect(editor).not_to_contain_text("](notes/other.md)")
    label.click()
    expect(editor).to_contain_text("[related note](notes/other.md)")
    expect(editor).not_to_contain_text("](https://example.com)")
    editor.locator('.cm-line').filter(has_text="Link reading").click()
    expect(editor).not_to_contain_text("](notes/other.md)")
    label.click(button="right")
    for name in ["Rename…", "Move to…", "Copy to…", "Delete", "Copy relative path", "Open in default program"]:
        expect(page.get_by_role("menuitem", name=name, exact=True)).to_be_visible()
    page.keyboard.press("Escape")
    label.dblclick()
    destination = page.locator('[data-spatial-key="document:notes/other.md"]')
    expect(destination).to_have_attribute("data-spatial-center", "true")
    page.goto(address)
    launches = []
    def launch(route):
        launches.append(route.request.post_data_json)
        route.fulfill(json={"ok": True})
    # A boundary witness records the requested browser without opening external apps.
    page.route("**/api/open-external", launch)
    web = page.locator('[data-spatial-key="document:links.md"] .cm-md-link').filter(has_text="the web")
    with page.expect_response(lambda response: response.url.endswith("/api/open-external")):
        web.dblclick()
    assert {"url": "https://example.com"} in launches, launches
    web.click(button="right")
    with page.expect_response(lambda response: response.url.endswith("/api/open-external")):
        page.get_by_role("menuitem", name="Open in Firefox", exact=True).click()
    assert {"url": "https://example.com", "browser": "firefox"} in launches, launches
    editor.locator('.cm-line').filter(has_text='file="notes/other.md"').dblclick()
    expect(destination).to_have_attribute("data-spatial-center", "true")
    page.goto(address)
    at_include = editor.locator('.cm-md-link').filter(has_text="@notes/other.md")
    expect(at_include).to_be_visible()
    at_include.click(button="right")
    expect(page.get_by_role("menuitem", name="Open in default program", exact=True)).to_be_visible()
    page.keyboard.press("Escape")
    at_include.dblclick()
    expect(destination).to_have_attribute("data-spatial-center", "true")
    print(json.dumps({"result": "PASS", "scope": "link labels, selection, file actions, double-click, include, @include, browser requests"}))


def main():
    if len(sys.argv) not in (2, 3):
        raise SystemExit("Usage: test-quiet-workspace.py DISPOSABLE_WORKSPACE_URL")
    with sync_playwright() as automation:
        browser = automation.chromium.launch()
        page = browser.new_page(viewport={"width": 1200, "height": 900})
        errors = []
        page.on("pageerror", lambda error: errors.append(str(error)))
        started = time.monotonic()
        if len(sys.argv) == 3 and sys.argv[2] == "--links":
            check_links(page, sys.argv[1].rstrip("/"))
            assert not errors, errors
            browser.close()
            return
        page.goto(sys.argv[1])
        editor = page.locator(".untitled-tab .cm-content").first
        expect(editor).to_be_visible(timeout=45000)
        open_ms = round((time.monotonic() - started) * 1000)
        footer = page.get_by_role("contentinfo")
        for label in ["Graph", "Environments", "Lineage", "Context", "Declared", "Continuity", "Execution", "Workspace tools"]:
            expect(footer.get_by_text(label, exact=True)).to_have_count(0)
        editor.click()
        page.keyboard.press("ControlOrMeta+End")
        text = "Quiet workspace typing witness."
        started = time.monotonic()
        page.keyboard.insert_text(text)
        expect(editor).to_contain_text(text)
        typing_ms = round((time.monotonic() - started) * 1000)
        command(page, "Connection display")
        panel = page.get_by_role("region", name="Connection display", exact=True)
        expect(panel).to_be_visible()
        expect(panel.get_by_role("button", name="Lineage", exact=True)).to_be_visible()
        panel.get_by_role("button", name="Close workspace tool").click()
        expect(panel).to_have_count(0)
        expect(editor).to_contain_text(text)
        command(page, "Agent")
        agent = page.get_by_role("region", name="Agent chat")
        expect(agent).to_be_visible()
        expect(agent.locator(".chat-identity")).to_have_count(0)
        expect(agent.get_by_role("checkbox", name="Measure this request with and without Hickory features")).to_have_count(0)
        status = footer.locator(".agent-status > summary")
        expect(status).to_be_visible()
        status.focus()
        page.keyboard.press("Enter")
        expect(footer.get_by_role("region", name="Agent account details")).to_be_visible()
        page.keyboard.press("Escape")
        expect(footer.get_by_role("region", name="Agent account details")).not_to_be_visible()
        agent.get_by_role("button", name="Agent settings").click()
        expect(page.get_by_role("checkbox", name="Measure this request with and without Hickory features")).to_be_visible()
        page.screenshot(path="/tmp/hickory-quiet-workspace.png", full_page=True)
        assert not errors, errors
        print(json.dumps({"result": "PASS", "first_editable_view_ms": open_ms, "insert_and_observe_ms": typing_ms,
                          "scope": "fresh development fixture, one sample; no release performance claim", "screenshot": "/tmp/hickory-quiet-workspace.png"}))
        browser.close()


if __name__ == "__main__":
    main()
