docker run \
    --cap-add SYS_PTRACE \
    -e "ACCEPT_EULA=1" \
    -e "MSSQL_SA_PASSWORD=test12345!Test" \
    -p 1433:1433 \
    --name mssql \
    mcr.microsoft.com/azure-sql-edge:latest
