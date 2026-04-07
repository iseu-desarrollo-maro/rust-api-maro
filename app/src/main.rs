use serde::{Deserialize, Serialize};
use reqwest::{Client, Response, header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT}};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct GraphQLquery {
    query: String,
}

#[derive(Deserialize, Debug)]
struct GitHubResponse {
    data: Option<Data>,
    errors: Option<Vec<GraphQLError>>, 
}

#[derive(Deserialize, Debug)]
struct Data {
    viewer: Viewer,
}

#[derive(Deserialize, Debug)]
struct Viewer {
    login: String,
    name: Option<String>,
    bio: Option<String>,
    url: String,
}

#[derive(Deserialize, Debug)]
struct GraphQLError {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Es recomendable usar variables de entorno para almacenar el token por seguridad
        // Puedes ejecutarlo con: GITHUB_TOKEN=tu_token cargo run
    let token: String = env::var("GITHUB_TOKEN").expect("Por favor, establece la variable de entorno GITHUB_TOKEN");
    
    let client: Client = reqwest::Client::new();

let mut headers: HeaderMap = HeaderMap::new();
headers.insert(USER_AGENT, HeaderValue::from_static("rust-api-maro"));
headers.insert(
    AUTHORIZATION,
    HeaderValue::from_str(&format!("Bearer {}", token))?,
);

    let query: GraphQLquery = GraphQLquery {
        query: "{ viewer { login name bio url } }".to_string(),
    };

    let response: Response = client
        .post("https://api.github.com/graphql")
        .headers(headers)
        .json(&query)
        .send()
        .await?;

    let github_data: GitHubResponse = response.json().await?;

    if let Some(errors) = github_data.errors {
        eprintln!("Errores de la API de GraphQL:");
        for error in errors {
            eprintln!("- {}", error.message);
        }
    } else if let Some(data) = github_data.data {
        let viewer: Viewer = data.viewer;
        print!("---Datos del Usuario GitHub---\n");
        println!("Login: {}", viewer.login);
        println!("Nombre: {}", viewer.name.unwrap_or("N/A".to_string()));
        println!("Bio: {}", viewer.bio.unwrap_or("Sin biografia".to_string()));
        println!("URL: {}", viewer.url);
    } 

    Ok(())
}