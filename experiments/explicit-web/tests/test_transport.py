# Contract: docs/guarantees/basic-app.md
import http.client
import tempfile
import unittest
import urllib.parse
from support import Client, build, server

class Transport(unittest.TestCase):
    @classmethod
    def setUpClass(cls): build("Checks")
    def test_failures_and_chunked_limits(self):
        with tempfile.TemporaryDirectory() as directory, server(directory, "Checks", arguments=["--serve-errors"], EXPLICIT_REQUEST_TIMEOUT_SECONDS="1") as address:
            client = Client(address)
            status, _, body = client.request("GET", "/throw")
            self.assertEqual(status, 500); self.assertNotIn(b"PRIVATE_EXCEPTION_MARKER", body); self.assertIn(b"request ID", body)
            self.assertEqual(client.request("GET", "/timeout")[0], 408)
            self.assertEqual(client.request("GET", "/ok")[0], 200)
            url = urllib.parse.urlsplit(address)
            connection = http.client.HTTPConnection(url.hostname, url.port, timeout=5)
            try:
                connection.request("POST", "/ok", body=iter([b"x"*32768, b"y"*32768, b"z"]), encode_chunked=True)
                response = connection.getresponse(); response.read()
                self.assertEqual(response.status, 413)
            finally: connection.close()

if __name__ == "__main__": unittest.main(verbosity=2)
