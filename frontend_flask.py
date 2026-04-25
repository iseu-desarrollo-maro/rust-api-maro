from flask import Flask, render_template, request, session, redirect, url_for
import requests
import os
from datetime import datetime
import json # Import json for potential error parsing

app = Flask(__name__, static_folder='app/static')
app.secret_key = os.getenv("FLASK_SECRET_KEY", "dev_key_para_local")

BASE_URL = os.getenv("API_URL", "http://127.0.0.1:7001")
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
            response = requests.get(url, params=params, headers=headers)
        elif method == "POST":
            response = requests.post(url, params=params, json=data, headers=headers)
        else:
            return None, "Método HTTP no soportado", 500

        # Verifica si la respuesta HTTP fue exitosa (código 2xx)
        if not response.ok: 
            try:
                error_data = response.json()
                error_msg = error_data.get("error", f"Error del servidor: {response.status_code}")
            except json.JSONDecodeError:
                error_msg = f"Error del servidor ({response.status_code}): {response.text}"
            return None, error_msg, response.status_code

        # Si fue exitosa, intenta parsear el JSON
        return response.json(), None, response.status_code

    except requests.exceptions.ConnectionError:
        return None, "Error de conexión con el servidor backend", 500
    except Exception as e:
        return None, f"Un error inesperado ocurrió: {e}", 500

@app.context_processor
def inject_now():
    return {'server_date': datetime.now().strftime("%d/%m/%Y")}

@app.route("/")
def home():
    return render_template("index.html")

@app.route("/logout")
def logout():
    session.clear()
    return redirect(url_for("home"))

@app.route("/info", methods=["GET"])
def info():
    data, error, _ = call_backend_api("info")
    # Asegúrate de pasar un diccionario vacío si data es None para evitar errores en la plantilla
    return render_template("info.html", data=data if data else {}, error=error)

@app.route("/datosUsuario", methods=["GET", "POST"])
def datos_usuario():
    if request.method == "POST":
        session["token"] = request.form.get("token")
        return redirect(url_for("datos_usuario"))
    
    token = session.get("token")
    if not token:
        return render_template("datos_usuario.html", data={}, error="Debes ingresar un token")
    
    # El backend espera el token en la cabecera Authorization
    headers = {"Authorization": f"Bearer {token}"}
    data, error, _ = call_backend_api("datosUsuario", headers=headers)
    return render_template("datos_usuario.html", data=data if data else {}, error=error)

@app.route("/usuarios", methods=["GET", "POST"])
def usuarios():
    if request.method == "POST":
        session["token"] = request.form.get("token")
        org = request.form.get("org", ORG_NAME)
        return redirect(url_for("usuarios", org=org))
    
    org = request.args.get("org", session.get("org", ORG_NAME))
    return render_template("usuarios.html", org_name=org)

@app.route("/api/miembros")
def api_miembros():
    token = session.get("token")
    org = request.args.get("org", ORG_NAME)
    if not token:
        return {"error": "No hay token en la sesión"}, 401
    
    # El backend espera el token en la cabecera Authorization y la organización en los parámetros de la consulta
    headers = {"Authorization": f"Bearer {token}"}
    params = {"org": org}
    data, error, status_code = call_backend_api("miembros", params=params, headers=headers)
    
    if error:
        return {"error": error}, status_code
    return data, status_code, {'Content-Type': 'application/json'}

if __name__ == "__main__":
    port = int(os.getenv("PORT", 80))
    app.run(host="0.0.0.0", port=port)
