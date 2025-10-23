import asyncio
from playwright.async_api import async_playwright
import http.server
import socketserver
import threading
import os

PORT = 8000
os.chdir('activity-tracker/dist')
Handler = http.server.SimpleHTTPRequestHandler
httpd = socketserver.TCPServer(("", PORT), Handler)

def run_server():
    print(f"Serving at port {PORT}")
    httpd.serve_forever()

server_thread = threading.Thread(target=run_server)
server_thread.daemon = True
server_thread.start()

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch()
        page = await browser.new_page()
        await page.goto(f"http://localhost:{PORT}")
        await page.screenshot(path="jules-scratch/verification/verification.png")
        await browser.close()
        httpd.shutdown()

asyncio.run(main())
