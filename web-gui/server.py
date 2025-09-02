#!/usr/bin/env python3
"""
Simple HTTP server for DeskAgent Web GUI
Serves the static HTML/CSS/JS files
"""

import http.server
import socketserver
import os
import webbrowser
from pathlib import Path

class DeskAgentHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    """Custom request handler with proper MIME types"""
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(Path(__file__).parent), **kwargs)
    
    def end_headers(self):
        # Add CORS headers for development
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        super().end_headers()
    
    def log_message(self, format, *args):
        """Custom log format"""
        print(f"🌐 {self.address_string()} - {format % args}")

def start_server(port=8080):
    """Start the DeskAgent Web GUI server"""
    
    # Change to the web-gui directory
    web_gui_dir = Path(__file__).parent
    os.chdir(web_gui_dir)
    
    print(f"🚀 Starting DeskAgent Web GUI Server")
    print(f"📁 Serving from: {web_gui_dir}")
    print(f"🌐 Port: {port}")
    
    try:
        with socketserver.TCPServer(("", port), DeskAgentHTTPRequestHandler) as httpd:
            server_url = f"http://localhost:{port}"
            print(f"✅ Server running at: {server_url}")
            print(f"🔗 Open in browser: {server_url}")
            print("Press Ctrl+C to stop the server")
            
            # Try to open browser automatically
            try:
                webbrowser.open(server_url)
                print("🌐 Browser opened automatically")
            except Exception as e:
                print(f"⚠️  Could not open browser automatically: {e}")
                print(f"   Please open {server_url} manually")
            
            # Start serving
            httpd.serve_forever()
            
    except KeyboardInterrupt:
        print("\n🛑 Server stopped by user")
    except OSError as e:
        if e.errno == 48:  # Address already in use
            print(f"❌ Port {port} is already in use")
            print("Try a different port or stop the existing server")
        else:
            print(f"❌ Server error: {e}")
    except Exception as e:
        print(f"❌ Unexpected error: {e}")

if __name__ == "__main__":
    import sys
    
    port = 8080
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            print("❌ Invalid port number")
            sys.exit(1)
    
    start_server(port)