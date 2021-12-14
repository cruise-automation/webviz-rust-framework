#!/usr/bin/python2

# Copyright (c) 2021-present, Cruise LLC
#
# This source code is licensed under the Apache License, Version 2.0,
# found in the LICENSE-APACHE file in the root directory of this source tree.
# You may not use this file except in compliance with the License.

# A small abstraction over SimpleHTTPServer so that
# we can set content headers correctly.

import SimpleHTTPServer
import SocketServer
import urllib

PORT = 3000
WEBPACK_URL = "http://localhost:3001"
EXCLUDED_HEADERS = [
    "Cross-Origin-Embedder-Policy",
    "Cross-Origin-Opener-Policy",
    "Date"
]


class Handler(SimpleHTTPServer.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Access-Control-Allow-Origin", "*")
        SimpleHTTPServer.SimpleHTTPRequestHandler.end_headers(self)

    # Simple proxying for webpack requests. We need this because web workers
    # must be from the same origin.
    def do_GET(self):
        # Assume webpack requests always start with `dist`
        if self.path.startswith("/dist"):
            self.send_response(200)
            response = urllib.urlopen(WEBPACK_URL + self.path)

            for key, value in response.headers.items():
                key = "-".join([w.capitalize() for w in key.split("-")])
                if key not in EXCLUDED_HEADERS:
                    self.send_header(key, value)

            self.end_headers()
            self.copyfile(response, self.wfile)
        else:
            SimpleHTTPServer.SimpleHTTPRequestHandler.do_GET(self)


Handler.extensions_map['.wasm'] = 'application/wasm'

httpd = SocketServer.TCPServer(("", PORT), Handler)

print("Serving at http://localhost:{}".format(PORT))
httpd.serve_forever()
