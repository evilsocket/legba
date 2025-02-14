pub(crate) mod manager;

mod plugin;

pub(crate) use plugin::Plugin;

// TODO: AFP
// TODO: SNMP
// TODO: network discovery

macro_rules! plug {
    ($($(#[$meta:meta])* $vis:vis $name:ident;)*) => {
        $($(#[$meta])* $vis mod $name;)*

        pub(crate) fn add_defaults(registrar: &mut impl crate::plugins::manager::PluginRegistrar) {
            $(
                $(#[$meta])*
                {
                    $name::register(registrar);
                }
            )*
        }
    };
}

plug! {
    pub(crate) cmd;

    #[cfg(feature = "amqp")]
    pub(crate) amqp;
    #[cfg(feature = "dns")]
    pub(crate) dns;
    #[cfg(feature = "ftp")]
    ftp;
    #[cfg(feature = "http")]
    pub(crate) http;
    #[cfg(feature = "imap")]
    imap;
    #[cfg(feature = "irc")]
    pub(crate) irc;
    #[cfg(feature = "kerberos")]
    pub(crate) kerberos;
    #[cfg(feature = "ldap")]
    pub(crate) ldap;
    #[cfg(feature = "mongodb")]
    pub(crate) mongodb;
    #[cfg(feature = "mqtt")]
    pub(crate) mqtt;
    #[cfg(feature = "mssql")]
    mssql;
    #[cfg(feature = "oracle")]
    pub(crate) oracle; // optional as it requires libclntsh that's a pain to install and configure
    #[cfg(feature = "pop3")]
    pub(crate) pop3;
    #[cfg(feature = "port_scanner")]
    pub(crate) port_scanner;
    #[cfg(feature = "rdp")]
    pub(crate) rdp;
    #[cfg(feature = "redis")]
    pub(crate) redis;
    #[cfg(feature = "samba")]
    pub(crate) samba;
    #[cfg(feature = "scylla")]
    pub(crate) scylla;
    #[cfg(feature = "smtp")]
    pub(crate) smtp;
    #[cfg(feature = "socks5")]
    pub(crate) socks5;
    #[cfg(feature = "sql")]
    sql;
    #[cfg(feature = "ssh")]
    pub(crate) ssh;
    #[cfg(feature = "stomp")]
    pub(crate) stomp;
    #[cfg(feature = "telnet")]
    pub(crate) telnet;
    #[cfg(feature = "vnc")]
    pub(crate) vnc;
}
