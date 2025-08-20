docker build -t test_snmp -f snmp.Dockerfile .

docker run --privileged -p 161:161/udp -p 162:162/udp test_snmp