#!/usr/bin/env python3
"""Native HTTP + browser witness for recipe gutters and shared overflow geometry.
Contract: .hickory-source/guarantees/authoring/the-editor-keeps-its-space/contract.md
Run in the disposable workspace provided by test-spatial-ui.sh --just-gutters.
"""
import json
import sys
import time
from playwright.sync_api import sync_playwright, expect


def main(origin):
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch()
        page = browser.new_page(viewport={"width": 1200, "height": 900})
        request = page.request
        def post(path, data):
            response = request.post(origin + path, data=data)
            assert response.ok, response.text()
            return response.json()
        def file(path, content):
            post("/api/files/op", {"op": "create", "path": path})
            response = request.put(origin + "/api/file", data={"path": path, "content": content})
            assert response.ok, response.text()
        for directory in ["a-recipes", "b-recipes"]:
            post("/api/files/op", {"op": "mkdir", "path": directory})
            file(directory + "/justfile", "value := `touch discovery-ran`\n" + "\n".join(f"recipe{i}:\n    printf 'recipe-{i}-ok' > result.txt\n" for i in range(8)))
        file("justfile", "root-one:\n    printf root > root-result.txt\nroot-two:\n    true\n")
        response = request.get(origin + "/api/just-recipes?path=a-recipes%2Fjustfile")
        assert response.ok, response.text()
        assert len(response.json()["recipes"]) == 8
        assert request.get(origin + "/api/file?path=a-recipes%2Fdiscovery-ran").status == 404
        assert request.get(origin + "/api/just-recipes?path=..%2Fjustfile").status == 400
        page.goto(origin)
        page.evaluate("window.dispatchEvent(new CustomEvent('hickory-menu', {detail:'files'}))")
        files = page.locator('[data-spatial-files=left]')
        handle = page.get_by_role("button", name="Show left drawer")
        if handle.is_visible(): handle.click()
        expect(files).to_be_visible()
        tree = files.locator(".filesystem-editor")
        a = tree.locator('[data-tree-path="a-recipes"]')
        b = tree.locator('[data-tree-path="b-recipes"]')
        expect(a).to_be_visible()
        expect(b).to_be_visible()
        expect(tree.locator(".cm-gutter-more").first).to_be_visible()
        # Fold controls and recipes share one column. The following directory
        # keeps its original row center while the first row opens a grid.
        before = {"a": a.bounding_box(), "b": b.bounding_box()}
        more = tree.locator(".cm-gutter-more").first
        more.click()
        expect(tree.get_by_role("dialog", name="Gutter icons")).to_be_visible()
        for key, line in [("a", a), ("b", b)]:
            after = line.bounding_box()
            for dimension in ["x", "y", "width", "height"]:
                assert abs(after[dimension] - before[key][dimension]) < 1, (key, before[key], after)
        buttons = tree.locator('.cm-just-recipe[aria-label$="in a-recipes"]')
        expect(buttons.first).to_be_visible()
        rects = buttons.evaluate_all("nodes => nodes.map(node => {const r=node.getBoundingClientRect();return {x:r.x,y:r.y}})")
        assert len({round(rect["x"]) for rect in rects}) > 1, rects
        assert len({round(rect["y"]) for rect in rects}) > 1, rects
        page.screenshot(path="/tmp/hickory-files-gutter-grid.png")
        page.keyboard.press("Escape")
        expect(tree.get_by_role("dialog", name="Gutter icons")).to_have_count(0)
        expect(more).to_be_focused()
        more.click()
        # Exactly one recipe request starts, and it runs in its owning directory.
        with page.expect_response(lambda response: response.url.endswith("/api/terminals") and response.request.method == "POST") as launch:
            tree.get_by_role("button", name="Run just recipe3 in a-recipes", exact=True).click()
        session = launch.value.json()
        assert launch.value.ok, session
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            result = request.get(origin + "/api/file?path=a-recipes%2Fresult.txt")
            if result.ok and result.json()["content"] == "recipe-3-ok": break
            page.wait_for_timeout(100)
        else: raise AssertionError("recipe terminal did not produce its file in a-recipes")
        assert request.get(origin + "/api/file?path=b-recipes%2Fresult.txt").status == 404
        # Workspace-root recipes use the same overflow mechanism in the header.
        root = files.locator(".folder-tree__header")
        expect(root.locator(".cm-gutter-more")).to_be_visible()
        root.locator(".cm-gutter-more").click()
        expect(root.get_by_role("button", name="Run just root-one in .", exact=True)).to_be_visible()
        page.mouse.click(1100, 850)
        expect(root.get_by_role("dialog", name="Gutter icons")).to_have_count(0)
        page.screenshot(path="/tmp/hickory-just-gutters.png")
        doc = post("/api/projects/local/docs", {"path": "gutter-actions.md", "source": '<hick:exec container="shell">true</hick:exec>' * 24 + '\n' + "\n".join("Space below the actions." for _ in range(30))})
        page.goto(origin + "/#/docs/" + doc["id"])
        page.get_by_role("tablist", name="Center views").get_by_role("tab", name="gutter-actions.md", exact=True).click()
        rail = page.locator('[data-spatial-center=true] .cm-card-rail')
        cell_more = rail.locator(".cm-card-rail__more").first
        expect(cell_more).to_be_visible()
        tops = rail.locator("[data-card]").evaluate_all("nodes => nodes.map(node => node.style.top)")
        cell_more.click()
        grid = page.get_by_role("dialog", name="Gutter icons")
        expect(grid).to_be_visible()
        assert grid.locator("button").count() >= 2
        assert rail.locator("[data-card]").evaluate_all("nodes => nodes.map(node => node.style.top)") == tops
        page.screenshot(path="/tmp/hickory-document-gutter-grid.png")
        page.keyboard.press("Escape")
        expect(grid).to_have_count(0)
        expect(cell_more).to_be_focused()
        browser.close()
        print("Recipe discovery, scoped launch, root controls, overflow grid and unchanged row geometry passed.")


if __name__ == "__main__":
    main(sys.argv[1].rstrip("/"))
