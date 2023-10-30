pub(crate) mod manager;

mod plugin;

pub(crate) use plugin::Plugin;

// TODO: AFP
// TODO: SNMP
// TODO: SMB

#[cfg(feature = "amqp")]
pub(crate) mod amqp;
#[cfg(feature = "dns")]
pub(crate) mod dns;
#[cfg(feature = "ftp")]
mod ftp;
#[cfg(feature = "http")]
pub(crate) mod http;
#[cfg(feature = "imap")]
mod imap;
#[cfg(feature = "kerberos")]
pub(crate) mod kerberos;
#[cfg(feature = "ldap")]
pub(crate) mod ldap;
#[cfg(feature = "mongodb")]
pub(crate) mod mongodb;
#[cfg(feature = "mssql")]
mod mssql;
#[cfg(feature = "oracle")]
pub(crate) mod oracle; // optional as it requires libclntsh that's a pain to install and configure
#[cfg(feature = "pop3")]
pub(crate) mod pop3;
#[cfg(feature = "rdp")]
pub(crate) mod rdp;
#[cfg(feature = "redis_server")]
pub(crate) mod redis_server;
#[cfg(feature = "smtp")]
pub(crate) mod smtp;
#[cfg(feature = "sql")]
mod sql;
#[cfg(feature = "ssh")]
pub(crate) mod ssh;
#[cfg(feature = "stomp")]
pub(crate) mod stomp;
#[cfg(feature = "telnet")]
pub(crate) mod telnet;
#[cfg(feature = "vnc")]
pub(crate) mod vnc;
