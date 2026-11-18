//! Recursos corporativos do painel. Nunca substituem o ícone de um site.

use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::Response;

/// A lista fechada evita acesso ao filesystem e não depende do Auth ou do banco.
/// O despachante chama este recurso somente no host configurado do painel.
pub(crate) fn recurso(caminho: &str, metodo: &Method) -> Option<Response> {
    let (bytes, tipo): (&'static [u8], &'static str) = match caminho {
        "/favicon.svg" | "/recursos/brand/favicon.svg" => (
            include_bytes!("../recursos/brand/favicon.svg"),
            "image/svg+xml",
        ),
        "/favicon.ico" | "/recursos/brand/favicon.ico" => (
            include_bytes!("../recursos/brand/favicon.ico"),
            "image/vnd.microsoft.icon",
        ),
        "/favicon-96x96.png" | "/recursos/brand/favicon-96x96.png" => (
            include_bytes!("../recursos/brand/favicon-96x96.png"),
            "image/png",
        ),
        "/apple-touch-icon.png" | "/recursos/brand/apple-touch-icon.png" => (
            include_bytes!("../recursos/brand/apple-touch-icon.png"),
            "image/png",
        ),
        "/recursos/brand/web-app-manifest-192x192.png" => (
            include_bytes!("../recursos/brand/web-app-manifest-192x192.png"),
            "image/png",
        ),
        "/recursos/brand/web-app-manifest-512x512.png" => (
            include_bytes!("../recursos/brand/web-app-manifest-512x512.png"),
            "image/png",
        ),
        "/recursos/brand/web-app-manifest-maskable-512x512.png" => (
            include_bytes!("../recursos/brand/web-app-manifest-maskable-512x512.png"),
            "image/png",
        ),
        "/recursos/brand/site.webmanifest" => (
            include_bytes!("../recursos/brand/site.webmanifest"),
            "application/manifest+json",
        ),
        _ => return None,
    };
    let mut resposta = Response::new(Body::empty());
    if *metodo != Method::GET && *metodo != Method::HEAD {
        *resposta.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        resposta
            .headers_mut()
            .insert("allow", HeaderValue::from_static("GET, HEAD"));
        return Some(resposta);
    }
    if *metodo == Method::GET {
        *resposta.body_mut() = Body::from(bytes);
    }
    resposta
        .headers_mut()
        .insert("content-type", HeaderValue::from_static(tipo));
    resposta.headers_mut().insert(
        "content-length",
        bytes.len().to_string().parse().expect("tamanho fixo"),
    );
    resposta.headers_mut().insert(
        "cache-control",
        HeaderValue::from_static("public, max-age=3600"),
    );
    resposta.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    Some(resposta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, StatusCode};
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn favicon_e_publico_e_o_head_nao_entrega_corpo() {
        let get = recurso("/favicon-96x96.png", &Method::GET).expect("ícone conhecido");
        assert_eq!(get.status(), StatusCode::OK);
        assert_eq!(get.headers()["content-type"], "image/png");
        assert_eq!(get.headers()["x-content-type-options"], "nosniff");
        let bytes = get.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 96);
        let head = recurso("/favicon-96x96.png", &Method::HEAD).unwrap();
        assert_eq!(head.headers()["content-length"], bytes.len().to_string());
        assert!(
            head.into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .is_empty()
        );
    }

    #[test]
    fn caminho_e_metodo_nao_escolhem_arquivos_do_servidor() {
        assert!(recurso("/recursos/brand/../../.env", &Method::GET).is_none());
        assert!(recurso("/recursos/brand/inexistente.png", &Method::GET).is_none());
        let resposta = recurso("/favicon.ico", &Method::POST).unwrap();
        assert_eq!(resposta.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(resposta.headers()["allow"], "GET, HEAD");
    }

    #[tokio::test]
    async fn manifesto_preserva_escopo_do_painel_e_aponta_icones_existentes() {
        let resposta = recurso("/recursos/brand/site.webmanifest", &Method::GET).unwrap();
        let bytes = resposta.into_body().collect().await.unwrap().to_bytes();
        let manifesto: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(manifesto["name"], "CMS • Avila Ops");
        assert_eq!(manifesto["start_url"], "/painel");
        assert_eq!(manifesto["scope"], "/painel");
        for icone in manifesto["icons"].as_array().unwrap() {
            assert!(recurso(icone["src"].as_str().unwrap(), &Method::GET).is_some());
        }
    }
}
