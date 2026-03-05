use schemars::JsonSchema;
use serde::Deserialize;
use zed::settings::ContextServerSettings;
use zed_extension_api::{
    self as zed, http_client, serde_json, Command, ContextServerConfiguration, ContextServerId,
    Project, Result, SlashCommand, SlashCommandArgumentCompletion, SlashCommandOutput,
    SlashCommandOutputSection, Worktree,
};

const MCP_PACKAGE: &str = "@agentdeskai/browser-tools-mcp";
const PACKAGE_VERSION: &str = "1.2.1";
const DEFAULT_PORT: u16 = 3025;
const DEFAULT_HOST: &str = "127.0.0.1";

#[derive(Debug, Deserialize, JsonSchema)]
struct BrowserToolsSettings {
    /// Port for the browser-tools-server (default: 3025)
    #[serde(default = "default_port")]
    port: u16,
    /// Host for the browser-tools-server (default: 127.0.0.1)
    #[serde(default = "default_host")]
    host: String,
    /// Override the server command (default: npx)
    #[serde(default = "default_server_command")]
    server_command: String,
    /// Override the server arguments (default: [@agentdeskai/browser-tools-mcp@<version>])
    #[serde(default = "default_server_args")]
    server_args: Vec<String>,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_host() -> String {
    DEFAULT_HOST.to_string()
}

fn default_server_command() -> String {
    "/bin/sh".to_string()
}

fn default_server_args() -> Vec<String> {
    // Uses /bin/sh to detect user's $SHELL, then launches it as login shell
    // with the appropriate rc file sourced (for node version managers like
    // nvm, fnm, proto, volta that modify PATH in shell configs).
    // grep --line-buffered filters stdout to only pass JSON-RPC messages
    // (browser-tools-mcp writes debug messages to stdout, breaking MCP protocol).
    let npx_cmd = format!(
        "exec npx -y {}@{} | grep --line-buffered '^{{'",
        MCP_PACKAGE, PACKAGE_VERSION
    );
    vec![
        "-c".to_string(),
        format!(
            r#"S="${{SHELL:-/bin/sh}}"; case "$S" in */zsh) exec "$S" -l -c "source ~/.zshrc 2>/dev/null; {npx_cmd}";; */bash) exec "$S" -l -c "source ~/.bashrc 2>/dev/null; {npx_cmd}";; *) exec "$S" -l -c "{npx_cmd}";; esac"#
        ),
    ]
}

struct BrowserToolsExtension {
    port: u16,
    host: String,
}

impl zed::Extension for BrowserToolsExtension {
    fn new() -> Self {
        Self {
            port: DEFAULT_PORT,
            host: DEFAULT_HOST.to_string(),
        }
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let settings = ContextServerSettings::for_project("browser-tools-context-server", project)?;
        let settings_value = settings.settings.unwrap_or_else(|| serde_json::json!({}));

        let settings: BrowserToolsSettings =
            serde_json::from_value(settings_value).unwrap_or_else(|_| BrowserToolsSettings {
                port: default_port(),
                host: default_host(),
                server_command: default_server_command(),
                server_args: default_server_args(),
            });

        self.port = settings.port;
        self.host = settings.host.clone();

        Ok(Command {
            command: settings.server_command,
            args: settings.server_args,
            env: vec![
                ("PORT".into(), settings.port.to_string()),
                ("HOST".into(), settings.host),
            ],
        })
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();
        let default_settings = include_str!("../configuration/default_settings.jsonc").to_string();
        let settings_schema = serde_json::to_string(&schemars::schema_for!(BrowserToolsSettings))
            .map_err(|e| e.to_string())?;

        Ok(Some(ContextServerConfiguration {
            installation_instructions,
            default_settings,
            settings_schema,
        }))
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "browser-capture" => Ok(vec![
                completion("Screenshot", "screenshot"),
                completion("Console Logs", "logs"),
                completion("Console Errors", "errors"),
                completion("Network Logs", "network"),
                completion("Network Errors", "network-errors"),
                completion("Clear Logs", "clear"),
                completion("DOM Element", "element"),
            ]),
            "browser-audit" => Ok(vec![
                completion("Accessibility", "accessibility"),
                completion("Performance", "performance"),
                completion("SEO", "seo"),
                completion("Best Practices", "best-practices"),
                completion("NextJS", "nextjs"),
                completion("Run All Audits", "all"),
            ]),
            "browser-debug" => Ok(vec![completion("Start Debugger Mode", "start")]),
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let arg = args.first().map(|s| s.as_str()).unwrap_or("");
        if arg.is_empty() {
            return Err("No argument provided. Please select an option.".to_string());
        }

        let (endpoint, method, body) = resolve_api_call(command.name.as_str(), arg)?;
        let url = format!("http://{}:{}/{}", self.host, self.port, endpoint);
        let text = execute_request(&url, &method, body, command.name.as_str(), arg)?;
        let label = section_label(command.name.as_str(), arg);

        Ok(SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..text.len()).into(),
                label: label.to_string(),
            }],
            text,
        })
    }
}

fn completion(label: &str, value: &str) -> SlashCommandArgumentCompletion {
    SlashCommandArgumentCompletion {
        label: label.to_string(),
        new_text: value.to_string(),
        run_command: true,
    }
}

fn timestamp_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn resolve_api_call(
    cmd: &str,
    arg: &str,
) -> Result<(String, String, serde_json::Value), String> {
    let ts = timestamp_millis();
    let post = "POST".to_string();
    let get = "GET".to_string();
    let empty = serde_json::json!({});

    match (cmd, arg) {
        ("browser-capture", "screenshot") => Ok(("capture-screenshot".into(), post, empty)),
        ("browser-capture", "logs") => Ok(("console-logs".into(), get, empty)),
        ("browser-capture", "errors") => Ok(("console-errors".into(), get, empty)),
        ("browser-capture", "network") => Ok(("network-success".into(), get, empty)),
        ("browser-capture", "network-errors") => Ok(("network-errors".into(), get, empty)),
        ("browser-capture", "clear") => Ok(("wipelogs".into(), post, empty)),
        ("browser-capture", "element") => Ok(("selected-element".into(), get, empty)),

        ("browser-audit", audit) => {
            let endpoint = match audit {
                "accessibility" => "accessibility-audit",
                "performance" => "performance-audit",
                "seo" => "seo-audit",
                "best-practices" => "best-practices-audit",
                "nextjs" => "nextjs-audit",
                "all" => "audit-all",
                _ => return Err(format!("Unknown audit type: {audit}")),
            };
            let body = serde_json::json!({
                "category": audit,
                "source": "zed_extension",
                "timestamp": ts
            });
            Ok((endpoint.into(), post, body))
        }

        ("browser-debug", "start") => Ok((
            "debug-mode".into(),
            post,
            serde_json::json!({ "source": "zed_extension", "timestamp": ts }),
        )),

        _ => Err(format!("Unknown command: {cmd} {arg}")),
    }
}

fn section_label<'a>(cmd: &'a str, arg: &'a str) -> &'a str {
    match (cmd, arg) {
        ("browser-capture", "screenshot") => "Browser Screenshot",
        ("browser-capture", "logs") => "Browser Console Logs",
        ("browser-capture", "errors") => "Browser Console Errors",
        ("browser-capture", "network") => "Browser Network Logs",
        ("browser-capture", "network-errors") => "Browser Network Errors",
        ("browser-capture", "clear") => "Clear Logs",
        ("browser-capture", "element") => "DOM Element",
        ("browser-audit", "accessibility") => "Accessibility Audit",
        ("browser-audit", "performance") => "Performance Audit",
        ("browser-audit", "seo") => "SEO Audit",
        ("browser-audit", "best-practices") => "Best Practices Audit",
        ("browser-audit", "nextjs") => "NextJS Audit",
        ("browser-audit", "all") => "All Audits",
        ("browser-debug", "start") => "Debugger Mode",
        _ => "Browser Tools",
    }
}

fn execute_request(
    url: &str,
    method: &str,
    body: serde_json::Value,
    cmd: &str,
    arg: &str,
) -> Result<String, String> {
    match http_call(url, method, &body) {
        Ok(response) => parse_response(&response, url),
        Err(e) => {
            let action = match cmd {
                "browser-capture" => match arg {
                    "screenshot" => "capture screenshot",
                    "logs" => "retrieve console logs",
                    "errors" => "retrieve console errors",
                    "network" | "network-errors" => "retrieve network logs",
                    "clear" => "clear logs",
                    "element" => "get DOM element",
                    _ => "execute command",
                },
                "browser-audit" => "run audit",
                "browser-debug" => "start debugger",
                _ => "execute command",
            };
            Err(format!(
                "Failed to {}. Make sure BrowserTools Chrome extension is running. Error: {}",
                action, e
            ))
        }
    }
}

fn http_call(url: &str, method: &str, body: &serde_json::Value) -> Result<String, String> {
    let request = if method == "POST" {
        let json = serde_json::to_string(body)
            .map_err(|e| format!("JSON serialization error: {}", e))?;
        http_client::HttpRequest::builder()
            .method(http_client::HttpMethod::Post)
            .url(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json.into_bytes())
            .build()
    } else {
        http_client::HttpRequest::builder()
            .method(http_client::HttpMethod::Get)
            .url(url)
            .header("Accept", "application/json")
            .build()
    };

    let response = request
        .map_err(|e| format!("Failed to build request: {}", e))?
        .fetch()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    String::from_utf8(response.body)
        .map_err(|e| format!("Invalid UTF-8 in response: {}", e))
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    status: String,
    #[serde(default)]
    data: serde_json::Value,
    #[serde(default)]
    message: String,
}

fn parse_response(raw: &str, url: &str) -> Result<String, String> {
    let endpoint = url.rsplit('/').next().unwrap_or("");

    if let Ok(api) = serde_json::from_str::<ApiResponse>(raw) {
        if api.status == "success" {
            return Ok(format_response(endpoint, api.data));
        }
        return Ok(format!("Error from BrowserTools: {}", api.message));
    }

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(raw) {
        return Ok(format_response(endpoint, json));
    }

    Ok(format!("Raw response from BrowserTools: {}", raw))
}

fn format_response(endpoint: &str, data: serde_json::Value) -> String {
    match endpoint {
        "capture-screenshot" => {
            if data.get("message").and_then(|v| v.as_str()).is_some() {
                "Successfully saved screenshot".to_string()
            } else {
                format!(
                    "Screenshot captured: {}",
                    serde_json::to_string_pretty(&data).unwrap_or_default()
                )
            }
        }
        "console-logs" | "console-errors" => format_console_logs(&data),
        "network-success" | "network-errors" => {
            format!(
                "Network logs:\n\n{}",
                serde_json::to_string_pretty(&data).unwrap_or_default()
            )
        }
        "wipelogs" => data
            .get("message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Browser logs cleared successfully.".to_string()),
        "selected-element" => format_selected_element(&data),
        ep if ep.ends_with("-audit") => format_audit(ep, &data),
        "audit-all" => format!(
            "Audit Mode Results:\n\n{}",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        ),
        "debug-mode" => format!(
            "Debugger Mode Results:\n\n{}",
            serde_json::to_string_pretty(&data).unwrap_or_default()
        ),
        _ => serde_json::to_string_pretty(&data).unwrap_or_default(),
    }
}

fn format_console_logs(data: &serde_json::Value) -> String {
    let Some(logs) = data.as_array() else {
        return format!(
            "Console logs: {}",
            serde_json::to_string_pretty(data).unwrap_or_default()
        );
    };

    if logs.is_empty() {
        return "No console logs found.".to_string();
    }

    let formatted: Vec<String> = logs
        .iter()
        .map(|log| {
            let level = log.get("level").and_then(|v| v.as_str()).unwrap_or("info");
            let message = log.get("message").and_then(|v| v.as_str()).unwrap_or("");
            format!("[{}] {}", level.to_uppercase(), message)
        })
        .collect();

    format!("Console Logs:\n\n{}", formatted.join("\n"))
}

fn format_selected_element(data: &serde_json::Value) -> String {
    let Some(element) = data.get("element") else {
        return "No DOM element selected. Click on an element in the browser to select it."
            .to_string();
    };

    let tag = element.get("tagName").and_then(|v| v.as_str()).unwrap_or("unknown");
    let mut info = format!("Selected DOM Element:\n- Tag: {}", tag);

    if let Some(id) = element.get("id").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        info.push_str(&format!("\n- ID: {}", id));
    }
    if let Some(cls) = element.get("className").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        info.push_str(&format!("\n- Classes: {}", cls));
    }
    if let Some(text) = element.get("innerText").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        info.push_str(&format!("\n- Text: {}", text));
    }
    if let Some(html) = element.get("outerHTML").and_then(|v| v.as_str()) {
        info.push_str(&format!("\n\nHTML:\n{}", html));
    }

    info
}

fn format_audit(endpoint: &str, data: &serde_json::Value) -> String {
    let audit_type = match endpoint {
        "accessibility-audit" => "Accessibility",
        "performance-audit" => "Performance",
        "seo-audit" => "SEO",
        "best-practices-audit" => "Best Practices",
        "nextjs-audit" => "NextJS",
        _ => "Unknown",
    };

    let score = data
        .get("score")
        .and_then(|v| v.as_f64())
        .map(|s| format!("Overall Score: {}%\n", (s * 100.0).round() as i32))
        .unwrap_or_default();

    let issues = match data.get("issues").and_then(|v| v.as_array()) {
        Some(issues) if issues.is_empty() => "\nNo issues found!".to_string(),
        Some(issues) => {
            let mut text = "\nIssues Found:\n".to_string();
            for (i, issue) in issues.iter().enumerate() {
                let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown issue");
                let desc = issue.get("description").and_then(|v| v.as_str()).unwrap_or("");
                text.push_str(&format!("\n{}. {}\n", i + 1, title));
                if !desc.is_empty() {
                    text.push_str(&format!("   {}\n", desc));
                }
            }
            text
        }
        None => String::new(),
    };

    if !score.is_empty() || !issues.is_empty() {
        format!("{} Audit Results:\n\n{}{}", audit_type, score, issues)
    } else {
        format!(
            "{} Audit Results:\n\n{}",
            audit_type,
            serde_json::to_string_pretty(data).unwrap_or_default()
        )
    }
}

zed::register_extension!(BrowserToolsExtension);
