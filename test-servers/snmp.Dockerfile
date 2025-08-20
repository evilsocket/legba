FROM ubuntu:22.04

# Install SNMP packages and utilities
RUN apt-get update && apt-get install -y \
    snmpd \
    snmp \
    snmp-mibs-downloader \
    libsnmp-dev \
    python3 \
    python3-pip \
    net-tools \
    iputils-ping \
    vim \
    procps \
    lsof \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Enable MIB loading by commenting out the mibs line
RUN sed -i 's/^mibs :$/# mibs :/' /etc/snmp/snmp.conf

# Download MIBs
RUN download-mibs && echo "mibdirs +/usr/share/snmp/mibs" >> /etc/snmp/snmp.conf

# Create directory for custom MIBs and configs
RUN mkdir -p /usr/share/snmp/mibs /var/lib/snmp /etc/snmp/snmpd.conf.d

# Configure SNMP daemon - simplified to avoid duplicates
RUN echo "# SNMP Test Configuration\n\
    # System Information\n\
    sysLocation    DataCenter-01, Rack-42, Unit-15\n\
    sysContact     admin@testcorp.local\n\
    sysName        test-snmp-server.testcorp.local\n\
    sysServices    72\n\
    \n\
    # Listen on all interfaces\n\
    agentAddress udp:161,udp6:[::1]:161\n\
    \n\
    # SNMPv1/v2c Community Strings with different access levels\n\
    rocommunity public default\n\
    rocommunity monitor 192.168.0.0/16\n\
    rocommunity readonly 10.0.0.0/8\n\
    rwcommunity private 127.0.0.1\n\
    rwcommunity admin 192.168.1.0/24\n\
    \n\
    # Hidden community strings (for enumeration testing)\n\
    rocommunity secret123 default\n\
    rocommunity internal default\n\
    rocommunity snmpd default\n\
    \n\
    # Process monitoring\n\
    proc sshd 1 1\n\
    proc apache2 1 1\n\
    proc mysqld 1 1\n\
    proc nginx 1 1\n\
    \n\
    # Disk monitoring\n\
    disk / 10000\n\
    disk /var 10000\n\
    \n\
    # Load monitoring\n\
    load 12 10 5\n\
    \n\
    # Execute scripts (for testing command execution)\n\
    exec test-echo /bin/echo hello world\n\
    exec test-date /bin/date\n\
    exec test-uptime /usr/bin/uptime\n\
    \n\
    # Extend OID tree with custom data\n\
    extend test-custom /bin/echo Custom OID Extension Data\n\
    extend test-osversion /bin/cat /etc/os-release\n\
    extend test-processes /bin/ps aux\n\
    \n\
    # Interface monitoring\n\
    interface eth0 6 1000000000\n\
    \n\
    # Trap destinations\n\
    trapsink 192.168.1.100 public\n\
    trap2sink 192.168.1.101 public\n\
    \n\
    # Trap community\n\
    trapcommunity trapcomm\n\
    \n\
    # AgentX settings\n\
    master agentx\n\
    agentXSocket tcp:localhost:705\n\
    " > /etc/snmp/snmpd.conf

# Create SNMPv3 users configuration
RUN echo "# SNMPv3 Users\n\
    createUser noAuthUser\n\
    createUser authMD5User MD5 authPass123\n\
    createUser authSHAUser SHA authPassSHA256\n\
    createUser privDESUser MD5 authPass456 DES privPass456\n\
    createUser privAESUser SHA authPass789 AES privPass789\n\
    createUser adminUser SHA adminAuth123 AES adminPriv123\n\
    \n\
    rouser noAuthUser noauth\n\
    rouser authMD5User auth\n\
    rouser authSHAUser auth\n\
    rwuser privDESUser priv\n\
    rwuser privAESUser priv\n\
    rwuser adminUser priv\n\
    " >> /etc/snmp/snmpd.conf

# Create startup script
RUN echo '#!/bin/bash\n\
    \n\
    # Start some fake services for monitoring\n\
    python3 -m http.server 8080 &\n\
    python3 -m http.server 8081 &\n\
    \n\
    # Create some interesting data in environment\n\
    export DB_PASSWORD="SecretPass123"\n\
    export API_KEY="sk-1234567890abcdef"\n\
    export AWS_ACCESS_KEY_ID="AKIAIOSFODNN7EXAMPLE"\n\
    export AWS_SECRET_ACCESS_KEY="wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"\n\
    \n\
    # Create some files with interesting names\n\
    touch /tmp/backup.sql\n\
    touch /tmp/passwords.txt\n\
    touch /tmp/config.xml\n\
    echo "username=admin" > /tmp/credentials.conf\n\
    echo "password=admin123" >> /tmp/credentials.conf\n\
    \n\
    # Start SNMP daemon in foreground\n\
    echo "Starting SNMP daemon..."\n\
    exec /usr/sbin/snmpd -f -Lo -C -c /etc/snmp/snmpd.conf\n\
    ' > /start-snmpd.sh && chmod +x /start-snmpd.sh

# Expose SNMP port
EXPOSE 161/udp
EXPOSE 162/udp

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
    CMD snmpget -v2c -c public localhost sysDescr.0 || exit 1

# Set working directory
WORKDIR /root

# Default command
CMD ["/start-snmpd.sh"]