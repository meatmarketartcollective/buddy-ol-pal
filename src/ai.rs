//! Pluggable AI voice for the buddy.
//!
//! `tb ai setup` walks through NVIDIA's free hosted models by default
//! (OpenAI-compatible endpoint, free credits at signup, no credit card). Any
//! other OpenAI-compatible endpoint (OpenAI, Groq, Ollama, LM Studio, vLLM,
//! OpenRouter, ...) or an Anthropic endpoint works by editing ai.toml or with
//! setup flags.
//!
//! Privacy mirrors the event ledger: requests carry only the buddy's name,
//! species, mood and the last event's kind/exit code/duration, never file
//! contents, paths or secrets, and nothing is logged. The key stays in
//! ai.toml (chmod 600 on setup) or an env var; it is never printed.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const NVIDIA_KEY_URL: &str = "https://build.nvidia.com/settings/api-keys";
const NVIDIA_BASE: &str = "https://integrate.api.nvidia.com/v1";
const OPENAI_BASE: &str = "https://api.openai.com/v1";
const ANTHROPIC_BASE: &str = "https://api.anthropic.com/v1";
const NVIDIA_MODEL: &str = "google/gemma-3-4b-it";
const OPENAI_MODEL: &str = "gpt-4o-mini";
const ANTHROPIC_MODEL: &str = "claude-3-5-haiku-latest";
const DEFAULT_TIMEOUT_SECS: u64 = 6;
const MAX_LINE_CHARS: usize = 140;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiConfig {
    /// "nvidia" (the free default) | "openai" | "anthropic", or any base URL
    /// for a custom OpenAI-compatible endpoint (ollama, LM Studio, vLLM...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The key itself. Stays in ai.toml (chmod 600 on setup); never printed,
    /// logged, or sent anywhere except its own endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Alternative: read the key from this env var instead of the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
    /// Overrides the endpoint's default base URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Capability tier: 1 text only, 2 tool calls, 3 full agent incl. edits.
    /// Unset means "whatever the probe granted"; 3 is never probed, only set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tier: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm_writes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirm_destructive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask_max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ask_timeout_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
    OpenAiCompat,
    Anthropic,
}

struct Resolved {
    provider: Provider,
    base_url: String,
    model: String,
    key: Option<String>,
    max_tokens: u32,
    timeout: Duration,
}

impl Resolved {
    fn ready(&self) -> bool {
        self.key.is_some() && !self.model.is_empty()
    }
}

fn default_env_name(endpoint: Option<&str>) -> &'static str {
    match endpoint {
        Some("openai") => "OPENAI_API_KEY",
        Some("anthropic") => "ANTHROPIC_API_KEY",
        _ => "NVIDIA_API_KEY",
    }
}

pub fn resolve(cfg: &AiConfig) -> Resolved {
    let (provider, base) = match cfg.endpoint.as_deref() {
        Some("anthropic") => (Provider::Anthropic, ANTHROPIC_BASE),
        Some("openai") => (Provider::OpenAiCompat, OPENAI_BASE),
        Some("nvidia") | None => (Provider::OpenAiCompat, NVIDIA_BASE),
        // Any other endpoint string is treated as a custom
        // OpenAI-compatible base URL (ollama, LM Studio, vLLM, openrouter...).
        Some(other) => (Provider::OpenAiCompat, other),
    };
    let base_url = cfg
        .base_url
        .clone()
        .unwrap_or_else(|| base.to_string());
    let key = cfg.api_key.clone().or_else(|| {
        let env_name = cfg
            .api_key_env
            .clone()
            .unwrap_or_else(|| default_env_name(cfg.endpoint.as_deref()).to_string());
        std::env::var(&env_name)
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
    });
    Resolved {
        provider,
        base_url,
        model: cfg.model.clone().unwrap_or_default(),
        key,
        max_tokens: cfg.max_tokens.unwrap_or(64),
        timeout: Duration::from_secs(cfg.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS)),
    }
}

/// What the buddy can say about itself. Deliberately small: no paths, no
/// command text, no secrets.
pub struct Voice<'a> {
    pub pet_name: &'a str,
    pub species: &'a str,
    pub mood: &'a str,
    pub last_event: Option<(&'a str, i32, u64)>,
}

/// One line from the configured endpoint, or None on any failure so the
/// caller falls back to the species' own lines.
pub fn speak(dir: &Path, v: &Voice) -> Option<String> {
    let cfg = load_config(dir)?;
    if cfg.enabled == Some(false) {
        return None;
    }
    let r = resolve(&cfg);
    if !r.ready() {
        return None;
    }
    let (url, body) = build_body(&r, v)?;
    let text = http_post(&r, &url, &body)?;
    let line = parse_line(&text, r.provider)?;
    sanitize(&line)
}

fn build_body(r: &Resolved, v: &Voice) -> Option<(String, String)> {
    let sys = format!(
        "You are {}, a tiny {} living in the user's terminal. Reply with ONE short line, 12 words max, playful, lowercase, no emoji, no quotes.",
        v.pet_name, v.species
    );
    let usr = match v.last_event {
        Some((kind, code, secs)) => format!(
            "You feel {}. Just happened: {} (exit {}, {}s). Say hi back.",
            v.mood, kind, code, secs
        ),
        None => format!("You feel {}. Say hi back.", v.mood),
    };
    match r.provider {
        Provider::OpenAiCompat => {
            let body = serde_json::json!({
                "model": r.model,
                "max_tokens": r.max_tokens,
                "temperature": 0.9,
                "messages": [
                    {"role": "system", "content": sys},
                    {"role": "user", "content": usr}
                ]
            });
            Some((
                format!("{}/chat/completions", r.base_url.trim_end_matches('/')),
                body.to_string(),
            ))
        }
        Provider::Anthropic => {
            let body = serde_json::json!({
                "model": r.model,
                "max_tokens": r.max_tokens,
                "system": sys,
                "messages": [{"role": "user", "content": usr}]
            });
            Some((
                format!("{}/messages", r.base_url.trim_end_matches('/')),
                body.to_string(),
            ))
        }
    }
}

/// One-line diagnostics to stderr, only when HATCH_AI_DEBUG is set.
/// Normal runs stay silent: unreachable endpoints fall back quietly.
fn ai_debug(msg: &str) {
    if std::env::var_os("HATCH_AI_DEBUG").is_some() {
        eprintln!("hatch ai: {msg}");
    }
}

fn http_post(r: &Resolved, url: &str, body: &str) -> Option<String> {
    match http_post_err(r, url, body, r.timeout) {
        Ok(t) => Some(t),
        Err(e) => {
            ai_debug(&e);
            None
        }
    }
}

/// Honest variant for ask/agent: returns the real failure reason instead of
/// silently falling back. `timeout` overrides the voice timeout.
fn http_post_err(
    r: &Resolved,
    url: &str,
    body: &str,
    timeout: Duration,
) -> Result<String, String> {
    let agent = ureq::AgentBuilder::new().timeout(timeout).build();
    let mut req = agent.post(url).set("Content-Type", "application/json");
    let key = r
        .key
        .as_ref()
        .ok_or_else(|| "no API key found".to_string())?;
    match r.provider {
        Provider::OpenAiCompat => {
            req = req.set("Authorization", &format!("Bearer {key}"));
        }
        Provider::Anthropic => {
            req = req.set("x-api-key", key);
            req = req.set("anthropic-version", "2023-06-01");
        }
    }
    let resp = req
        .send_string(body)
        .map_err(|e| format!("request to {url} failed: {e}"))?;
    let status = resp.status();
    if status != 200 {
        let detail = resp
            .into_string()
            .ok()
            .map(|t| {
                let t = t.trim();
                if t.chars().count() > 200 {
                    format!(": {:.200}", t)
                } else {
                    format!(": {t}")
                }
            })
            .unwrap_or_default();
        return Err(format!("{url} answered HTTP {status}{detail}"));
    }
    resp.into_string()
        .map_err(|e| format!("could not read {url} response body: {e}"))
}

fn parse_line(text: &str, provider: Provider) -> Option<String> {
    let j: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            ai_debug(&format!("response was not valid JSON: {e} (first 200 bytes: {:.200})", text));
            return None;
        }
    };
    match provider {
        Provider::OpenAiCompat => match j
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(str::to_string)
        {
            Some(l) => Some(l),
            None => {
                ai_debug("openai-compat response had no choices[0].message.content");
                None
            }
        },
        Provider::Anthropic => {
            let arr = j.get("content")?.as_array()?;
            for item in arr {
                if item.get("type")?.as_str()? == "text" {
                    return item.get("text")?.as_str().map(str::to_string);
                }
            }
            None
        }
    }
}

/// First line only, one pair of wrapped quotes stripped, clamped short.
fn sanitize(line: &str) -> Option<String> {
    let mut out = line.trim().lines().next().unwrap_or("").trim().to_string();
    while out.len() > 1
        && ((out.starts_with('"') && out.ends_with('"'))
            || (out.starts_with('\'') && out.ends_with('\'')))
    {
        out = out[1..out.len() - 1].trim().to_string();
    }
    if out.chars().count() > MAX_LINE_CHARS {
        out = out.chars().take(MAX_LINE_CHARS - 1).collect::<String>() + "\u{2026}";
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

// ---- capability tiers ----
// T1: text answers over the event log (tb ask). T2: tool-call-capable models
// unlock the agent's read/run tools. T3 is a deliberate user choice: set
// tier = 3 in ai.toml once the model can be trusted to edit files. The probe
// never grants T3 on its own.

pub fn tier_cache_path(dir: &Path) -> PathBuf {
    dir.join("ai_tier")
}

pub fn save_tier_cache(dir: &Path, tier: u8) -> std::io::Result<()> {
    fs::write(tier_cache_path(dir), tier.to_string())
}

pub fn cached_tier(dir: &Path) -> Option<u8> {
    fs::read_to_string(tier_cache_path(dir))
        .ok()?
        .trim()
        .parse::<u8>()
        .ok()
        .filter(|t| (1..=3).contains(t))
}

pub fn effective_tier(dir: &Path, cfg: &AiConfig) -> Option<u8> {
    cfg.tier
        .filter(|t| (1..=3).contains(t))
        .or_else(|| cached_tier(dir))
}

/// Full completion for ask/agent: honest errors, no silent fallback, and a
/// per-call timeout; when max_tokens is None the request carries no cap.
pub fn chat(
    r: &Resolved,
    msgs: &[(String, String)],
    max_tokens: Option<u32>,
    timeout: Duration,
) -> Result<String, String> {
    if !r.ready() {
        return Err("no API key found".to_string());
    }
    let (url, body) = chat_body(r, msgs, max_tokens);
    let text = http_post_err(r, &url, &body, timeout)?;
    parse_content(&text, r.provider)
}

fn chat_body(
    r: &Resolved,
    msgs: &[(String, String)],
    max_tokens: Option<u32>,
) -> (String, String) {
    match r.provider {
        Provider::OpenAiCompat => {
            let mut body = serde_json::json!({
                "model": r.model,
                "messages": msgs
                    .iter()
                    .map(|(role, c)| serde_json::json!({"role": role, "content": c}))
                    .collect::<Vec<_>>(),
            });
            if let Some(m) = max_tokens {
                body["max_tokens"] = serde_json::json!(m);
            }
            (
                format!("{}/chat/completions", r.base_url.trim_end_matches('/')),
                body.to_string(),
            )
        }
        Provider::Anthropic => {
            let sys: Vec<&String> = msgs
                .iter()
                .filter(|(role, _)| role == "system")
                .map(|(_, c)| c)
                .collect();
            let rest: Vec<serde_json::Value> = msgs
                .iter()
                .filter(|(role, _)| role != "system")
                .map(|(role, c)| serde_json::json!({"role": role, "content": c}))
                .collect();
            let mut body = serde_json::json!({ "model": r.model });
            if !sys.is_empty() {
                body["system"] = serde_json::json!(
                    sys.iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join("\n\n")
                );
            }
            body["messages"] = serde_json::json!(rest);
            // Anthropic requires max_tokens; unset means a roomy default.
            body["max_tokens"] = serde_json::json!(max_tokens.unwrap_or(4096));
            (
                format!("{}/messages", r.base_url.trim_end_matches('/')),
                body.to_string(),
            )
        }
    }
}

/// Like parse_line but honest: ask/agent must see the real failure.
fn parse_content(text: &str, provider: Provider) -> Result<String, String> {
    let j: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| {
            format!(
                "response was not valid JSON: {e} (first 200 bytes: {:.200})",
                text
            )
        })?;
    match provider {
        Provider::OpenAiCompat => j
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                "openai-compat response had no choices[0].message.content".to_string()
            }),
        Provider::Anthropic => {
            let arr = j
                .get("content")
                .and_then(|c| c.as_array())
                .ok_or_else(|| "anthropic response had no content array".to_string())?;
            for item in arr {
                if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                    return Ok(item
                        .get("text")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string());
                }
            }
            Err("anthropic response had no text block".to_string())
        }
    }
}

/// T2 probe: a tool-call-shaped round trip. Text-only models fail it and
/// stay tier 1; T3 is never granted by the probe, only pinned by the user.
fn probe_tool_call(r: &Resolved) -> bool {
    let msgs = vec![
        (
            "system".to_string(),
            "Reply ONLY with a JSON object: {\"action\":\"done\",\"message\":\"ok\",\"success\":true}"
                .to_string(),
        ),
        ("user".to_string(), "acknowledge".to_string()),
    ];
    match chat(r, &msgs, Some(48), Duration::from_secs(15)) {
        Ok(text) => parse_agent_probe(&text)
            .map(|v| v.get("action").is_some())
            .unwrap_or(false),
        Err(_) => false,
    }
}

fn parse_agent_probe(text: &str) -> Option<serde_json::Value> {
    let mut t = text.trim();
    if let Some(rest) = t.strip_prefix("```json") {
        t = rest;
    } else if let Some(rest) = t.strip_prefix("```") {
        t = rest;
    }
    if let Some(rest) = t.strip_suffix("```") {
        t = rest;
    }
    let t = t.trim();
    let start = t.find('{')?;
    let end = t.rfind('}')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&t[start..=end]).ok()
}

pub fn config_path(dir: &Path) -> PathBuf {
    dir.join("ai.toml")
}

pub fn load_config(dir: &Path) -> Option<AiConfig> {
    let text = fs::read_to_string(config_path(dir)).ok()?;
    match toml::from_str(&text) {
        Ok(c) => Some(c),
        Err(_) => None,
    }
}

fn save_config(dir: &Path, cfg: &AiConfig) -> std::io::Result<PathBuf> {
    let path = config_path(dir);
    fs::write(&path, toml::to_string(cfg).unwrap_or_default())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(path)
}

fn ping_body(r: &Resolved) -> (String, String) {
    let msg = serde_json::json!({"role": "user", "content": "Reply with the single word: ready"});
    match r.provider {
        Provider::OpenAiCompat => (
            format!("{}/chat/completions", r.base_url.trim_end_matches('/')),
            serde_json::json!({"model": r.model, "max_tokens": 8u32, "messages": [msg]}).to_string(),
        ),
        Provider::Anthropic => (
            format!("{}/messages", r.base_url.trim_end_matches('/')),
            serde_json::json!({"model": r.model, "max_tokens": 8u32, "messages": [msg]}).to_string(),
        ),
    }
}

fn ping(r: &Resolved) -> Result<(), String> {
    if !r.ready() {
        return Err("no API key found".to_string());
    }
    let (url, body) = ping_body(r);
    let text = http_post(r, &url, &body)
        .ok_or_else(|| "no answer (bad key, quota, or unreachable endpoint)".to_string())?;
    let line =
        parse_line(&text, r.provider).ok_or_else(|| "reply shape not recognized".to_string())?;
    if line.trim().is_empty() {
        Err("empty reply".to_string())
    } else {
        Ok(())
    }
}

pub fn ai_cmd(rest: &[String]) -> i32 {
    match rest.first().map(|s| s.as_str()) {
        None => status_cmd(),
        Some("setup") => setup_cmd(&rest[1..]),
        Some("test") => test_cmd(),
        Some("on") => toggle_cmd(true),
        Some("off") => toggle_cmd(false),
        Some("clear") => clear_cmd(),
        Some(other) => {
            eprintln!("unknown ai command: {other}\nusage: tb ai [setup|test|on|off|clear]");
            2
        }
    }
}

fn status_cmd() -> i32 {
    let dir = crate::state::config_dir();
    let path = config_path(&dir);
    if !path.exists() {
        println!("no AI voice configured; the buddy speaks its built-in lines.");
        println!(
            "`tb ai setup` walks you through a free NVIDIA key (no card), or edit {}.",
            path.display()
        );
        return 0;
    }
    let cfg = match load_config(&dir) {
        Some(c) => c,
        None => {
            println!(
                "{} exists but does not parse; fix or delete it, then run tb ai setup.",
                path.display()
            );
            return 1;
        }
    };
    let endpoint = cfg.endpoint.clone().unwrap_or_else(|| "nvidia".into());
    let model = cfg.model.clone().unwrap_or_else(|| "(none set)".into());
    let key = match (&cfg.api_key, &cfg.api_key_env) {
        (Some(_), _) => "stored in ai.toml (hidden)".to_string(),
        (None, Some(env)) => format!("from env ${env}"),
        (None, None) => format!("from env ${}", default_env_name(Some(&endpoint))),
    };
    let on = match cfg.enabled {
        Some(false) => "off",
        _ => "on",
    };
    println!("AI voice: {on}");
    println!("  endpoint: {endpoint}");
    println!("  model:    {model}");
    println!("  key:      {key}");
    let tier_line = match effective_tier(&dir, &cfg) {
        Some(t) if cfg.tier.is_some() => format!("{t} (pinned in ai.toml)"),
        Some(t) => format!("{t} (from the last tb ai test)"),
        None => "unprobed (run tb ai test)".to_string(),
    };
    println!("  tier:     {tier_line}");
    println!("  config:   {}", path.display());
    0
}

fn setup_cmd(rest: &[String]) -> i32 {
    let mut endpoint = "nvidia".to_string();
    let mut model = NVIDIA_MODEL.to_string();
    let mut base_url: Option<String> = None;
    let mut key_env: Option<String> = None;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--openai" => {
                endpoint = "openai".into();
                model = OPENAI_MODEL.into();
            }
            "--anthropic" => {
                endpoint = "anthropic".into();
                model = ANTHROPIC_MODEL.into();
            }
            "--url" => match rest.get(i + 1) {
                Some(v) => {
                    endpoint = v.clone();
                    base_url = Some(v.clone());
                    i += 1;
                }
                None => {
                    eprintln!("--url needs a value");
                    return 2;
                }
            },
            "--model" => match rest.get(i + 1) {
                Some(v) => {
                    model = v.clone();
                    i += 1;
                }
                None => {
                    eprintln!("--model needs a value");
                    return 2;
                }
            },
            "--env" => match rest.get(i + 1) {
                Some(v) => {
                    key_env = Some(v.clone());
                    i += 1;
                }
                None => {
                    eprintln!("--env needs a value");
                    return 2;
                }
            },
            other => {
                eprintln!("unknown setup flag: {other}\nusage: tb ai setup [--openai|--anthropic|--url U --model M] [--env NAME]");
                return 2;
            }
        }
        i += 1;
    }

    if endpoint == "nvidia" && key_env.is_none() {
        println!("hatch can give your buddy an AI voice. The free default is NVIDIA's");
        println!("hosted models: free credits when you sign up, no credit card.");
        println!();
        println!("  1. Open {NVIDIA_KEY_URL}");
        println!("  2. Sign in (email or GitHub works) and click Generate Key.");
        println!("  3. Copy the key (it starts with nvapi-) and paste it below.");
        println!();
    } else if endpoint == "openai" && key_env.is_none() {
        println!("Get a key at https://platform.openai.com/api-keys, then paste it below.");
    } else if endpoint == "anthropic" && key_env.is_none() {
        println!("Get a key at https://console.anthropic.com, then paste it below.");
    }

    let key = match &key_env {
        Some(name) => {
            println!("reading the key from env ${name} at call time; nothing is stored.");
            None
        }
        None => {
            print!("Paste the API key (input is hidden, press Enter to cancel): ");
            use std::io::Write;
            let _ = std::io::stdout().flush();
            match rpassword::read_password() {
                Ok(k) if !k.trim().is_empty() => Some(k.trim().to_string()),
                Ok(_) => {
                    println!("cancelled; nothing written.");
                    return 0;
                }
                Err(e) => {
                    eprintln!("couldn't read the key: {e}");
                    return 1;
                }
            }
        }
    };

    let cfg = AiConfig {
        endpoint: Some(endpoint.clone()),
        model: Some(model.clone()),
        api_key: key,
        api_key_env: key_env,
        base_url,
        max_tokens: Some(64),
        timeout_secs: Some(DEFAULT_TIMEOUT_SECS),
        enabled: Some(true),
        tier: None,
        confirm_writes: None,
        confirm_destructive: None,
        ask_max_tokens: None,
        ask_timeout_secs: None,
        agent_max_tokens: None,
        agent_timeout_secs: None,
    };
    let dir = crate::state::config_dir();
    let path = match save_config(&dir, &cfg) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("couldn't write {}: {e}", config_path(&dir).display());
            return 1;
        }
    };
    println!("saved {} (permissions 600).", path.display());

    let r = resolve(&cfg);
    match ping(&r) {
        Ok(()) => println!(
            "voice is live. `tb talk` to hear it. If the key ever runs dry, the buddy falls back to its built-in lines."
        ),
        Err(e) => {
            println!("the test call failed: {e}");
            println!("the buddy keeps its built-in lines until this works; check the key and run tb ai test.");
        }
    }
    0
}

fn test_cmd() -> i32 {
    let dir = crate::state::config_dir();
    let cfg = match load_config(&dir) {
        Some(c) => c,
        None => {
            println!("no AI voice configured; run tb ai setup first.");
            return 1;
        }
    };
    let r = resolve(&cfg);
    match ping(&r) {
        Ok(()) => {
            println!("ok. {} answered for model {}.", r.base_url, r.model);
            if let Some(t) = cfg.tier.filter(|t| (1..=3).contains(t)) {
                let _ = save_tier_cache(&dir, t);
                println!("tier pinned in ai.toml: {t}.");
                print_tier_help(t);
                return 0;
            }
            let tool = probe_tool_call(&r);
            let tier = if tool { 2 } else { 1 };
            let _ = save_tier_cache(&dir, tier);
            print_tier_help(tier);
            0
        }
        Err(e) => {
            println!("failed: {e}");
            println!("the buddy falls back to its built-in lines either way.");
            1
        }
    }
}

fn print_tier_help(t: u8) {
    match t {
        1 => println!("tier granted: 1 (text answers only). tb ask works; tool-call-capable models unlock tb agent."),
        2 => println!("tier granted: 2 (tool calls work). tb ask is live; tb agent can read files and run commands (writes need tier 3)."),
        _ => println!("tier pinned: 3. tb ask and tb agent are fully live, writes included."),
    }
}

fn toggle_cmd(on: bool) -> i32 {
    let dir = crate::state::config_dir();
    let cfg = match load_config(&dir) {
        Some(c) => c,
        None => {
            println!("no AI voice configured; run tb ai setup first.");
            return 1;
        }
    };
    let mut cfg = cfg;
    cfg.enabled = Some(on);
    match save_config(&dir, &cfg) {
        Ok(_) => {
            if on {
                println!("AI voice on.");
            } else {
                println!("AI voice off; the buddy uses its built-in lines.");
            }
            0
        }
        Err(e) => {
            eprintln!("couldn't rewrite {}: {e}", config_path(&dir).display());
            1
        }
    }
}

fn clear_cmd() -> i32 {
    let dir = crate::state::config_dir();
    match fs::remove_file(config_path(&dir)) {
        Ok(()) => {
            println!("AI config removed; the buddy speaks its built-in lines.");
            0
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("nothing to remove; no AI voice configured.");
            0
        }
        Err(e) => {
            eprintln!("couldn't remove {}: {e}", config_path(&dir).display());
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use tempfile::tempdir;

    fn voice<'a>() -> Voice<'a> {
        Voice {
            pet_name: "ember",
            species: "fox",
            mood: "celebrating",
            last_event: Some(("build_finished", 0, 12)),
        }
    }

    fn resolved(provider: Provider, base: &str) -> Resolved {
        Resolved {
            provider,
            base_url: base.to_string(),
            model: "m".to_string(),
            key: Some("sk-secret".to_string()),
            max_tokens: 64,
            timeout: Duration::from_secs(2),
        }
    }

    #[test]
    fn openai_body_carries_only_safe_facts() {
        let r = resolved(Provider::OpenAiCompat, "https://x/v1");
        let (url, body) = build_body(&r, &voice()).unwrap();
        assert_eq!(url, "https://x/v1/chat/completions");
        assert!(!body.contains("sk-secret"), "the key never enters a body");
        let j: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(j["model"], "m");
        let sys = j["messages"][0]["content"].as_str().unwrap();
        assert!(sys.contains("ember") && sys.contains("fox"));
        let usr = j["messages"][1]["content"].as_str().unwrap();
        assert!(usr.contains("celebrating") && usr.contains("build_finished"));
        assert!(!usr.contains('/'), "no paths in the prompt");
    }

    #[test]
    fn anthropic_body_uses_the_messages_shape() {
        let r = resolved(Provider::Anthropic, "https://y/v1");
        let (url, body) = build_body(&r, &voice()).unwrap();
        assert_eq!(url, "https://y/v1/messages");
        let j: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(j.get("system").is_some());
        assert_eq!(j["messages"][0]["role"], "user");
        assert!(j.get("temperature").is_none());
    }

    #[test]
    fn resolve_defaults_to_nvidia() {
        let cfg = AiConfig::default();
        let r = resolve(&cfg);
        assert_eq!(r.base_url, NVIDIA_BASE);
        assert_eq!(r.provider, Provider::OpenAiCompat);
        assert!(!r.ready(), "no key, not ready");
    }

    #[test]
    fn any_url_endpoint_is_openai_compatible() {
        let cfg = AiConfig {
            endpoint: Some("http://127.0.0.1:9999/v1".into()),
            model: Some("m".into()),
            ..Default::default()
        };
        let r = resolve(&cfg);
        assert_eq!(r.base_url, "http://127.0.0.1:9999/v1");
        assert_eq!(r.provider, Provider::OpenAiCompat);
    }

    #[test]
    fn parse_handles_both_shapes_and_garbage() {
        let openai = r#"{"choices":[{"message":{"content":" hi there "}}]}"#;
        assert_eq!(
            parse_line(openai, Provider::OpenAiCompat).as_deref(),
            Some(" hi there ")
        );
        let anthropic =
            r#"{"content":[{"type":"thinking","thinking":"x"},{"type":"text","text":" hi "}]}"#;
        assert_eq!(
            parse_line(anthropic, Provider::Anthropic).as_deref(),
            Some(" hi ")
        );
        assert!(parse_line("not json", Provider::OpenAiCompat).is_none());
        assert!(parse_line(r#"{"choices":[]}"#, Provider::OpenAiCompat).is_none());
    }

    #[test]
    fn sanitize_strips_quotes_and_first_line_only() {
        assert_eq!(
            sanitize("  \"hello from the cloud\"  ").as_deref(),
            Some("hello from the cloud")
        );
        assert_eq!(sanitize("one\ntwo").as_deref(), Some("one"));
        assert_eq!(sanitize("   \n  "), None);
    }

    #[test]
    fn missing_config_means_no_ai() {
        let dir = tempdir().unwrap();
        assert!(speak(dir.path(), &voice()).is_none());
    }

    #[test]
    fn disabled_config_means_no_ai() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("ai.toml"), "enabled = false\n").unwrap();
        assert!(speak(dir.path(), &voice()).is_none());
    }

    #[test]
    fn end_to_end_over_plain_http() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 4096];
            let n = sock.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            assert!(req.starts_with("POST /v1/chat/completions"));
            assert!(req.contains("Bearer test-key"));
            let body = r#"{"choices":[{"message":{"content":"  \"hello from the cloud\"  "}}]}"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            sock.write_all(resp.as_bytes()).unwrap();
        });
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("ai.toml"),
            format!(
                "endpoint = \"http://127.0.0.1:{port}/v1\"\nmodel = \"m\"\napi_key = \"test-key\"\n"
            ),
        )
        .unwrap();
        let line = speak(dir.path(), &voice()).unwrap();
        assert_eq!(line, "hello from the cloud");
    }

    #[test]
    fn server_error_falls_back() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let _ = sock.read(&mut [0u8; 4096]);
            let _ = sock.write_all(b"HTTP/1.1 500 Nope\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        });
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("ai.toml"),
            format!(
                "endpoint = \"http://127.0.0.1:{port}/v1\"\nmodel = \"m\"\napi_key = \"k\"\ntimeout_secs = 2\n"
            ),
        )
        .unwrap();
        assert!(speak(dir.path(), &voice()).is_none());
    }
}
