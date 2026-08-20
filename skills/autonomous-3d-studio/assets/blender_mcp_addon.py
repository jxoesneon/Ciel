# blender_mcp_addon.py - CIEL 3D Studio Live Bridge Addon for Blender (v2.0.0)
# Hardened with Cryptographic Token Authentication and Length-Prefixed JSON-RPC 2.0 Framing

bl_info = {
    "name": "CIEL 3D Studio Live Bridge",
    "author": "CIEL Autonomous 3D Studio",
    "version": (2, 0, 0),
    "blender": (3, 0, 0),
    "location": "View3D > Sidebar > CIEL Studio",
    "description": "Authenticated live TCP/MCP bridge enabling autonomous agent interaction with Blender viewport and scene graph",
    "category": "3D View",
}

import bpy
import os
import json
import socket
import struct
import secrets
import threading
import traceback
import queue
import io
import sys

SERVER_HOST = "127.0.0.1"
SERVER_PORT = 9876
server_running = False
server_thread = None
command_queue = queue.Queue()
response_dict = {}
response_lock = threading.Lock()

def get_or_create_auth_token():
    """Generates or retrieves cryptographic bridge auth token stored in ~/.ciel/.bridge_token (0600 permissions)."""
    env_token = os.environ.get("CIEL_BRIDGE_AUTH_TOKEN")
    if env_token:
        return env_token.strip()

    ciel_dir = os.path.expanduser("~/.ciel")
    os.makedirs(ciel_dir, exist_ok=True)
    token_file = os.path.join(ciel_dir, ".bridge_token")

    if os.path.exists(token_file):
        try:
            with open(token_file, 'r', encoding='utf-8') as f:
                token = f.read().strip()
                if token:
                    return token
        except Exception:
            pass

    # Generate new 256-bit cryptographic token
    new_token = secrets.token_hex(32)
    try:
        with open(token_file, 'w', encoding='utf-8') as f:
            f.write(new_token)
        os.chmod(token_file, 0o600)
    except Exception:
        pass
    return new_token

AUTH_TOKEN = get_or_create_auth_token()

def execute_on_main_thread(code_str):
    """Executes python code in Blender's main context and captures stdout/errors."""
    old_stdout = sys.stdout
    old_stderr = sys.stderr
    redirected_output = io.StringIO()
    redirected_error = io.StringIO()
    sys.stdout = redirected_output
    sys.stderr = redirected_error

    exec_result = {"status": "SUCCESS", "stdout": "", "stderr": "", "return_value": None}
    try:
        scope = {"bpy": bpy}
        exec(code_str, scope)
        exec_result["stdout"] = redirected_output.getvalue()
        exec_result["stderr"] = redirected_error.getvalue()
    except Exception as e:
        exec_result["status"] = "ERROR"
        exec_result["stderr"] = traceback.format_exc()
    finally:
        sys.stdout = old_stdout
        sys.stderr = old_stderr

    return exec_result

def process_command_queue():
    while not command_queue.empty():
        req_id, code_str = command_queue.get()
        res = execute_on_main_thread(code_str)
        with response_lock:
            response_dict[req_id] = res
    return 0.05

def send_framed_msg(sock, data_dict):
    """Sends 4-byte length-prefixed JSON message."""
    payload = json.dumps(data_dict).encode('utf-8')
    header = struct.pack("!I", len(payload))
    sock.sendall(header + payload)

def recv_framed_msg(sock):
    """Receives 4-byte length-prefixed JSON message."""
    header_data = sock.recv(4)
    if not header_data or len(header_data) < 4:
        return None
    msg_len = struct.unpack("!I", header_data)[0]
    # Safety ceiling: max 64MB per message
    if msg_len > 64 * 1024 * 1024:
        raise ValueError("Message length exceeds 64MB safety limit")
    
    chunks = []
    bytes_recd = 0
    while bytes_recd < msg_len:
        chunk = sock.recv(min(msg_len - bytes_recd, 65536))
        if not chunk:
            break
        chunks.append(chunk)
        bytes_recd += len(chunk)
    return json.loads(b"".join(chunks).decode('utf-8'))

def tcp_server_worker():
    global server_running
    server_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    
    try:
        server_sock.bind((SERVER_HOST, SERVER_PORT))
        server_sock.listen(5)
        server_sock.settimeout(1.0)
        print(f"[CIEL 3D Studio] Authenticated Live TCP Bridge active on {SERVER_HOST}:{SERVER_PORT}")

        req_counter = 0
        while server_running:
            try:
                client_sock, client_addr = server_sock.accept()
            except socket.timeout:
                continue
            except Exception:
                break

            try:
                client_sock.settimeout(30.0)
                # Keep-alive loop for persistent connections
                while server_running:
                    req = recv_framed_msg(client_sock)
                    if req is None:
                        break

                    # Mandatory Cryptographic Token Authentication
                    client_token = req.get("auth_token", "")
                    if not secrets.compare_digest(client_token, AUTH_TOKEN):
                        send_framed_msg(client_sock, {
                            "status": "UNAUTHORIZED",
                            "error": "Invalid or missing CIEL_BRIDGE_AUTH_TOKEN"
                        })
                        break

                    req_id = req.get("id", f"req_{req_counter}")
                    req_counter += 1
                    action = req.get("action", "execute_code")

                    if action == "execute_code":
                        code = req.get("code", "")
                        command_queue.put((req_id, code))

                        import time
                        start_t = time.time()
                        resp = None
                        while time.time() - start_t < 30.0:
                            with response_lock:
                                if req_id in response_dict:
                                    resp = response_dict.pop(req_id)
                                    break
                            time.sleep(0.02)

                        if resp is None:
                            resp = {"status": "TIMEOUT", "error": "Execution timed out on Blender main thread."}

                        send_framed_msg(client_sock, resp)

                    elif action == "get_scene_summary":
                        obs = []
                        for obj in bpy.data.objects:
                            obs.append({
                                "name": obj.name,
                                "type": obj.type,
                                "visible": obj.visible_get(),
                                "location": list(obj.location),
                                "dimensions": list(obj.dimensions),
                                "polycount": len(obj.data.polygons) if obj.type == 'MESH' else 0
                            })
                        resp = {"status": "SUCCESS", "objects": obs, "active_object": bpy.context.active_object.name if bpy.context.active_object else None}
                        send_framed_msg(client_sock, resp)

            except Exception as e:
                try:
                    send_framed_msg(client_sock, {"status": "ERROR", "error": str(e)})
                except Exception:
                    pass
            finally:
                client_sock.close()

    finally:
        server_sock.close()

class CIEL_OT_StartServer(bpy.types.Operator):
    bl_idname = "ciel.start_server"
    bl_label = "Start CIEL Live Bridge"
    bl_description = "Starts authenticated TCP socket server for live AI agent interaction"

    def execute(self, context):
        global server_running, server_thread
        if not server_running:
            server_running = True
            server_thread = threading.Thread(target=tcp_server_worker, daemon=True)
            server_thread.start()
            bpy.app.timers.register(process_command_queue)
            self.report({'INFO'}, f"CIEL Authenticated Bridge running on port {SERVER_PORT}")
        return {'FINISHED'}

class CIEL_OT_StopServer(bpy.types.Operator):
    bl_idname = "ciel.stop_server"
    bl_label = "Stop CIEL Live Bridge"
    bl_description = "Stops the live AI agent TCP socket server"

    def execute(self, context):
        global server_running
        server_running = False
        if bpy.app.timers.is_registered(process_command_queue):
            bpy.app.timers.unregister(process_command_queue)
        self.report({'INFO'}, "CIEL Bridge stopped.")
        return {'FINISHED'}

class CIEL_PT_StudioPanel(bpy.types.Panel):
    bl_label = "CIEL 3D Studio Bridge"
    bl_idname = "CIEL_PT_StudioPanel"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = 'CIEL Studio'

    def draw(self, context):
        layout = self.layout
        col = layout.column(align=True)
        global server_running
        if not server_running:
            col.operator("ciel.start_server", icon='PLAY')
            col.label(text="Status: Offline", icon='CANCEL')
        else:
            col.operator("ciel.stop_server", icon='PAUSE')
            col.label(text=f"Status: Online ({SERVER_PORT})", icon='CHECKMARK')

def register():
    bpy.utils.register_class(CIEL_OT_StartServer)
    bpy.utils.register_class(CIEL_OT_StopServer)
    bpy.utils.register_class(CIEL_PT_StudioPanel)

def unregister():
    global server_running
    server_running = False
    if bpy.app.timers.is_registered(process_command_queue):
        bpy.app.timers.unregister(process_command_queue)
    bpy.utils.unregister_class(CIEL_OT_StartServer)
    bpy.utils.unregister_class(CIEL_OT_StopServer)
    bpy.utils.unregister_class(CIEL_PT_StudioPanel)

if __name__ == "__main__":
    register()
