use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    #[clap(long)]
    /// Set a User-Agent. If none is specified, it'll be picked randomly for each request.
    pub http_ua: Option<String>,

    #[clap(long, default_value = "status == 200")]
    /// Boolean expression to use to determine if a request is successful.
    pub http_success: String,

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
        default_value = r#"<input type="hidden" name="([^\"]+)" value="([^"]+)""#,
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

    // TODO: implement rotation over multiple proxies
    #[clap(long)]
    /// Proxy URL.
    pub proxy: Option<String>,
    #[clap(long)]
    /// Proxy authentication as username:password.
    pub proxy_auth: Option<String>,
}
