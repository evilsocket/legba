use ahash::HashMap;
use lazy_static::lazy_static;
use snmp2::Oid;

lazy_static! {
    static ref OID_MAP: HashMap<Oid<'static>, &'static str> = {
        let mut oids = HashMap::default();

        // System MIB (1.3.6.1.2.1.1)
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 1, 1]).unwrap(),
            "sysDescr",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 1, 2]).unwrap(),
            "sysObjectID",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 1, 3]).unwrap(),
            "sysUpTime",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 1, 4]).unwrap(),
            "sysContact",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 1, 5]).unwrap(),
            "sysName",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 1, 6]).unwrap(),
            "sysLocation",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 1, 7]).unwrap(),
            "sysServices",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 1, 8]).unwrap(), "sysORLastChange");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 1, 9]).unwrap(), "sysORTable");

        // Interfaces MIB (1.3.6.1.2.1.2)
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 2, 1]).unwrap(),
            "ifNumber",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2]).unwrap(),
            "ifTable",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 1]).unwrap(),
            "ifIndex",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 2]).unwrap(),
            "ifDescr",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 3]).unwrap(),
            "ifType",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 4]).unwrap(), "ifMtu");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 5]).unwrap(), "ifSpeed");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 6]).unwrap(),
            "ifPhysAddress",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 7]).unwrap(), "ifAdminStatus");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 8]).unwrap(), "ifOperStatus");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 9]).unwrap(), "ifLastChange");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 10]).unwrap(), "ifInOctets");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 11]).unwrap(), "ifInUcastPkts");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 2, 2, 1, 16]).unwrap(), "ifOutOctets");

        // Network Address Translation (1.3.6.1.2.1.3)
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 3, 1]).unwrap(), "atTable");

        // IP MIB (1.3.6.1.2.1.4)
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 4, 1]).unwrap(),
            "ipForwarding",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 2]).unwrap(), "ipDefaultTTL");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 3]).unwrap(), "ipInReceives");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 4, 20]).unwrap(),
            "ipAddrTable",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 20, 1]).unwrap(), "ipAddrEntry");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 4, 21]).unwrap(),
            "ipRouteTable",
        );
        // IP Routing Table entries
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 21, 1, 1]).unwrap(), "ipRouteDest");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 21, 1, 2]).unwrap(), "ipRouteMask");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 21, 1, 7]).unwrap(), "ipRouteNextHop");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 21, 1, 8]).unwrap(), "ipRouteType");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 21, 1, 11]).unwrap(), "ipRouteMetric");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 21, 1, 13]).unwrap(), "ipRouteAge");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 21, 1, 14]).unwrap(), "ipRouteProto");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 21, 1, 15]).unwrap(), "ipRouteInfo");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 4, 22]).unwrap(),
            "ipNetToMediaTable",
        );
        // ARP Cache (ipNetToMediaTable entries)
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 22, 1, 1]).unwrap(), "ipNetToMediaIfIndex");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 22, 1, 2]).unwrap(), "ipNetToMediaPhysAddress");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 22, 1, 3]).unwrap(), "ipNetToMediaNetAddress");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 4, 22, 1, 4]).unwrap(), "ipNetToMediaType");

        // ICMP MIB (1.3.6.1.2.1.5)
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 5, 1]).unwrap(), "icmpInMsgs");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 5, 2]).unwrap(), "icmpInErrors");

        // TCP MIB (1.3.6.1.2.1.6)
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 1]).unwrap(), "tcpRtoAlgorithm");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 2]).unwrap(), "tcpRtoMin");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 3]).unwrap(), "tcpRtoMax");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 4]).unwrap(), "tcpMaxConn");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 5]).unwrap(), "tcpActiveOpens");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 9]).unwrap(), "tcpCurrEstab");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 10]).unwrap(), "tcpInSegs");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 11]).unwrap(), "tcpOutSegs");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 12]).unwrap(), "tcpRetransSegs");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 6, 13]).unwrap(),
            "tcpConnTable",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 14]).unwrap(), "tcpInErrs");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 6, 15]).unwrap(), "tcpOutRsts");

        // UDP MIB (1.3.6.1.2.1.7)
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 7, 1]).unwrap(), "udpInDatagrams");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 7, 2]).unwrap(), "udpNoPorts");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 7, 3]).unwrap(), "udpInErrors");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 7, 4]).unwrap(), "udpOutDatagrams");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 7, 5]).unwrap(),
            "udpTable",
        );

        // SNMP MIB (1.3.6.1.2.1.11)
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 11, 1]).unwrap(),
            "snmpInPkts",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 11, 2]).unwrap(),
            "snmpOutPkts",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 3]).unwrap(), "snmpInBadVersions");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 4]).unwrap(), "snmpInBadCommunityNames");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 5]).unwrap(), "snmpInBadCommunityUses");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 6]).unwrap(), "snmpInASNParseErrs");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 8]).unwrap(), "snmpInTooBigs");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 16]).unwrap(), "snmpInGetRequests");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 17]).unwrap(), "snmpInGetNexts");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 18]).unwrap(), "snmpInSetRequests");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 19]).unwrap(), "snmpInGetResponses");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 20]).unwrap(), "snmpInTraps");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 30]).unwrap(), "snmpEnableAuthenTraps");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 31]).unwrap(), "snmpSilentDrops");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 11, 32]).unwrap(), "snmpProxyDrops");

        // SNMPv2 Trap OIDs (1.3.6.1.2.1.11)
        oids.insert(Oid::from(&[1, 3, 6, 1, 6, 3, 1, 1, 5, 1]).unwrap(), "coldStart");
        oids.insert(Oid::from(&[1, 3, 6, 1, 6, 3, 1, 1, 5, 2]).unwrap(), "warmStart");
        oids.insert(Oid::from(&[1, 3, 6, 1, 6, 3, 1, 1, 5, 3]).unwrap(), "linkDown");
        oids.insert(Oid::from(&[1, 3, 6, 1, 6, 3, 1, 1, 5, 4]).unwrap(), "linkUp");
        oids.insert(Oid::from(&[1, 3, 6, 1, 6, 3, 1, 1, 5, 5]).unwrap(), "authenticationFailure");

        // Host Resources MIB (1.3.6.1.2.1.25)
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 25, 1, 1]).unwrap(),
            "hrSystemUptime",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 25, 1, 2]).unwrap(),
            "hrSystemDate",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 1, 3]).unwrap(), "hrSystemInitialLoadDevice");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 1, 4]).unwrap(), "hrSystemInitialLoadParameters");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 25, 1, 5]).unwrap(),
            "hrSystemNumUsers",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 25, 1, 6]).unwrap(),
            "hrSystemProcesses",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 1, 7]).unwrap(), "hrSystemMaxProcesses");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 2, 1]).unwrap(), "hrMemorySize");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 25, 2, 2]).unwrap(),
            "hrStorageTable",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 2, 3]).unwrap(), "hrStorageEntry");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 3, 2]).unwrap(), "hrDeviceTable");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 3, 3]).unwrap(), "hrProcessorTable");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 3, 4]).unwrap(), "hrNetworkTable");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 3, 5]).unwrap(), "hrPrinterTable");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 3, 6]).unwrap(), "hrDiskStorageTable");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 4, 1]).unwrap(), "hrSWOSIndex");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 25, 4, 2]).unwrap(),
            "hrSWRunTable",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 5, 1]).unwrap(), "hrSWRunPerfTable");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 6, 1]).unwrap(), "hrSWInstalledLastChange");
        oids.insert(Oid::from(&[1, 3, 6, 1, 2, 1, 25, 6, 2]).unwrap(), "hrSWInstalledLastUpdateTime");
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 2, 1, 25, 6, 3]).unwrap(),
            "hrSWInstalledTable",
        );

        // Enterprise MIBs - Microsoft (1.3.6.1.4.1.311)
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 311, 1, 1, 3, 1, 1]).unwrap(),
            "Windows version",
        );

        // Enterprise MIBs - Cisco (1.3.6.1.4.1.9)
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 9, 2, 1]).unwrap(),
            "Cisco local variables",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 9, 9, 23]).unwrap(),
            "Cisco CDP MIB",
        );

        // UCD-SNMP MIB (1.3.6.1.4.1.2021)
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 2]).unwrap(),
            "prTable",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 4]).unwrap(),
            "memory",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 8]).unwrap(),
            "extTable",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 9]).unwrap(),
            "dskTable",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 10]).unwrap(),
            "laTable",
        );
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 11]).unwrap(),
            "systemStats",
        );
        oids.insert(Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 13, 15]).unwrap(), "ucdExperimental");
        oids.insert(Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 15]).unwrap(), "fileTable");
        oids.insert(Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 16]).unwrap(), "logMatch");
        oids.insert(Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 100]).unwrap(), "version");
        oids.insert(Oid::from(&[1, 3, 6, 1, 4, 1, 2021, 101]).unwrap(), "snmpErrors");

        // NET-SNMP (1.3.6.1.4.1.8072)
        oids.insert(
            Oid::from(&[1, 3, 6, 1, 4, 1, 8072, 1, 3, 2]).unwrap(),
            "nsExtensions",
        );

        oids
    };
}

pub(crate) fn get_oid_name(oid: &Oid) -> String {
    let oid_str = oid.to_string();
    if let Some(desc) = OID_MAP.get(oid) {
        return desc.to_string();
    } else {
        // lookup parent oid
        let parts: Vec<u64> = oid_str.split('.').filter_map(|s| s.parse().ok()).collect();
        if parts.len() > 1 {
            let mut trimmed_parts = parts;
            trimmed_parts.pop();
            if let Ok(parent_oid) = Oid::from(&trimmed_parts)
                && let Some(desc) = OID_MAP.get(&parent_oid)
            {
                return desc.to_string();
            }
        }
    }

    oid_str
}
