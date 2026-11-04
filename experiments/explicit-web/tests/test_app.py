# Contract: docs/guarantees/basic-app.md
import concurrent.futures
import json
import tempfile
import unittest
from support import Client, build, server

class BasicApp(unittest.TestCase):
    @classmethod
    def setUpClass(cls): build()

    def test_complete_app_and_safety_boundaries(self):
        with tempfile.TemporaryDirectory() as directory:
            with server(directory) as address:
                alice, bob, guest = Client(address), Client(address), Client(address)
                status, headers, body = guest.request("GET", "/")
                self.assertEqual(status, 200); self.assertIn(b"Create account", body)
                self.assertIn("HttpOnly", headers["Set-Cookie"]); self.assertIn("SameSite=Lax", headers["Set-Cookie"])
                self.assertEqual(guest.request("GET", "/api/tasks")[0], 401)
                alice.register("alice"); bob.register("bob")
                old_cookie, old_csrf = alice.cookie, alice.csrf
                self.assertEqual(alice.write("POST", "/api/tasks", {"title": ""})[0], 422)
                self.assertEqual(alice.request("POST", "/api/tasks", {"title": "must not exist"})[0], 403)
                self.assertEqual(alice.write("POST", "/api/tasks", {"title": "x", "unexpected": 1})[0], 400)
                self.assertEqual(alice.write("POST", "/api/tasks", b"{bad")[0], 415)
                self.assertEqual(alice.request("POST", "/api/tasks", b"{bad", {"Content-Type": "application/json", "X-CSRF-Token": alice.csrf})[0], 400)
                malicious = '<img src=x onerror=alert(1)>' + chr(39) + " OR 1=1 --"
                status, headers, body = alice.write("POST", "/api/tasks", {"title": malicious})
                self.assertEqual(status, 201); task = json.loads(body); path = headers["Location"]
                self.assertEqual(task["version"], 1)
                status, _, page = alice.request("GET", "/tasks/" + task["id"])
                self.assertEqual(status, 200); self.assertNotIn(b"<img", page); self.assertIn(b"&lt;img", page)
                self.assertEqual(json.loads(bob.request("GET", "/api/tasks")[2]), [])
                self.assertEqual(bob.request("GET", path)[0], 404)
                self.assertEqual(bob.write("PUT", path, {"title": "stolen", "version": 1})[0], 404)
                self.assertEqual(bob.write("DELETE", path, {"version": 1})[0], 404)
                self.assertEqual(alice.write("PUT", path, {"title": "changed", "done": True, "version": 1})[0], 200)
                self.assertEqual(alice.write("PUT", path, {"title": "stale", "version": 1})[0], 409)
                self.assertEqual(alice.write("DELETE", path, {"version": 1})[0], 409)
                current = json.loads(alice.request("GET", path)[2]); self.assertEqual(current["title"], "changed")
                self.assertTrue(current["done"]); self.assertEqual(current["version"], 2)
                # Two simultaneous edits of one version must produce one winner.
                def edit(title): return alice.write("PUT", path, {"title": title, "version": 2})[0]
                with concurrent.futures.ThreadPoolExecutor(2) as workers:
                    self.assertEqual(sorted(workers.map(edit, ["winner a", "winner b"])), [200, 409])
                current = json.loads(alice.request("GET", path)[2]); self.assertEqual(current["version"], 3)
                self.assertEqual(alice.request("GET", "/api/tasks/not-a-guid")[0], 400)
                self.assertEqual(alice.request("PATCH", path)[0], 405)
                status, headers, body = alice.request("HEAD", "/")
                self.assertEqual(status, 200); self.assertEqual(body, b""); self.assertGreater(int(headers["Content-Length"]), 0)
                status, headers, css = guest.request("GET", "/site.css")
                self.assertEqual(status, 200); self.assertIn("text/css", headers["Content-Type"])
                self.assertEqual(guest.request("GET", "/site.css", headers={"If-None-Match": headers["ETag"]})[0], 304)
                self.assertEqual(guest.request("GET", "/../app.sqlite")[0], 404)
                for origin in ["https://evil.example", "null", "https://reader.example.evil"]:
                    status, headers, _ = alice.request("GET", path, headers={"Origin": origin})
                    self.assertEqual(status, 403); self.assertNotIn("Access-Control-Allow-Origin", headers)
                status, headers, _ = guest.request("GET", "/missing", headers={"Origin": "https://reader.example"})
                self.assertEqual(status, 404); self.assertEqual(headers["Access-Control-Allow-Origin"], "https://reader.example")
                preflight = {"Origin": "https://reader.example", "Access-Control-Request-Method": "PUT", "Access-Control-Request-Headers": "content-type,x-csrf-token"}
                status, headers, body = guest.request("OPTIONS", path, headers=preflight)
                self.assertEqual(status, 204); self.assertEqual(body, b""); self.assertEqual(headers["Access-Control-Allow-Credentials"], "true")
                preflight["Access-Control-Request-Headers"] = "x-unknown"
                self.assertEqual(guest.request("OPTIONS", path, headers=preflight)[0], 403)
                self.assertEqual(alice.request("POST", "/api/tasks", b"x" * 65537, {"X-CSRF-Token": alice.csrf, "Content-Type": "application/json"})[0], 413)
                self.assertEqual(len(json.loads(alice.request("GET", "/api/tasks")[2])), 1)
            with server(directory) as address:
                alice.address = address
                self.assertEqual(alice.session()["user"], "alice")
                self.assertEqual(json.loads(alice.request("GET", path)[2])["version"], 3)
                self.assertEqual(alice.write("DELETE", path, {"version": 3})[0], 204)
                self.assertEqual(alice.request("GET", path)[0], 404)
                self.assertEqual(alice.request("POST", "/logout", form={"csrf": alice.csrf})[0], 303)
                replay = Client(address); replay.cookie = old_cookie
                self.assertEqual(replay.request("GET", "/api/tasks")[0], 401)
                alice.session()
                self.assertEqual(alice.request("POST", "/login", form={"name": "alice", "password": "incorrect password", "csrf": alice.csrf})[0], 401)
                self.assertEqual(alice.request("POST", "/login", form={"name": "alice", "password": "correct horse battery", "csrf": alice.csrf})[0], 303)
                self.assertNotEqual(alice.cookie, old_cookie)
                alice.session()
                self.assertEqual(alice.request("POST", "/tasks", form={"csrf": alice.csrf, "title": "form task"})[0], 303)
                task = json.loads(alice.request("GET", "/api/tasks")[2])[0]
                form_path = "/tasks/" + task["id"]
                form = {"csrf": alice.csrf, "title": "form edited", "done": "true", "version": 1, "action": "save"}
                self.assertEqual(alice.request("POST", form_path, form=form)[0], 303)
                form.update(version=2, action="delete")
                self.assertEqual(alice.request("POST", form_path, form=form)[0], 303)
                self.assertEqual(json.loads(alice.request("GET", "/api/tasks")[2]), [])

    def test_session_expiry_and_login_throttle(self):
        import time
        with tempfile.TemporaryDirectory() as directory, server(directory, EXPLICIT_SESSION_SECONDS="2", EXPLICIT_LOGIN_ATTEMPTS="2") as address:
            client = Client(address); client.register("expiry")
            time.sleep(2.1)
            self.assertEqual(client.request("GET", "/api/tasks")[0], 401)
            client.session()
            fields = {"csrf": client.csrf, "name": "expiry", "password": "wrong password"}
            self.assertEqual(client.request("POST", "/login", form=fields)[0], 401)
            self.assertEqual(client.request("POST", "/login", form=fields)[0], 429)

if __name__ == "__main__": unittest.main(verbosity=2)
