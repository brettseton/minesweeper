FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:7.0-alpine AS publish
ARG TARGETARCH
WORKDIR /src
COPY backend.csproj ./

# Map Docker TARGETARCH to .NET RID
RUN dotnet restore backend.csproj --runtime alpine-$(if [ "$TARGETARCH" = "amd64" ]; then echo "x64"; else echo "$TARGETARCH"; fi)
COPY . .

RUN dotnet publish backend.csproj -c Release -o /app/publish \
    --no-restore \
    --runtime alpine-$(if [ "$TARGETARCH" = "amd64" ]; then echo "x64"; else echo "$TARGETARCH"; fi) \
    --self-contained true \
    /p:PublishTrimmed=true \
    /p:PublishSingleFile=true

# Building final image used in running container
FROM mcr.microsoft.com/dotnet/runtime-deps:6.0-alpine AS final

RUN adduser --disabled-password \
  --home /app \
  --gecos '' dotnetuser && chown -R dotnetuser /app

USER dotnetuser

WORKDIR /app
EXPOSE 80
COPY --from=publish /app/publish .
ENV ASPNETCORE_URLS=http://*:8080

ENTRYPOINT ["./backend"]
