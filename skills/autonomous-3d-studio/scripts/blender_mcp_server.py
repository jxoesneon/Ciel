#!/usr/bin/env python3
"""
blender_mcp_server.py - CIEL 3D Studio Authenticated Live MCP & Socket RPC Bridge

Connects to a running Blender instance (via the CIEL Live Bridge Addon on port 9876)
with mandatory token authentication and length-prefixed framing.
"""

import sys
import os
import json
import socket
import struct
import argparse

DEFAULT_HOST = "127.0.0.1"
DEFAULT_PORT = 9876

def get_auth_token():
    """Reads cryptographic token from environment or ~/.ciel/.bridge_token."""
    env_tok = os.environ.get("CIEL_BRIDGE_AUTH_TOKEN")
    if env_tok:
        return env_tok.strip()
    
    token_file = os.path.expanduser("~/.ciel/.bridge_token")
    if os.path.exists(token_file):
        try:
            with open(token_file, 'r', encoding='utf-8') as f:
                return f.read().strip()
        except Exception:
            pass
    return ""

def send_framed_msg(sock, data_dict):
    payload = json.dumps(data_dict).encode('utf-8')
    header = struct.pack("!I", len(payload))
    sock.sendall(header + payload)

def recv_framed_msg(sock):
    header_data = sock.recv(4)
    if not header_data or len(header_data) < 4:
        return None
    msg_len = struct.unpack("!I", header_data)[0]
    if msg_len > 64 * 1024 * 1024:
        raise ValueError("Message length exceeds 64MB limit")

    chunks = []
    bytes_recd = 0
    while bytes_recd < msg_len:
        chunk = sock.recv(min(msg_len - bytes_recd, 65536))
        if not chunk:
            break
        chunks.append(chunk)
        bytes_recd += len(chunk)
    return json.loads(b"".join(chunks).decode('utf-8'))

def send_rpc_request(action, payload, host=DEFAULT_HOST, port=DEFAULT_PORT, timeout=30.0):
    """Sends authenticated length-prefixed JSON-RPC request to Blender bridge."""
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.settimeout(timeout)
    try:
        s.connect((host, port))
        token = get_auth_token()
        req = {
            "action": action,
            "auth_token": token
        }
        req.update(payload)
        send_framed_msg(s, req)
        resp = recv_framed_msg(s)
        if resp is None:
            return {"status": "ERROR", "error": "Empty or closed response from Blender socket."}
        return resp
    except ConnectionRefusedError:
        return {
            "status": "UNAVAILABLE",
            "error": f"Could not connect to Blender on {host}:{port}. Ensure Blender is open and CIEL Live Bridge is running."
        }
    except socket.timeout:
        return {"status": "TIMEOUT", "error": f"Blender RPC timed out after {timeout} seconds."}
    except Exception as e:
        return {"status": "ERROR", "error": str(e)}
    finally:
        s.close()

def execute_bpy_live(code_str, host=DEFAULT_HOST, port=DEFAULT_PORT):
    return send_rpc_request("execute_code", {"code": code_str}, host=host, port=port)

def get_live_scene_summary(host=DEFAULT_HOST, port=DEFAULT_PORT):
    return send_rpc_request("get_scene_summary", {}, host=host, port=port)

def capture_viewport_frame(out_png_path, host=DEFAULT_HOST, port=DEFAULT_PORT):
    escaped_out = json.dumps(os.path.abspath(out_png_path))
    code = f"""
import bpy
bpy.context.scene.render.image_settings.file_format = 'PNG'
bpy.context.scene.render.filepath = {escaped_out}
bpy.ops.render.opengl(write_still=True)
print(f"[Live Bridge] Viewport frame saved to: {{{escaped_out}}}")
"""
    return execute_bpy_live(code, host=host, port=port)

def main():
    parser = argparse.ArgumentParser(description="CIEL 3D Studio Authenticated Live Blender MCP Bridge")
    parser.add_argument("--exec", "-e", help="Python/bpy code string to execute in live Blender")
    parser.add_argument("--file", "-f", help="Python file to execute in live Blender")
    parser.add_argument("--summary", "-s", action="store_true", help="Get live scene summary")
    parser.add_argument("--screenshot", help="Capture live viewport screenshot to target PNG")
    parser.add_argument("--host", default=DEFAULT_HOST, help="Blender host IP")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help="Blender bridge port")

    args = parser.parse_args()

    if args.summary:
        res = get_live_scene_summary(host=args.host, port=args.port)
        print(json.dumps(res, indent=2))
        return

    if args.screenshot:
        res = capture_viewport_frame(args.screenshot, host=args.host, port=args.port)
        print(json.dumps(res, indent=2))
        return

    if args.file:
        if not os.path.exists(args.file):
            print(f"Error: File '{args.file}' not found.", file=sys.stderr)
            sys.exit(1)
        with open(args.file, 'r', encoding='utf-8') as f:
            code = f.read()
        res = execute_bpy_live(code, host=args.host, port=args.port)
        print(json.dumps(res, indent=2))
        return

    if args.exec:
        res = execute_bpy_live(args.exec, host=args.host, port=args.port)
        print(json.dumps(res, indent=2))
        return

    parser.print_help()

if __name__ == "__main__":
    main()
