use crate::state::{self, Credentials};
use anyhow::{bail, ensure, Context, Result};
use reqwest::{blocking::Client, redirect::Policy, Url};
use serde_json::{json, Value};
use std::{
    fs,
    process::Command,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub fn server(input: &str) -> Result<String> {
    let url = Url::parse(input).context("Invalid --server origin")?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    ensure!(
        url.scheme() == "https" || (url.scheme() == "http" && local),
        "Server must use HTTPS (HTTP permitted only on loopback)"
    );
    ensure!(
        url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && url.path() == "/",
        "--server must be an origin without credentials, path, query or fragment"
    );
    Ok(url.as_str().trim_end_matches('/').into())
}
pub struct Api {
    pub server: String,
    client: Client,
    token: Option<String>,
}
impl Api {
    pub fn new(server: &str, authenticated: bool) -> Result<Self> {
        let token = if authenticated {
            let credential: Credentials = state::read(&state::token_path(server)?)?
                .context("Not logged in to this server. Run cargo proofs login --server URL")?;
            ensure!(
                credential.expires_at > epoch(),
                "Token expired; run cargo proofs login again"
            );
            Some(credential.access_token)
        } else {
            None
        };
        Ok(Self {
            server: server.into(),
            client: Client::builder()
                .add_root_certificate(reqwest::Certificate::from_pem(&fs::read("/usr/local/share/ca-certificates/environment-proxy-ca.crt")?)?)
                .timeout(Duration::from_secs(60))
                .redirect(Policy::none())
                .user_agent(concat!("cargo-proofs/", env!("CARGO_PKG_VERSION")))
                .build()?,
            token,
        })
    }
    pub fn raw(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        key: Option<&str>,
    ) -> Result<(u16, Value)> {
        ensure!(
            path.starts_with('/') && !path.starts_with("//"),
            "Invalid API path"
        );
        let mut req = self
            .client
            .request(method.parse()?, format!("{}{}", self.server, path));
        if let Some(token) = &self.token {
            req = req.bearer_auth(token);
        }
        if let Some(body) = body {
            req = req.json(body);
        }
        if let Some(key) = key {
            req = req.header("Idempotency-Key", key);
        }
        let response = req.send().context("Service request failed")?;
        let status = response.status().as_u16();
        let body = response
            .json::<Value>()
            .context("Expected a JSON response from proofs.rs; check --server")?;
        Ok((status, body))
    }
    pub fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        key: Option<&str>,
    ) -> Result<Value> {
        let (status, value) = self.raw(method, path, body, key)?;
        if !(200..300).contains(&status) {
            let hint = match status {
                401 => " Run cargo proofs login again.",
                428 => " Accept the updated terms in the browser, then retry.",
                429 => " Rate limit reached; retry later.",
                _ => "",
            };
            bail!(
                "HTTP {status}: {} — {}.{hint}",
                value["error"].as_str().unwrap_or("request_failed"),
                value["message"].as_str().unwrap_or("")
            );
        }
        Ok(value)
    }
    pub fn upload(&self, path: &str, bytes: &[u8]) -> Result<Value> {
        ensure!(path.starts_with("/api/v1/runs/"), "Invalid artifact path");
        let response = self
            .client
            .post(format!("{}{}", self.server, path))
            .bearer_auth(self.token.as_ref().context("Not logged in")?)
            .header("Content-Type", "application/octet-stream")
            .body(bytes.to_vec())
            .send()?;
        ensure!(
            response.status().is_success(),
            "Artifact upload failed: {}",
            response.text()?
        );
        response.json().context("Expected artifact upload JSON response")
    }
    pub fn get(&self, path: &str) -> Result<Value> {
        self.request("GET", path, None, None)
    }
    pub fn prepare(&self, name: &str, version: &str) -> Result<()> {
        let mut job = self.request(
            "POST",
            "/api/v1/publish/prepare",
            Some(&json!({"crate":name,"version":version})),
            None,
        )?;
        let start = Instant::now();
        while job["status"] != "ready" {
            ensure!(
                job["status"] != "failed",
                "API catalogue import failed: {}",
                job["error_code"]
            );
            ensure!(
                start.elapsed() < Duration::from_secs(600),
                "API catalogue import is still pending; retry publish later"
            );
            let id = job["id"].as_str().context("Missing import job ID")?;
            eprintln!("Preparing API catalogue ({})…", job["status"]);
            thread::sleep(Duration::from_secs(3));
            job = self.get(&format!("/api/v1/imports/{id}"))?;
        }
        Ok(())
    }
    pub fn tool_version(&self, name: &str, version: &str) -> Result<String> {
        let all = self.get("/api/v1/tools")?;
        let tools = all["items"].as_array().context("Missing tool catalogue")?;
        let tool = tools
            .iter()
            .find(|t| {
                (t["name"]
                    .as_str()
                    .is_some_and(|s| s.eq_ignore_ascii_case(name))
                    || t["id"] == name)
                    && enabled(&t["active"])
            })
            .context("Tool is not active/registered on this server")?;
        let matches: Vec<_> = all["versions"]
            .as_array()
            .context("Missing tool versions")?
            .iter()
            .filter(|v| {
                v["tool_id"] == tool["id"] && v["version"] == version && enabled(&v["selectable"])
            })
            .collect();
        ensure!(matches.len() == 1, "{name} version {version} is not registered/selectable on this server; request it before publishing");
        Ok(matches[0]["id"]
            .as_str()
            .context("Invalid tool version ID")?
            .into())
    }
    pub fn resolve(&self, name: &str, version: &str, paths: &[String]) -> Result<(String, String)> {
        let mut found = std::collections::BTreeMap::new();
        for path in paths {
            let mut url = Url::parse(&format!("{}/api/v1/resolve-api", self.server))?;
            url.query_pairs_mut().extend_pairs([
                ("crate", name),
                ("version", version),
                ("path", path),
            ]);
            let (status, value) = self.raw(
                "GET",
                &format!("{}?{}", url.path(), url.query().unwrap()),
                None,
                None,
            )?;
            if status == 404 {
                continue;
            }
            ensure!(
                status == 200,
                "API resolution failed (HTTP {status}): {}",
                value["error"]
            );
            let id = value["id"].as_str().context("Missing API ID")?.to_owned();
            found.insert(id, path.clone());
        }
        ensure!(found.len() == 1, "Expected one public API match for {:?}, found {}. Check release, features, and reexports.", paths, found.len());
        Ok(found.into_iter().next().unwrap())
    }
}
fn enabled(v: &Value) -> bool {
    v == true || v == 1
}
fn epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn login(server: &str, no_browser: bool) -> Result<()> {
    let api = Api::new(server, false)?;
    let d = api.request(
        "POST",
        "/auth/device/code",
        Some(&json!({"client_id":"proofs-cli","scope":"publish"})),
        None,
    )?;
    let verification = d["verification_uri_complete"]
        .as_str()
        .or(d["verification_uri"].as_str())
        .context("Missing authorization URL")?;
    let uri = Url::parse(verification)?;
    ensure!(
        uri.origin() == Url::parse(server)?.origin(),
        "Server returned an authorization URL on another origin"
    );
    println!(
        "Open {verification}\nCode: {}",
        d["user_code"].as_str().context("Missing user code")?
    );
    if !no_browser {
        #[cfg(target_os = "macos")]
        let result = Command::new("open").arg(verification).spawn();
        #[cfg(target_os = "windows")]
        let result = Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", verification])
            .spawn();
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let result = Command::new("xdg-open").arg(verification).spawn();
        if result.is_err() {
            eprintln!("Open the URL above in your browser to authorize.");
        }
    }
    let mut interval = d["interval"].as_u64().unwrap_or(5).max(1);
    let deadline = Instant::now() + Duration::from_secs(d["expires_in"].as_u64().unwrap_or(600));
    let body = json!({"client_id":"proofs-cli","device_code":d["device_code"],"grant_type":"urn:ietf:params:oauth:grant-type:device_code"});
    loop {
        ensure!(
            Instant::now() + Duration::from_secs(interval) < deadline,
            "Device code expired; run login again"
        );
        thread::sleep(Duration::from_secs(interval));
        let (status, token) = api.raw("POST", "/auth/device/token", Some(&body), None)?;
        if status == 200 {
            let credentials = Credentials {
                access_token: token["access_token"]
                    .as_str()
                    .context("Missing token")?
                    .into(),
                expires_at: epoch()
                    + token["expires_in"]
                        .as_u64()
                        .context("Missing token expiry")?,
            };
            state::write(&state::token_path(server)?, &credentials)?;
            println!("Logged in to {server}.");
            return Ok(());
        }
        match token["error"].as_str() {
            Some("authorization_pending") => {}
            Some("slow_down") => interval = (interval + 5).min(60),
            Some("terms_required") => {
                bail!("Accept current terms in the browser, then run login again")
            }
            _ => bail!(
                "Device authorization failed: {}. Run login again.",
                token["error"]
            ),
        }
    }
}
pub fn logout(server: &str) -> Result<()> {
    let path = state::token_path(server)?;
    let Some(credentials) = state::read::<Credentials>(&path)? else {
        println!("Already logged out.");
        return Ok(());
    };
    if credentials.expires_at > epoch() {
        let api = Api::new(server, true)?;
        let (status, value) = api.raw("POST", "/api/v1/tokens/revoke", Some(&json!({})), None)?;
        ensure!(
            status == 200 || status == 401,
            "Could not revoke token (HTTP {status}): {}. Local token retained so you can retry.",
            value["error"]
        );
    }
    fs::remove_file(path)?;
    println!("Logged out of {server}.");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn server_origins() {
        assert_eq!(server("https://proofs.rs/").unwrap(), "https://proofs.rs");
        assert!(server("http://example.com").is_err());
        assert!(server("https://example.com/path").is_err());
        assert!(server("http://127.0.0.1:1234").is_ok());
        assert!(server("https://token@example.com").is_err());
    }
    #[test]
    fn http_errors_and_idempotency_header() {
        let srv = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let origin = format!("http://{}", srv.server_addr());
        let worker = std::thread::spawn(move || {
            let req = srv.recv().unwrap();
            assert!(req
                .headers()
                .iter()
                .any(|h| h.field.equiv("Idempotency-Key") && h.value.as_str() == "same-key"));
            req.respond(
                tiny_http::Response::from_string(r#"{"error":"terms_required"}"#)
                    .with_status_code(428),
            )
            .unwrap();
        });
        let api = Api::new(&origin, false).unwrap();
        let error = api
            .request(
                "POST",
                "/api/v1/reports",
                Some(&json!({})),
                Some("same-key"),
            )
            .unwrap_err();
        assert!(error.to_string().contains("terms"));
        worker.join().unwrap();
    }
}
