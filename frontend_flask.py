from flask import Flask, render_template, request, session, redirect, url_for
import requests
import os
from datetime import datetime
import json  # Import json for potential error parsing

app = Flask(__name__, static_folder='app/static')
app.secret_key = os.getenv("FLASK_SECRET_KEY", "una-clave-muy-segura-y-secreta")

BASE_URL = os.getenv("API_URL", "http://backend:7001").rstrip('/')
ORG_NAME = os.getenv("GITHUB_ORG", "iseu-desarrollo-maro").strip()

def call_backend_api(endpoint, method="GET", params=None, data=None, headers=None):
    """
    Función auxiliar para realizar llamadas a la API del backend.
    Maneja errores de conexión, códigos de estado HTTP y el parseo de JSON.
    Retorna una tupla: (data, error_message, status_code)
    """
    url = f"{BASE_URL}/{endpoint}"
    try:
        if method == "GET":
            response = requests.get(url, params=params, headers=headers, timeout=10)
        elif method == "POST":
            response = requests.post(url, params=params, json=data, headers=headers, timeout=10)
        else:
            return None, "Método HTTP no soportado", 500

        if not response.ok: 
            try:
                error_data = response.json()
                error_msg = error_data.get("error", f"Error del servidor: {response.status_code}")
            except json.JSONDecodeError:
                error_msg = f"Error del servidor ({response.status_code}): {response.text}"
            return None, error_msg, response.status_code

        return response.json(), None, response.status_code

    except requests.exceptions.ConnectionError:
        return None, f"Error de conexión con el servidor backend en {url}. ¿Está el servidor Rust encendido?", 500
    except Exception as e:
        return None, f"Un error inesperado ocurrió: {e}", 500

@app.context_processor
def inject_now():
    return {'server_date': datetime.now().strftime("%d/%m/%Y")}

@app.route("/")
def home():
    return render_template("index.html")

@app.route("/health")
def health():
    """Ruta de salud para el WarmUp Probe de Azure"""
    return "OK", 200

@app.route("/info", methods=["GET"])
def info():
    data, error, _ = call_backend_api("info")
    return render_template("info.html", data=data if data else {}, error=error)

@app.route("/datosUsuario", methods=["GET", "POST"])
def datos_usuario():
    if request.method == "POST":
        # Guardamos el token en la sesión
        session["token"] = request.form.get("token")
        return redirect(url_for("datos_usuario"))
    
    # Recuperamos el token de la sesión o del query string
    token = session.get("token") or request.args.get("token")
    if not token:
        return render_template("datos_usuario.html", data={}, error="Debes ingresar un token")
    
    # El backend espera el token en la cabecera Authorization
    headers = {"Authorization": f"Bearer {token}"}
    data, error, _ = call_backend_api("datosUsuario", headers=headers)
    return render_template("datos_usuario.html", data=data if data else {}, error=error)

@app.route("/api/miembros")
def api_miembros():
    token = session.get("token") or os.getenv("GITHUB_TOKEN")
    org = request.args.get("org", os.getenv("GITHUB_ORG"))

    if not token:
        return {"error": "No hay token en la sesión"}, 401

    headers = {"Authorization": f"Bearer {token}"}
    params = {"org": org}

    data, error, status_code = call_backend_api("miembros", params=params, headers=headers)
    return (data if data else {"error": error}), status_code, {'Content-Type': 'application/json'}

@app.route("/api/datosUsuario")
def api_datos_usuario():
    token = session.get("token") or os.getenv("GITHUB_TOKEN")

    if not token:
        return {"error": "No hay token en la sesión"}, 401

    headers = {"Authorization": f"Bearer {token}"}
    data, error, status_code = call_backend_api("datosUsuario", headers=headers)
    return (data if data else {"error": error}), status_code, {'Content-Type': 'application/json'}

# Este bloque va al final del archivo, fuera de cualquier función
if __name__ == "__main__":
    port = int(os.environ.get("PORT", 80))
    app.run(host="0.0.0.0", port=port)
