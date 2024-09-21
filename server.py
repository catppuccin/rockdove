from http.server import HTTPServer, BaseHTTPRequestHandler
import json
from pathlib import Path


class WebhookHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers["Content-Length"])
        post_data = self.rfile.read(content_length)
        payload = json.loads(post_data.decode("utf-8"))

        event_type = self.headers.get("X-GitHub-Event")
        # Worth noting that the action field can be the same for different
        # events that we want to track so we may want some custom logic in the
        # future to handle this or not bother with auto-saving the payload from
        # it.
        action = payload.get("action", "default")

        filename = Path(f"fixtures/{event_type}/{action}.json")
        filename.parent.mkdir(parents=True, exist_ok=True)
        with open(filename, "w") as f:
            json.dump(payload, f, indent=2)

        print(f"Received {event_type} event with action: {action}")
        print(f"Payload saved to {filename}")

        self.send_response(200)
        self.end_headers()


def run_server(port=3000):
    server_address = ("", port)
    httpd = HTTPServer(server_address, WebhookHandler)
    print(f"Server running on http://localhost:{port}")
    httpd.serve_forever()


if __name__ == "__main__":
    run_server()
