use serde::Deserialize;
use zed::settings::ContextServerSettings;
use zed_extension_api::{
    self as zed, http_client, serde_json, Command, ContextServerId, Project, Result, SlashCommand,
    SlashCommandArgumentCompletion, SlashCommandOutput, SlashCommandOutputSection, Worktree,
};

const DEFAULT_PORT: u16 = 3025;
const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_BROWSERTOOLS_NPX_COMMAND: &str = "@agentdeskai/browser-tools-server@1.2.0";

#[derive(Debug, Deserialize)]
struct BrowserToolsSettings {
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_npx_command")]
    npx_command: String,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_host() -> String {
    DEFAULT_HOST.to_string()
}

fn default_npx_command() -> String {
    DEFAULT_BROWSERTOOLS_NPX_COMMAND.to_string()
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    status: String,
    #[serde(default)]
    data: serde_json::Value,
    #[serde(default)]
    message: String,
}

struct BrowserToolsExtension {
    port: u16,
    host: String,
    npx_command: String,
}

impl zed::Extension for BrowserToolsExtension {
    fn new() -> Self {
        Self {
            port: DEFAULT_PORT,
            host: DEFAULT_HOST.to_string(),
            npx_command: DEFAULT_BROWSERTOOLS_NPX_COMMAND.to_string(),
        }
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let settings = ContextServerSettings::for_project("browser-tools-context-server", project)?;
        let settings = settings.settings.unwrap_or_else(|| serde_json::json!({}));

        let settings: BrowserToolsSettings =
            serde_json::from_value(settings).unwrap_or_else(|_| BrowserToolsSettings {
                port: default_port(),
                host: default_host(),
                npx_command: default_npx_command(),
            });

        self.port = settings.port;
        self.host = settings.host.clone();
        self.npx_command = settings.npx_command.clone();

        Ok(Command {
            command: "npx".to_string(),
            args: vec![settings.npx_command],
            env: vec![
                ("PORT".into(), settings.port.to_string()),
                ("HOST".into(), settings.host),
            ],
        })
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "browser-capture" => Ok(vec![
                create_completion("Screenshot", "screenshot"),
                create_completion("Console Logs", "logs"),
                create_completion("Console Errors", "errors"),
                create_completion("Network Logs", "network"),
                create_completion("Network Errors", "network-errors"),
                create_completion("Clear Logs", "clear"),
                create_completion("DOM Element", "element"),
            ]),
            "browser-audit" => Ok(vec![
                create_completion("Accessibility", "accessibility"),
                create_completion("Performance", "performance"),
                create_completion("SEO", "seo"),
                create_completion("Best Practices", "best-practices"),
                create_completion("NextJS", "nextjs"),
                create_completion("Run All Audits", "all"),
            ]),
            "browser-debug" => Ok(vec![
                create_completion("Start Debugger Mode", "start"),
            ]),
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let command_name = command.name.as_str();
        let arg = args.first().map(|s| s.as_str()).unwrap_or("");

        if arg.is_empty() {
            return Err("No argument provided. Please select an option.".to_string());
        }

        let (api_endpoint, method, api_params) = get_api_params(command_name, arg)?;
        let api_url = format!("http://{}:{}/{}", self.host, self.port, api_endpoint);
        let result_text = process_api_request(&api_url, &method, api_params, command_name, arg)?;
        let section_label = get_section_label(command_name, arg);

        Ok(SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..result_text.len()).into(),
                label: section_label.to_string(),
            }],
            text: result_text,
        })
    }
}

fn create_completion(label: &str, command: &str) -> SlashCommandArgumentCompletion {
    SlashCommandArgumentCompletion {
        label: label.to_string(),
        new_text: command.to_string(),
        run_command: true,
    }
}

fn get_current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn get_api_params(command_name: &str, arg: &str) -> Result<(String, String, serde_json::Value), String> {
    let timestamp = get_current_timestamp();

    match (command_name, arg) {
        ("browser-capture", "screenshot") => Ok(("capture-screenshot".to_string(), "POST".to_string(), serde_json::json!({}))),
        ("browser-capture", "logs") => Ok(("console-logs".to_string(), "GET".to_string(), serde_json::json!({}))),
        ("browser-capture", "errors") => Ok(("console-errors".to_string(), "GET".to_string(), serde_json::json!({}))),
        ("browser-capture", "network") => Ok(("network-success".to_string(), "GET".to_string(), serde_json::json!({}))),
        ("browser-capture", "network-errors") => Ok(("network-errors".to_string(), "GET".to_string(), serde_json::json!({}))),
        ("browser-capture", "clear") => Ok(("wipelogs".to_string(), "POST".to_string(), serde_json::json!({}))),
        ("browser-capture", "element") => Ok(("selected-element".to_string(), "GET".to_string(), serde_json::json!({}))),

        ("browser-audit", "accessibility") => Ok(("accessibility-audit".to_string(), "POST".to_string(), serde_json::json!({
            "category": "accessibility",
            "source": "zed_extension",
            "timestamp": timestamp
        }))),
        ("browser-audit", "performance") => Ok(("performance-audit".to_string(), "POST".to_string(), serde_json::json!({
            "category": "performance",
            "source": "zed_extension",
            "timestamp": timestamp
        }))),
        ("browser-audit", "seo") => Ok(("seo-audit".to_string(), "POST".to_string(), serde_json::json!({
            "category": "seo",
            "source": "zed_extension",
            "timestamp": timestamp
        }))),
        ("browser-audit", "best-practices") => Ok(("best-practices-audit".to_string(), "POST".to_string(), serde_json::json!({
            "category": "best-practices",
            "source": "zed_extension",
            "timestamp": timestamp
        }))),
        ("browser-audit", "nextjs") => Ok(("nextjs-audit".to_string(), "POST".to_string(), serde_json::json!({
            "source": "zed_extension",
            "timestamp": timestamp
        }))),
        ("browser-audit", "all") => Ok(("audit-all".to_string(), "POST".to_string(), serde_json::json!({
            "source": "zed_extension",
            "timestamp": timestamp
        }))),

        ("browser-debug", "start") => Ok(("debug-mode".to_string(), "POST".to_string(), serde_json::json!({
            "source": "zed_extension",
            "timestamp": timestamp
        }))),

        (command, arg) => Err(format!("Unknown command or argument: {command} {arg}")),
    }
}

fn get_section_label<'a>(command_name: &'a str, arg: &'a str) -> &'a str {
    match (command_name, arg) {
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
        _ => "Browser Tools"
    }
}

fn process_api_request(
    api_url: &str,
    method: &str,
    api_params: serde_json::Value,
    command_name: &str,
    arg: &str,
) -> Result<String, String> {
    match call_browsertools_api(api_url, method, &api_params) {
        Ok(response) => process_api_response(response, api_url),
        Err(e) => {
            let message = get_error_message(command_name, arg);
            Err(format!("{} Error: {}", message, e))
        }
    }
}

fn get_error_message(command_name: &str, arg: &str) -> String {
    match (command_name, arg) {
        ("browser-capture", "screenshot") =>
            "Failed to capture screenshot. Make sure BrowserTools extension is running in Chrome.".to_string(),
        ("browser-capture", "logs") =>
            "Failed to retrieve console logs. Make sure BrowserTools extension is running in Chrome.".to_string(),
        ("browser-capture", "errors") =>
            "Failed to retrieve console errors. Make sure BrowserTools extension is running in Chrome.".to_string(),
        ("browser-capture", "network") =>
            "Failed to retrieve network logs. Make sure BrowserTools extension is running in Chrome.".to_string(),
        ("browser-capture", "network-errors") =>
            "Failed to retrieve network errors. Make sure BrowserTools extension is running in Chrome.".to_string(),
        ("browser-capture", "clear") =>
            "Failed to clear logs. Make sure BrowserTools extension is running in Chrome.".to_string(),
        ("browser-capture", "element") =>
            "Failed to get DOM element. Make sure BrowserTools extension is running in Chrome.".to_string(),
        ("browser-audit", _) =>
            "Failed to run audit. Make sure BrowserTools extension is running in Chrome.".to_string(),
        ("browser-debug", _) =>
            "Failed to start debugger. Make sure BrowserTools extension is running in Chrome.".to_string(),
        _ => "Unknown command".to_string()
    }
}

fn process_api_response(response: String, api_url: &str) -> Result<String, String> {
    let endpoint = api_url.split('/').last().unwrap_or("");

    match serde_json::from_str::<ApiResponse>(&response) {
        Ok(api_response) => {
            if api_response.status == "success" {
                Ok(format_browser_tools_response(endpoint, api_response.data))
            } else {
                Ok(format!("Error from BrowserTools: {}", api_response.message))
            }
        },
        Err(e) => {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
                Ok(format_browser_tools_response(endpoint, json))
            } else {
                Ok(format!("Raw response from BrowserTools (parse error: {}): {}", e, response))
            }
        }
    }
}

fn call_browsertools_api(url: &str, method: &str, params: &serde_json::Value) -> Result<String, String> {
    let response = if method == "POST" {
        let json_string = serde_json::to_string(params)
            .map_err(|e| format!("JSON serialization error: {}", e))?;

        let request = http_client::HttpRequest::builder()
            .method(http_client::HttpMethod::Post)
            .url(url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(json_string.into_bytes())
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        request.fetch()
    } else {
        let request = http_client::HttpRequest::builder()
            .method(http_client::HttpMethod::Get)
            .url(url)
            .header("Accept", "application/json")
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        request.fetch()
    };

    match response {
        Ok(response) => String::from_utf8(response.body)
            .map_err(|e| format!("Failed to convert response body to UTF-8: {}", e)),
        Err(e) => Err(format!("HTTP request failed: {}", e)),
    }
}

fn format_browser_tools_response(endpoint: &str, data: serde_json::Value) -> String {
    match endpoint {
        "capture-screenshot" => {
            if data.get("message").and_then(|v| v.as_str()).is_some() {
                "Successfully saved screenshot".to_string()
            } else {
                format!("Screenshot captured: {}", serde_json::to_string_pretty(&data).unwrap_or_default())
            }
        },
        "console-logs" | "console-errors" => format_console_logs(&data),
        "network-success" | "network-errors" => {
            format!("Network logs:\n\n{}", serde_json::to_string_pretty(&data).unwrap_or_default())
        },
        "wipelogs" => {
            data.get("message")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Browser logs cleared successfully.".to_string())
        },
        "selected-element" => format_selected_element(&data),
        "accessibility-audit" | "performance-audit" | "seo-audit" | "best-practices-audit" | "nextjs-audit" => {
            format_audit_response(endpoint, &data)
        },
        "audit-all" => {
            format!("Audit Mode Results:\n\n{}", serde_json::to_string_pretty(&data).unwrap_or_default())
        },
        "debug-mode" => {
            format!("Debugger Mode Results:\n\n{}", serde_json::to_string_pretty(&data).unwrap_or_default())
        },
        _ => serde_json::to_string_pretty(&data).unwrap_or_default()
    }
}

fn format_console_logs(data: &serde_json::Value) -> String {
    if let Some(logs) = data.as_array() {
        let formatted_logs = logs.iter()
            .map(|log| {
                let level = log.get("level").and_then(|v| v.as_str()).unwrap_or("info");
                let message = log.get("message").and_then(|v| v.as_str()).unwrap_or("");
                format!("[{}] {}", level.to_uppercase(), message)
            })
            .collect::<Vec<String>>()
            .join("\n");

        if formatted_logs.is_empty() {
            "No console logs found.".to_string()
        } else {
            format!("Console Logs:\n\n{}", formatted_logs)
        }
    } else {
        format!("Console logs: {}", serde_json::to_string_pretty(data).unwrap_or_default())
    }
}

fn format_selected_element(data: &serde_json::Value) -> String {
    if let Some(element) = data.get("element") {
        let tag_name = element.get("tagName").and_then(|v| v.as_str()).unwrap_or("unknown");
        let class_name = element.get("className").and_then(|v| v.as_str()).unwrap_or("");
        let id = element.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let text = element.get("innerText").and_then(|v| v.as_str()).unwrap_or("");

        let mut element_info = format!("Selected DOM Element:\n- Tag: {}", tag_name);

        if !id.is_empty() {
            element_info.push_str(&format!("\n- ID: {}", id));
        }

        if !class_name.is_empty() {
            element_info.push_str(&format!("\n- Classes: {}", class_name));
        }

        if !text.is_empty() {
            element_info.push_str(&format!("\n- Text: {}", text));
        }

        if let Some(html) = element.get("outerHTML").and_then(|v| v.as_str()) {
            element_info.push_str(&format!("\n\nHTML:\n{}", html));
        }

        element_info
    } else {
        "No DOM element selected. Click on an element in the browser to select it.".to_string()
    }
}

fn format_audit_response(endpoint: &str, data: &serde_json::Value) -> String {
    let audit_type = match endpoint {
        "accessibility-audit" => "Accessibility",
        "performance-audit" => "Performance",
        "seo-audit" => "SEO",
        "best-practices-audit" => "Best Practices",
        "nextjs-audit" => "NextJS",
        _ => "Unknown"
    };

    // Try to extract score
    let score = data.get("score")
        .and_then(|v| v.as_f64())
        .map(|score| {
            let score_percentage = (score * 100.0).round() as i32;
            format!("Overall Score: {}%\n", score_percentage)
        })
        .unwrap_or_default();

    // Try to extract issues
    let issues = if let Some(issues) = data.get("issues").and_then(|v| v.as_array()) {
        if issues.is_empty() {
            "\nNo issues found!".to_string()
        } else {
            let mut issues_text = "\nIssues Found:\n".to_string();

            for (i, issue) in issues.iter().enumerate() {
                let title = issue.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown issue");
                let description = issue.get("description").and_then(|v| v.as_str()).unwrap_or("");

                issues_text.push_str(&format!("\n{}. {}\n", i + 1, title));
                if !description.is_empty() {
                    issues_text.push_str(&format!("   {}\n", description));
                }
            }

            issues_text
        }
    } else {
        String::new()
    };

    // If we extracted structured data, format it nicely
    if !score.is_empty() || !issues.is_empty() {
        format!("{} Audit Results:\n\n{}{}", audit_type, score, issues)
    } else {
        // Fall back to raw JSON if we couldn't extract structured data
        format!("{} Audit Results:\n\n{}", audit_type, serde_json::to_string_pretty(data).unwrap_or_default())
    }
}

zed::register_extension!(BrowserToolsExtension);
