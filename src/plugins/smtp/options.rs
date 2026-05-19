use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone, Default)]
#[group(skip)]
pub(crate) struct Options {
    /// SMTP authentication mechanism: PLAIN (RFC4616), LOGIN (obsolete, used by office365), XOAUTH2, NTLM (NTLMv2, [MS-SMTPNTLM]) or NTLMv1.
    #[clap(long, default_value = "PLAIN")]
    pub smtp_mechanism: String,

    /// Upgrade the connection with STARTTLS after EHLO before authenticating. Required by most modern submission and Exchange servers.
    #[clap(long, default_value_t = false)]
    pub smtp_starttls: bool,

    /// NTLM domain to use when --smtp-mechanism is NTLM or NTLMv1.
    #[clap(long, default_value = "")]
    pub smtp_ntlm_domain: String,

    /// NTLM workstation identifier to use when --smtp-mechanism is NTLM or NTLMv1. Doubles as the EHLO host name.
    #[clap(long, default_value = "")]
    pub smtp_ntlm_workstation: String,
}
