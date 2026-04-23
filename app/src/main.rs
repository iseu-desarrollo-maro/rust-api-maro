use actix_web::{web, App, HttpServer, Result as ActixResult, HttpResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use chrono_tz::America::Mexico_City;

#[derive(Serialize, Deserialize)]
struct Member {
    login: String,
    avatar_url: String,
    html_url: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    login: String,
    id: u64,
    avatar_url: String,
    html_url: String,
    name: Option<String>,
    company: Option<String>,
    location: Option<String>,
    bio: Option<String>,
}

#[derive(Serialize)]
struct InfoResponse {
    user_agent: String,
    time: String,
    name: String,
}

#[derive(Deserialize)]
struct TokenQuery {
    token: String,
    org: Option<String>,
}

async fn miembros(query: web::Query<TokenQuery>, client: web::Data<Client>) -> ActixResult<HttpResponse> {
    let TokenQuery { token, org } = query.into_inner();
    let org_name = org.unwrap_or_else(|| std::env::var("GITHUB_ORG").unwrap_or_else(|_| "iseu-desarrollo-maro".to_string()));
    let members_url = format!("https://api.github.com/orgs/{}/members?filter=all", org_name);

    let members_resp = client
        .get(&members_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "mi-proyecto-enpoint")
        .send()
        .await;

    match members_resp {
        Ok(r) => {
            if r.status().is_success() {
                r.json::<Vec<Member>>().await
                    .map(|members| HttpResponse::Ok().json(members))
                    .map_err(|e| {
                        eprintln!("Error de parseo de miembros: {}", e);
                        serde_json::json!({ "error": "Error al parsear JSON de miembros" })
                    }).or_else(|err| Ok(HttpResponse::InternalServerError().json(err)))
            } else {
                Ok(HttpResponse::Unauthorized().json(
                    serde_json::json!({ "error": "Token inválido o miembros no encontrados" }),
                ))
            }
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json(
            serde_json::json!({ "error": "Error de conexión con GitHub" }),
        )),
    }
}

async fn datos_usuario(query: web::Query<TokenQuery>, client: web::Data<Client>) -> ActixResult<HttpResponse> {
    let TokenQuery { token, .. } = query.into_inner();
    let user_url = "https://api.github.com/user";

    let user_resp = client
        .get(user_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "mi-proyecto-enpoint")
        .send()
        .await;

    match user_resp {
        Ok(r) => {
            if r.status().is_success() {
                match r.json::<User>().await {
                    Ok(user) => Ok(HttpResponse::Ok().json(user)),
                    Err(_) => Ok(HttpResponse::InternalServerError().json(
                        serde_json::json!({ "error": "Error al parsear JSON de usuario" }),
                    )),
                }
            } else {
                Ok(HttpResponse::Unauthorized().json(
                    serde_json::json!({ "error": "Token inválido o usuario no encontrado" }),
                ))
            }
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json(
            serde_json::json!({ "error": "Error de conexión con GitHub" }),
        )),
    }
}

async fn info(req: actix_web::HttpRequest) -> ActixResult<HttpResponse> {
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let time = Utc::now()
        .with_timezone(&Mexico_City)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let name = "mi-proyecto-enpoint".to_string();
    let response = InfoResponse { user_agent, time, name };
    Ok(HttpResponse::Ok().json(response))
}

async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().body("OK"))
}

async fn welcome() -> ActixResult<HttpResponse> {
    // Obtenemos la URL del frontend de una variable de entorno, 
    // En Azure, esta variable debe ser la URL de tu App Service (https://...)
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5001".to_string());

    Ok(HttpResponse::Found()
        .append_header(("Location", frontend_url))
        .finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::new();
    
    let port = std::env::var("PORT").unwrap_or_else(|_| "7001".to_string());
    let host = "0.0.0.0";
    let addr = format!("{}:{}", host, port);
    let frontend_url = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5001".to_string());

    println!("\n🚀 Servidor API escuchando en todas las interfaces ({}).", addr);
    println!("👉 Accede localmente en: http://localhost:{}", port);
    println!("🔗 Redirección configurada hacia: {}\n", frontend_url);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .route("/", web::get().to(welcome))
            .route("/health", web::get().to(health_check))
            .route("/miembros", web::get().to(miembros))
            .route("/datosUsuario", web::get().to(datos_usuario))
            .route("/info", web::get().to(info))
    });

    println!("🚀 Intentando enlazar servidor en {}...", addr);
    
    server.bind(&addr)
        .map_err(|e| {
            eprintln!("❌ Error fatal: No se pudo enlazar el puerto {}. Detalles: {}", port, e);
            e
        })?
        .run()
        .await?;

    Ok(())
}
