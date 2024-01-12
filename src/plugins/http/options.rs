use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long, default_value = "200")]
    /// Comma separated status codes to consider as successful authentication attempts for HTTP based plugins.
    pub http_success_codes: String,
    #[clap(long)]
    /// Set a User-Agent. If none is specified, it'll be picked randomly for each request.
    pub http_ua: Option<String>,
    #[clap(long)]
    /// Check for the presence of this string in the response in order to recognize a succesful attempt.
    pub http_success_string: Option<String>,
    #[clap(long)]
    /// Check for the presence of this string in the response in order to recognize a failed attempt.
    pub http_failure_string: Option<String>,
    #[clap(long, default_value_t = false)]
    /// Follow HTTP redirects.
    pub http_follow_redirects: bool,
    #[clap(long, default_value = "GET")]
    /// Request method for HTTP based plugins.
    pub http_method: String,
    #[clap(long, num_args = 1..)]
    /// Request headers for HTTP based plugins.
    pub http_headers: Vec<String>,
    #[clap(long)]
    /// For each request grab a CSRF token from this page.
    pub http_csrf_page: Option<String>,
    #[clap(
        long,
        default_value = r#"<input type="hidden" name="(token)" value="([^"]+)""#,
        help_heading = "HTTP"
    )]
    /// Regular expression to use to grab the CSRF token name and value.
    pub http_csrf_regexp: String,
    #[clap(long)]
    /// Request payload (query string, post body or form data) for HTTP based plugins.
    pub http_payload: Option<String>,
    #[clap(long, default_value = "php")]
    /// File extension for HTTP enumeration.
    pub http_enum_ext: String,
    #[clap(long, default_value = "%EXT%")]
    /// File extension placeholder for HTTP enumeration wordlist.
    pub http_enum_ext_placeholder: String,
    #[clap(long)]
    /// Domain for NTLM authentication over HTTP.
    pub http_ntlm_domain: Option<String>,
    #[clap(long, default_value = "CLIENT")]
    /// Workstation name for NTLM authentication over HTTP.
    pub http_ntlm_workstation: String,
    #[clap(long)]
    /// Proxy URL.
    pub proxy: Option<String>,
    #[clap(long)]
    /// Proxy authentication as username:password.
    pub proxy_auth: Option<String>,
}
